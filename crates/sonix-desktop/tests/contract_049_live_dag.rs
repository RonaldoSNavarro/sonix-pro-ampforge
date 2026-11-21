//! crates/sonix-desktop/tests/contract_049_live_dag.rs
//! Testes de Contrato para Spec 049 (Parte C — CA-23..CA-28):
//! - CA-23: Grade 4x12 dirige topologia ao vivo (drive antes vs depois do amp > 6 dB em harmônicos; /api/graph 48 slots).
//! - CA-24: Troca de topologia sem buraco de silêncio e zero alocações no callback (CountingAllocator).
//! - CA-25: Ciclo ou conexão para trás retorna 400 DagCycleError e preserva identidade da topologia ativa.
//! - CA-26: Split CROSSOVER (±0,2 dB), Mixer fase 180° (< -90 dBFS) e DYNAMIC_SPLIT pela API.
//! - CA-27: AMP_OUT_P10_NO_CAB entrega tap pré-gabinete (< -90 dBFS de diferença).
//! - CA-28: Bypass em linha paralela é isolado das demais linhas.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::json;
use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{CompiledDagTopology, DagBlockKind, RealtimeDspChain, TelemetryBuffer};

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
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 2048);
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

fn harmonic_mag_db(sig: &[f32], sr: f32, freq: f32) -> f32 {
    let n = sig.len();
    let mut re = 0.0f64;
    let mut im = 0.0f64;
    for (i, &x) in sig.iter().enumerate() {
        let w = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos());
        let ang = 2.0 * std::f64::consts::PI * (freq as f64) * (i as f64) / (sr as f64);
        re += (x as f64) * w * ang.cos();
        im -= (x as f64) * w * ang.sin();
    }
    let mag = (re * re + im * im).sqrt().max(1e-12);
    20.0 * (mag as f32).log10()
}

#[test]
fn ca23_moving_drive_before_vs_after_amp_changes_harmonics_and_graph_returns_48_slots() {
    let (mut ctx, dsp) = make_ctx();

    // GET /api/graph deve devolver 48 coordenadas
    let (st_g, _, body_g) = dispatch(&mut ctx, "/api/graph", "GET", b"");
    assert_eq!(st_g, 200);
    let vg: serde_json::Value = serde_json::from_slice(&body_g).unwrap();
    let slots = vg["slots"]
        .as_array()
        .expect("CA-23: /api/graph deve devolver array `slots` com 48 coordenadas");
    assert_eq!(slots.len(), 48, "CA-23: grade 4x12 deve ter 48 slots");

    // Configura drive + amp com ganho moderado e sem cab/delay/reverb para medir harmônicos puros
    let (st_p, _, _) = dispatch(
        &mut ctx,
        "/api/params",
        "POST",
        br#"{"gate.enabled":false,"drive.enabled":true,"drive.model":"rat","drive.drive":8.0,"drive.level":7.0,"amp_1.enabled":true,"amp_1.gain":7.5,"amp_1.master":6.0,"cab_1.enabled":false,"eq_1.enabled":false,"dly_1.enabled":false,"rev_1.enabled":false}"#,
    );
    assert_eq!(st_p, 200);

    let sr = 48000.0f32;
    let sine: Vec<f32> = (0..4096)
        .map(|i| 0.25 * (2.0 * PI * 220.0 * i as f32 / sr).sin())
        .collect();

    // 1. Drive ANTES do Amp (padrão: col 1 -> col 3)
    let mut out_before = vec![0.0f32; 4096];
    {
        let mut c = dsp.lock().unwrap();
        c.process(&sine, &mut out_before);
        c.process(&sine, &mut out_before);
    }

    // 2. Move Drive para DEPOIS do Amp via POST /api/graph (amp col 3, drive col 5)
    let move_payload = json!({
        "slots": [
            { "row": 0, "col": 0, "block": "gate" },
            { "row": 0, "col": 3, "block": "amp_1" },
            { "row": 0, "col": 5, "block": "drive" }
        ]
    });
    let (st_m, _, body_m) = dispatch(
        &mut ctx,
        "/api/graph",
        "POST",
        &serde_json::to_vec(&move_payload).unwrap(),
    );
    assert_eq!(st_m, 200, "CA-23: POST /api/graph falhou: {:?}", String::from_utf8_lossy(&body_m));

    let mut out_after = vec![0.0f32; 4096];
    {
        let mut c = dsp.lock().unwrap();
        c.process(&sine, &mut out_after);
        c.process(&sine, &mut out_after);
    }

    // Equaliza RMS
    let rms_a = (out_before[1024..].iter().map(|x| x * x).sum::<f32>() / 3072.0).sqrt().max(1e-9);
    let rms_b = (out_after[1024..].iter().map(|x| x * x).sum::<f32>() / 3072.0).sqrt().max(1e-9);
    let norm_a: Vec<f32> = out_before[1024..].iter().map(|x| x / rms_a).collect();
    let norm_b: Vec<f32> = out_after[1024..].iter().map(|x| x / rms_b).collect();

    let max_harm_diff = (2..=7)
        .map(|h| {
            let f = 220.0 * h as f32;
            (harmonic_mag_db(&norm_a, sr, f) - harmonic_mag_db(&norm_b, sr, f)).abs()
        })
        .fold(0.0f32, f32::max);
    assert!(
        max_harm_diff > 6.0,
        "CA-23: mover drive antes/depois do amp mudou harmônicos em apenas {:.2} dB (esperado > 6 dB)",
        max_harm_diff
    );
}

