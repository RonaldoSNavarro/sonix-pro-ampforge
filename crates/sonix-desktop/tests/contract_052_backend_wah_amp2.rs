//! crates/sonix-desktop/tests/contract_052_backend_wah_amp2.rs
//! Testes de Contrato Backend para Spec 052:
//! - CA-3: `wah_1.*` (enabled, position, q, mode + EXP 1 + DagBlockKind::Wah1) validados pela API e com efeito mensurável + zero-alloc.
//! - CA-4: `amp_2.*` (gain, bass, middle, treble, master) validados pela API e atuando no motor em caminho paralelo.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::json;
use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{RealtimeDspChain, TelemetryBuffer};

struct CountingAllocator;
thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static DEALLOCS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.with(|c| c.get()) {
            ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if COUNTING.with(|c| c.get()) {
            DEALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL_ALLOC: CountingAllocator = CountingAllocator;

struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        COUNTING.with(|c| c.set(false));
    }
}

fn measure_allocs<F: FnOnce()>(f: F) -> (usize, usize) {
    ALLOCS.store(0, Ordering::SeqCst);
    DEALLOCS.store(0, Ordering::SeqCst);
    COUNTING.with(|c| c.set(true));
    let _g = Guard;
    f();
    drop(_g);
    (ALLOCS.load(Ordering::SeqCst), DEALLOCS.load(Ordering::SeqCst))
}

fn make_ctx() -> (ApiContext, Arc<Mutex<RealtimeDspChain>>) {
    let mut chain = RealtimeDspChain::new(48_000.0);
    chain.prepare(48_000.0, 2048);
    let dsp = Arc::new(Mutex::new(chain));
    let pq = dsp.lock().unwrap().param_queue();
    let ctx = ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(128)),
        Some(dsp.clone()),
        Some(pq),
        None,
        None,
    );
    (ctx, dsp)
}

fn post_params(ctx: &ApiContext, payload: serde_json::Value) -> (u16, String) {
    let raw = payload.to_string();
    let (status, _ct, body) = dispatch(ctx, "/api/params", "POST", raw.as_bytes());
    (status, String::from_utf8_lossy(&body).into_owned())
}

fn render_dual_tone(dsp: &Arc<Mutex<RealtimeDspChain>>, f1: f32, f2: f32, blocks: usize) -> Vec<f32> {
    let sr = 48_000.0f32;
    let bs = 256usize;
    let mut all = Vec::with_capacity(blocks * bs);
    let mut ch = dsp.lock().unwrap();
    for b in 0..blocks {
        let mut inp = vec![0.0f32; bs];
        for i in 0..bs {
            let t = (b * bs + i) as f32 / sr;
            inp[i] = 0.25 * (2.0 * PI * f1 * t).sin() + 0.25 * (2.0 * PI * f2 * t).sin();
        }
        let mut out_l = vec![0.0f32; bs];
        ch.process(&inp, &mut out_l);
        all.extend_from_slice(&out_l);
    }
    all
}

fn goertzel_amp(sig: &[f32], freq: f32, sr: f32) -> f32 {
    let n = sig.len();
    let omega = 2.0 * PI * freq / sr;
    let coeff = 2.0 * omega.cos() as f64;
    let mut s1 = 0.0f64;
    let mut s2 = 0.0f64;
    for &x in sig {
        let s = (x as f64) + coeff * s1 - s2;
        s2 = s1;
        s1 = s;
    }
    let power = s2 * s2 + s1 * s1 - coeff * s1 * s2;
    (power.max(0.0).sqrt() / (n as f64 * 0.5)) as f32
}

fn rms(sig: &[f32]) -> f32 {
    let sum: f32 = sig.iter().map(|x| x * x).sum();
    (sum / sig.len().max(1) as f32).sqrt()
}