#[test]
fn ca24_topology_swap_has_no_silence_hole_and_zero_audio_allocs() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);
    let rq = chain.resource_queue();

    let sr = 48000.0f32;
    let block = 256usize;
    let mut input = vec![0.0f32; block];
    let mut output = vec![0.0f32; block];

    // Aquecimento
    for b in 0..8 {
        for i in 0..block {
            let t = (b * block + i) as f32 / sr;
            input[i] = 0.3 * (2.0 * PI * 330.0 * t).sin();
        }
        chain.process(&input, &mut output);
    }

    // Compila nova topologia FORA da thread de áudio e envia pela fila lock-free
    let mut topo = CompiledDagTopology::default_layout(99);
    topo.slots[2] = None;
    topo.slots[7] = Some(DagBlockKind::Drive);
    let _ = rq.push_topology(Box::new(topo));

    let (allocs, deallocs) = measure_allocs(|| {
        for b in 8..24 {
            for i in 0..block {
                let t = (b * block + i) as f32 / sr;
                input[i] = 0.3 * (2.0 * PI * 330.0 * t).sin();
            }
            chain.process(&input, &mut output);
            // Verifica que nenhuma janela de 48 amostras (~1ms) caiu para silêncio
            for win in output.chunks_exact(48) {
                let rms = (win.iter().map(|x| x * x).sum::<f32>() / 48.0).sqrt();
                assert!(rms > 0.01, "CA-24: buraco na troca de topologia (rms={})", rms);
            }
        }
    });
    assert_eq!(allocs, 0, "CA-24: callback alocou {}x durante troca de topologia", allocs);
    assert_eq!(deallocs, 0, "CA-24: callback desalocou {}x durante troca de topologia", deallocs);
    rq.reclaim_retired();
}

#[test]
fn ca25_cycle_returns_400_dag_cycle_error_and_preserves_topology_id() {
    let (mut ctx, dsp) = make_ctx();
    let id_before = dsp.lock().unwrap().topology_id();

    let bad = json!({
        "connections": [
            { "from": { "row": 0, "col": 5 }, "to": { "row": 0, "col": 3 } }
        ]
    });
    let (st, _, body) = dispatch(
        &mut ctx,
        "/api/graph",
        "POST",
        &serde_json::to_vec(&bad).unwrap(),
    );
    let txt = String::from_utf8_lossy(&body);
    assert_eq!(st, 400, "CA-25: conexão para trás/ciclo deve retornar 400, obtido {}: {}", st, txt);
    assert!(txt.contains("DagCycleError"), "CA-25: erro deve citar DagCycleError: {}", txt);
    assert_eq!(
        dsp.lock().unwrap().topology_id(),
        id_before,
        "CA-25: topologia em execução não pode mudar após ciclo rejeitado"
    );
}

#[test]
fn ca26_split_crossover_dynamic_and_phase_invert_merge_via_api() {
    let (mut ctx, dsp) = make_ctx();
    let (st, _, _) = dispatch(
        &mut ctx,
        "/api/params",
        "POST",
        br#"{"gate.enabled":false,"drive.enabled":false,"amp_1.enabled":false,"cab_1.enabled":false,"eq_1.enabled":false,"comp_2.enabled":false,"amp_2.enabled":false,"cab_2.enabled":false,"mod_2.enabled":false,"dly_1.enabled":false,"rev_1.enabled":false,"input.boost":1.0,"split.enabled":true,"split.mode":"CROSSOVER","split.crossover_hz":500.0,"merge.row_1_level_db":0.0,"merge.row_1_phase_invert":false}"#,
    );
    assert_eq!(st, 200);

    // 1. Crossover a 500 Hz soma plano em ±0.2 dB de 40 Hz a 4 kHz
    let sr = 48000.0f32;
    let n = 4096usize;
    let mut amps = Vec::new();
    for f in [40.0f32, 200.0, 500.0, 1500.0, 4000.0] {
        let sig: Vec<f32> = (0..n).map(|i| 0.4 * (2.0 * PI * f * i as f32 / sr).sin()).collect();
        let mut out = vec![0.0f32; n];
        {
            let mut c = dsp.lock().unwrap();
            c.process(&sig, &mut out);
        }
        let rms = (out[2048..].iter().map(|x| x * x).sum::<f32>() / 2048.0).sqrt();
        amps.push(20.0 * rms.log10());
    }
    let min_db = amps.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_db = amps.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        (max_db - min_db) <= 0.4,
        "CA-26: crossover 500 Hz variação pico-a-pico {:.3} dB (> ±0.2 dB)",
        max_db - min_db
    );

    // 2. Mixer fase 180° soma abaixo de -90 dBFS
    let (st2, _, _) = dispatch(
        &mut ctx,
        "/api/params",
        "POST",
        br#"{"split.mode":"ab","split.ratio":0.5,"merge.row_1_level_db":0.0,"merge.row_1_phase_invert":true}"#,
    );
    assert_eq!(st2, 200);
    let sig: Vec<f32> = (0..n).map(|i| 0.5 * (2.0 * PI * 440.0 * i as f32 / sr).sin()).collect();
    let mut out_inv = vec![0.0f32; n];
    {
        let mut c = dsp.lock().unwrap();
        c.process(&sig, &mut out_inv);
    }
    let rms_inv = (out_inv[1024..].iter().map(|x| x * x).sum::<f32>() / 3072.0).sqrt();
    let db_inv = 20.0 * rms_inv.max(1e-12).log10();
    assert!(db_inv < -90.0, "CA-26: cancelamento de fase = {:.1} dBFS (esperado < -90 dBFS)", db_inv);

    // 3. DYNAMIC_SPLIT aceito pela API
    let (st3, _, body3) = dispatch(
        &mut ctx,
        "/api/params",
        "POST",
        br#"{"split.mode":"DYNAMIC_SPLIT","merge.row_1_phase_invert":false}"#,
    );
    assert_eq!(st3, 200, "CA-26: DYNAMIC_SPLIT deve ser aceito: {:?}", String::from_utf8_lossy(&body3));
}

#[test]
fn ca27_amp_out_p10_no_cab_matches_pre_cab_tap_under_minus_90dbfs() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 512);
    chain.set_gate_enabled(false);
    chain.set_cab_enabled(true);

    let sr = 48000.0f32;
    let sig: Vec<f32> = (0..512).map(|i| 0.25 * (2.0 * PI * 330.0 * i as f32 / sr).sin()).collect();
    let mut out_with_cab = vec![0.0f32; 512];

    // Compara contra cadeia idêntica com cab_enabled = false (após crossfade de 10 ms assentar)
    let mut chain_nocab = RealtimeDspChain::new(48000.0);
    chain_nocab.prepare(48000.0, 512);
    chain_nocab.set_gate_enabled(false);
    chain_nocab.set_cab_enabled(false);
    let mut expected_nocab = vec![0.0f32; 512];

    chain.process(&sig, &mut out_with_cab);
    chain_nocab.process(&sig, &mut expected_nocab);
    chain.process(&sig, &mut out_with_cab);
    chain_nocab.process(&sig, &mut expected_nocab);

    let mut p10_tap = vec![0.0f32; 512];
    chain.read_amp_out_p10_no_cab(&mut p10_tap);

    let diff_rms = (p10_tap
        .iter()
        .zip(expected_nocab.iter())
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f32>()
        / 512.0)
        .sqrt();
    let diff_db = 20.0 * diff_rms.max(1e-12).log10();
    assert!(diff_db < -90.0, "CA-27: diferença AMP_OUT_P10_NO_CAB = {:.1} dBFS", diff_db);
}