#[test]
fn test_ca_03_api_wah_1_validation_spectral_sweep_exp1_and_zero_alloc() {
    let (ctx, dsp) = make_ctx();

    // Validação atômica rejeita fora de faixa/inválido e preserva estado
    let (st_bad_pos, _) = post_params(&ctx, json!({ "wah_1.position": 1.5 }));
    assert_eq!(st_bad_pos, 400);
    let (st_bad_q, _) = post_params(&ctx, json!({ "wah_1.q": 15.0 }));
    assert_eq!(st_bad_q, 400);
    let (st_bad_mode, _) = post_params(&ctx, json!({ "wah_1.mode": "desconhecido" }));
    assert_eq!(st_bad_mode, 400);

    // Configura cadeia isolada com Wah no caminho direto pré-amp
    let setup = |pos: f32, mode: &str| -> Vec<f32> {
        let (c, d) = make_ctx();
        let (st, body) = post_params(
            &c,
            json!({
                "gate.enabled": false,
                "amp_1.enabled": false,
                "cab_1.enabled": false,
                "wah_1.enabled": true,
                "wah_1.mode": mode,
                "wah_1.q": 6.0,
                "wah_1.position": pos
            }),
        );
        assert_eq!(st, 200, "CA-3: POST wah_1 deve retornar 200, obteve: {body}");
        render_dual_tone(&d, 400.0, 2000.0, 16)
    };

    let heel = setup(0.0, "crybaby_gcb95");
    let toe = setup(1.0, "crybaby_gcb95");
    let tail_heel = &heel[heel.len() / 2..];
    let tail_toe = &toe[toe.len() / 2..];

    let heel_400 = goertzel_amp(tail_heel, 400.0, 48_000.0);
    let heel_2k = goertzel_amp(tail_heel, 2000.0, 48_000.0);
    let toe_400 = goertzel_amp(tail_toe, 400.0, 48_000.0);
    let toe_2k = goertzel_amp(tail_toe, 2000.0, 48_000.0);

    assert!(
        heel_400 > heel_2k * 2.0,
        "CA-3: Wah no calcanhar deve enfatizar 400 Hz sobre 2000 Hz (400={heel_400:.4}, 2k={heel_2k:.4})"
    );
    assert!(
        toe_2k > heel_2k * 3.0 && toe_2k > toe_400 * 1.5,
        "CA-3: Wah na ponta deve deslocar pico para 2 kHz (toe_2k={toe_2k:.4}, heel_2k={heel_2k:.4}, toe_400={toe_400:.4})"
    );

    // Integração com palco.exp_1 quando wah_1.enabled == true
    let (st_exp, _) = post_params(
        &ctx,
        json!({
            "gate.enabled": false,
            "amp_1.enabled": false,
            "cab_1.enabled": false,
            "wah_1.enabled": true,
            "wah_1.q": 6.0,
            "palco.exp_1": 100.0
        }),
    );
    assert_eq!(st_exp, 200);
    let out_exp = render_dual_tone(&dsp, 400.0, 2000.0, 16);
    let exp_tail = &out_exp[out_exp.len() / 2..];
    assert!(
        goertzel_amp(exp_tail, 2000.0, 48_000.0) > goertzel_amp(exp_tail, 400.0, 48_000.0),
        "CA-3: palco.exp_1=100 deve abrir o Wah para a região aguda (~2 kHz)"
    );

    // CountingAllocator zero-alloc no callback RT com Wah ativo
    let inp = [0.1f32; 256];
    let mut out_l = [0.0f32; 256];
    let (allocs, deallocs) = measure_allocs(|| {
        let mut ch = dsp.lock().unwrap();
        for _ in 0..32 {
            ch.process(&inp, &mut out_l);
        }
    });
    assert_eq!(allocs, 0, "CA-3: callback RT com Wah alocou {allocs} vez(es)");
    assert_eq!(deallocs, 0, "CA-3: callback RT com Wah desalocou {deallocs} vez(es)");
}

#[test]
fn test_ca_04_api_amp_2_controls_validated_and_measurably_affect_parallel_chain() {
    let (ctx_check, _) = make_ctx();
    let (st_bad_gain, _) = post_params(&ctx_check, json!({ "amp_2.gain": 12.0 }));
    assert_eq!(st_bad_gain, 400);
    let (st_bad_master, _) = post_params(&ctx_check, json!({ "amp_2.master": -1.0 }));
    assert_eq!(st_bad_master, 400);

    let run_amp2 = |overrides: serde_json::Value| -> Vec<f32> {
        let (c, d) = make_ctx();
        let mut base = json!({
            "gate.enabled": false,
            "amp_1.enabled": true,
            "amp_1.standby": true,
            "cab_1.enabled": false,
            "split.enabled": true,
            "split.mode": "ab",
            "amp_2.enabled": true,
            "amp_2.model": "twin_ab763",
            "cab_2.enabled": false,
            "merge.row_1_level_db": 0.0,
            "amp_2.gain": 5.0,
            "amp_2.bass": 5.0,
            "amp_2.middle": 5.0,
            "amp_2.treble": 5.0,
            "amp_2.master": 7.0
        });
        if let (Some(bm), Some(om)) = (base.as_object_mut(), overrides.as_object()) {
            for (k, v) in om {
                bm.insert(k.clone(), v.clone());
            }
        }
        let (st, body) = post_params(&c, base);
        assert_eq!(st, 200, "CA-4: POST amp_2.* falhou: {body}");
        render_dual_tone(&d, 110.0, 2200.0, 12)
    };

    let m_lo = run_amp2(json!({ "amp_2.master": 1.0 }));
    let m_hi = run_amp2(json!({ "amp_2.master": 9.5 }));
    assert!(
        rms(&m_hi[m_hi.len() / 2..]) > rms(&m_lo[m_lo.len() / 2..]) * 1.4,
        "CA-4: amp_2.master alto deve elevar nível da linha paralela"
    );

    let g_lo = run_amp2(json!({ "amp_2.gain": 1.0 }));
    let g_hi = run_amp2(json!({ "amp_2.gain": 9.5 }));
    let g_delta: f32 = g_lo[g_lo.len() / 2..]
        .iter()
        .zip(&g_hi[g_hi.len() / 2..])
        .map(|(a, b)| (a - b).abs())
        .sum();
    assert!(g_delta > 1.0, "CA-4: amp_2.gain deve alterar resposta (delta={g_delta})");

    for key in ["amp_2.bass", "amp_2.middle", "amp_2.treble"] {
        let eq_lo = run_amp2(json!({ key: 0.5 }));
        let eq_hi = run_amp2(json!({ key: 9.5 }));
        let delta: f32 = eq_lo[eq_lo.len() / 2..]
            .iter()
            .zip(&eq_hi[eq_hi.len() / 2..])
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(delta > 0.25, "CA-4: controle {key} deve alterar espectro de saída (delta={delta})");
    }
}