#[test]
fn ca28_parallel_row_bypass_isolated_from_row_0() {
    let (mut ctx, _) = make_ctx();
    let (st1, _, _) = dispatch(
        &mut ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.enabled":true,"amp_1.gain":6.5,"comp_2.enabled":false}"#,
    );
    assert_eq!(st1, 200);
    let (st2, _, _) = dispatch(&mut ctx, "/api/params", "POST", br#"{"comp_2.enabled":true}"#);
    assert_eq!(st2, 200);
    let (_, _, st_body) = dispatch(&mut ctx, "/api/state", "GET", b"");
    let v: serde_json::Value = serde_json::from_slice(&st_body).unwrap();
    assert_eq!(v["amp_1.enabled"], true);
    assert!((v["amp_1.gain"].as_f64().unwrap() - 6.5).abs() < 1e-4);
}

#[test]
fn ca24_reclaim_retired_drains_1000_topology_swaps_with_zero_leaks_and_zero_rt_deallocs() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 64);
    let rq = chain.resource_queue();

    let input = [0.1f32; 64];
    let mut output = [0.0f32; 64];

    for i in 1..=1000u64 {
        let topo = CompiledDagTopology::default_layout(i + 1);
        assert!(
            rq.push_topology(Box::new(topo)).is_ok(),
            "CA-24 (R5/R9): push_topology falhou na troca {}",
            i
        );
        let (allocs, deallocs) = measure_allocs(|| {
            chain.process(&input, &mut output);
        });
        assert_eq!(allocs, 0, "CA-24 (R5/R9): alocação no callback na troca {}", i);
        assert_eq!(deallocs, 0, "CA-24 (R5/R9): desalocação no callback na troca {}", i);
        rq.reclaim_retired();
    }

    assert_eq!(
        rq.reclaimed_count(),
        1000,
        "CA-24 (R5/R9): esperado 1000 topologias recuperadas pelo dreno (zero vazamento), obtido {}",
        rq.reclaimed_count()
    );
}

#[test]
fn ca40_mod2_active_with_split_disabled_modulates_output() {
    let sr = 48000.0f32;
    let inp: Vec<f32> = (0..256)
        .map(|i| 0.3 * (2.0 * PI * 440.0 * i as f32 / sr).sin())
        .collect();

    let run_chain = |split_on: bool, mod2_on: bool| -> Vec<f32> {
        let mut chain = RealtimeDspChain::new(sr);
        chain.prepare(sr, 256);
        chain.set_gate_enabled(false);
        chain.set_amp_enabled(false);
        chain.set_cab_enabled(false);
        chain.set_amp_2_enabled(false);
        chain.set_cab_2_enabled(false);
        chain.set_dly_1_enabled(false);
        chain.set_rev_1_enabled(false);
        chain.set_merge_config(0.0, false);
        chain.set_split_enabled(split_on);
        chain.set_mod_2_enabled(mod2_on);
        chain.set_mod_2_depth(80.0);
        chain.set_mod_2_rate_hz(2.0);
        chain.set_mod_2_mix(50.0);
        let mut out = vec![0.0f32; 256];
        for _ in 0..20 {
            chain.process(&inp, &mut out);
        }
        out
    };

    // AUD-21: com split desligado, Linha 2 (comp_2/mod_2) NÃO processa.
    let off_no_split = run_chain(false, false);
    let on_no_split = run_chain(false, true);
    let diff_no_split = (off_no_split
        .iter()
        .zip(on_no_split.iter())
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f32>()
        / 256.0)
        .sqrt();
    assert!(
        diff_no_split < 1e-6,
        "AUD-21: com split desligado mod_2 não pode alterar saída (diff_rms={:.6})",
        diff_no_split
    );

    // Com split ligado, mod_2 atua normalmente na Linha 2.
    let off_with_split = run_chain(true, false);
    let on_with_split = run_chain(true, true);
    let diff_with_split = (off_with_split
        .iter()
        .zip(on_with_split.iter())
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f32>()
        / 256.0)
        .sqrt();
    assert!(
        diff_with_split > 0.01,
        "CA-40 / AUD-21: mod_2 ligado com split ativo deve modular saída (diff_rms={:.6})",
        diff_with_split
    );
}
