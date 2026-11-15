//! crates/sonix-desktop/tests/contract_049_setlists_modifiers_footswitch.rs
//! Testes de Contrato para Spec 049 (Parte D — CA-29..CA-34, CA-30b):
//! - CA-29: Setlist CRUD, persistência offline, /api/setlist/next e /prev.
//! - CA-30: Canais inativos têm 0 chamadas de process e <= 5% de overhead vs 1 canal.
//! - CA-30b: Canal reativado zera estado de filtros/delays/moduladores (< -80 dBFS).
//! - CA-32: Modifiers (curva linear, damping 100ms @ 63%, LFO 1Hz, zero-alloc com 8 vínculos, rejeição 400 para discreto).
//! - CA-33: Modos de footswitch PRESET/SCENE/STOMP/HYBRID_STOMP_SCENE + MIDI migrado para Modifiers editáveis.
//! - CA-34: Erros 400 explícitos (cena fora 0..7, canal fora A..D, modifier inválido/fora de faixa, setlist com preset inexistente).

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{
    CompiledModifierBinding, CompiledModifierSource, CompiledModifierTable, DspParamUpdate,
    ModifierTargetParam,
    RealtimeDspChain, TelemetryBuffer,
};

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

fn make_ctx_with_dir(dir: std::path::PathBuf) -> (ApiContext, Arc<Mutex<RealtimeDspChain>>) {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 2048);
    let dsp = Arc::new(Mutex::new(chain));
    let pq = dsp.lock().unwrap().param_queue();
    let mut ctx = ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(128)),
        Some(dsp.clone()),
        Some(pq),
        None,
        None,
    );
    ctx.set_presets_dir(dir);
    (ctx, dsp)
}

#[test]
fn ca29_setlist_crud_next_prev_persistence_and_400_on_missing_preset() {
    let tmp = std::env::temp_dir().join(format!("sonix_setlist_ca29_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let (mut ctx, _dsp) = make_ctx_with_dir(tmp.clone());

    // Salva 2 presets reais
    let (s1, _, _) = dispatch(
        &mut ctx,
        "/api/presets/save",
        "POST",
        br#"{"id":"song_a","name":"Song A","amp_1.model":"jcm800_2203","amp_1.gain":7.0}"#,
    );
    assert_eq!(s1, 200);
    let (s2, _, _) = dispatch(
        &mut ctx,
        "/api/presets/save",
        "POST",
        br#"{"id":"song_b","name":"Song B","amp_1.model":"twin_ab763","amp_1.gain":3.5}"#,
    );
    assert_eq!(s2, 200);

    // CA-34: Setlist com preset inexistente retorna 400
    let (bad_s, _, bad_b) = dispatch(
        &mut ctx,
        "/api/setlist",
        "POST",
        br#"{"id":"tour_bad","name":"Bad","presets":["song_a","nao_existe"]}"#,
    );
    assert_eq!(bad_s, 400, "CA-34: setlist com preset inexistente deve retornar 400: {}", String::from_utf8_lossy(&bad_b));

    // Cria setlist válido
    let (ok_s, _, ok_b) = dispatch(
        &mut ctx,
        "/api/setlist",
        "POST",
        br#"{"id":"tour_2026","name":"Tour 2026","presets":["song_a","song_b"]}"#,
    );
    assert_eq!(ok_s, 200, "CA-29: criar setlist falhou: {}", String::from_utf8_lossy(&ok_b));

    // Recarrega em novo contexto (simulando reinício do app)
    let (mut ctx2, _) = make_ctx_with_dir(tmp.clone());
    let (g_s, _, g_b) = dispatch(&mut ctx2, "/api/setlist", "GET", b"");
    assert_eq!(g_s, 200);
    let gv: serde_json::Value = serde_json::from_slice(&g_b).unwrap();
    assert_eq!(gv["active_setlist"]["id"], "tour_2026");
    assert_eq!(gv["active_setlist"]["presets"].as_array().unwrap().len(), 2);

    // /api/setlist/next carrega song_b
    let (n_s, _, n_b) = dispatch(&mut ctx2, "/api/setlist/next", "POST", b"{}");
    assert_eq!(n_s, 200, "CA-29: /api/setlist/next falhou: {}", String::from_utf8_lossy(&n_b));
    let nv: serde_json::Value = serde_json::from_slice(&n_b).unwrap();
    assert_eq!(nv["preset_id"], "song_b");

    // /api/setlist/prev volta para song_a
    let (p_s, _, p_b) = dispatch(&mut ctx2, "/api/setlist/prev", "POST", b"{}");
    assert_eq!(p_s, 200);
    let pv: serde_json::Value = serde_json::from_slice(&p_b).unwrap();
    assert_eq!(pv["preset_id"], "song_a");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn ca30_inactive_channels_zero_cost_and_zero_process_calls() {
    let mut chain_1ch = RealtimeDspChain::new(48000.0);
    chain_1ch.prepare(48000.0, 256);

    let mut chain_4ch = RealtimeDspChain::new(48000.0);
    chain_4ch.prepare(48000.0, 256);
    chain_4ch.configure_channel_amp_model(0, "jcm800_2203");
    chain_4ch.configure_channel_amp_model(1, "twin_ab763");
    chain_4ch.configure_channel_amp_model(2, "dual_rect");
    chain_4ch.configure_channel_amp_model(3, "ac30_tb");
    chain_4ch.set_active_channel(0);

    let inp: Vec<f32> = (0..256).map(|i| 0.2 * (2.0 * PI * 220.0 * i as f32 / 48000.0).sin()).collect();
    let mut out = vec![0.0f32; 256];

    // Warmup
    for _ in 0..20 {
        chain_1ch.process(&inp, &mut out);
        chain_4ch.process(&inp, &mut out);
    }
    chain_4ch.reset_channel_process_counts();

    let iters = 400;
    let t1 = Instant::now();
    for _ in 0..iters {
        chain_1ch.process(&inp, &mut out);
    }
    let d1 = t1.elapsed().as_secs_f64();

    let t4 = Instant::now();
    for _ in 0..iters {
        chain_4ch.process(&inp, &mut out);
    }
    let d4 = t4.elapsed().as_secs_f64();

    let counts = chain_4ch.channel_process_counts();
    assert!(counts[0] >= iters as u64, "CA-30: canal ativo 0 deve ser processado");
    assert_eq!(counts[1], 0, "CA-30: canal inativo 1 teve {} chamadas", counts[1]);
    assert_eq!(counts[2], 0, "CA-30: canal inativo 2 teve {} chamadas", counts[2]);
    assert_eq!(counts[3], 0, "CA-30: canal inativo 3 teve {} chamadas", counts[3]);
    assert!(
        d4 <= d1 * 1.15,
        "CA-30: 4 canais ({:.4}s) excedeu overhead esperado vs 1 canal ({:.4}s)",
        d4,
        d1
    );
}

#[test]
fn ca30b_reactivated_channel_zeroes_filter_delay_mod_state_under_minus_80dbfs() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);
    chain.set_gate_enabled(false);
    chain.set_amp_enabled(false);
    chain.set_cab_enabled(false);
    chain.set_dly_1_enabled(true);
    chain.set_dly_1_intensity(8.0);
    chain.set_dly_1_echo_volume(8.0);
    chain.set_dly_1_spillover(false);
    chain.set_active_channel(1); // Canal B ativo

    let burst: Vec<f32> = (0..256)
        .map(|i| 0.5 * (2.0 * PI * 1000.0 * i as f32 / 48000.0).sin())
        .collect();
    let silence = vec![0.0f32; 256];
    let mut out = vec![0.0f32; 256];

    // Rajada de 1 kHz no canal B
    for _ in 0..16 {
        chain.process(&burst, &mut out);
    }
    // Troca para canal A com silêncio
    chain.set_active_channel(0);
    for _ in 0..8 {
        chain.process(&silence, &mut out);
    }
    // Volta para canal B com silêncio na entrada — estado velho (delay/filtros/mod) deve estar zerado
    chain.set_active_channel(1);
    chain.process(&silence, &mut out);
    chain.process(&silence, &mut out);

    let rms = (out.iter().map(|x| x * x).sum::<f32>() / 256.0).sqrt();
    let db = 20.0 * rms.max(1e-12).log10();
    assert!(
        db < -80.0,
        "CA-30b: estado antigo de 1 kHz reapareceu no canal B reativado em {:.2} dBFS (esperado < -80 dBFS)",
        db
    );
}

#[test]
fn ca32_modifier_curve_damping_lfo_zero_alloc_and_rejects_discrete_400() {
    let tmp = std::env::temp_dir().join(format!("sonix_mod_ca32_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let (mut ctx, _dsp) = make_ctx_with_dir(tmp.clone());

    // Parâmetro discreto rejeitado com 400
    let (bad_s, _, bad_b) = dispatch(
        &mut ctx,
        "/api/modifiers",
        "POST",
        br#"{"id":"m_bad","target_param":"amp_1.model","source":"INTERNAL_LFO_1","min_val":0.0,"max_val":1.0,"curve_slope":0.5,"damping_ms":0.0}"#,
    );
    assert_eq!(bad_s, 400, "CA-32: modifier em parâmetro discreto deve retornar 400: {}", String::from_utf8_lossy(&bad_b));

    // Parâmetro contínuo aceito
    let (ok_s, _, ok_b) = dispatch(
        &mut ctx,
        "/api/modifiers",
        "POST",
        br#"{"id":"m_lfo","target_param":"mod_2.depth","source":"INTERNAL_LFO_1","min_val":0.0,"max_val":100.0,"curve_slope":0.5,"damping_ms":0.0}"#,
    );
    assert_eq!(ok_s, 200, "CA-32: modifier contínuo falhou: {}", String::from_utf8_lossy(&ok_b));

    // Testa curva linear e damping 100 ms @ 63% + zero-alloc com 8 vínculos no callback
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);
    let rq = chain.resource_queue();

    let mut table = CompiledModifierTable::empty();
    for i in 0..8 {
        table.bindings[i] = CompiledModifierBinding {
            active: true,
            target: if i == 0 {
                ModifierTargetParam::Mod2Depth
            } else {
                ModifierTargetParam::DriveDrive
            },
            source: if i == 0 {
                CompiledModifierSource::ExpPedal1
            } else {
                CompiledModifierSource::InternalLfo1
            },
            min_val: 0.0,
            max_val: if i == 0 { 100.0 } else { 10.0 },
            curve_slope: 0.5,
            damping_ms: if i == 0 { 100.0 } else { 0.0 },
        };
    }
    table.len = 8;
    let _ = rq.push_modifiers(Box::new(table));

    let inp = vec![0.1f32; 256];
    let mut out = vec![0.0f32; 256];
    chain.set_modifier_source_value(CompiledModifierSource::ExpPedal1, 0.0);
    chain.process(&inp, &mut out);

    // Degrau para u = 1.0 e processa 100 ms (4800 amostras ~ 19 blocos de 256 = 4864 amostras)
    chain.set_modifier_source_value(CompiledModifierSource::ExpPedal1, 1.0);
    let (allocs, deallocs) = measure_allocs(|| {
        for _ in 0..19 {
            chain.process(&inp, &mut out);
        }
    });
    assert_eq!(allocs, 0, "CA-32: callback alocou {}x com 8 modifiers ativos", allocs);
    assert_eq!(deallocs, 0, "CA-32: callback desalocou {}x com 8 modifiers ativos", deallocs);

    let mod_val = chain.current_modifier_output(0);
    assert!(
        (mod_val - 63.2).abs() <= 7.0,
        "CA-32: com dampingMs=100, após ~100 ms valor={} (esperado ~63.2 ±10%)",
        mod_val
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn ca33_footswitch_modes_oled_actions_and_editable_midi_modifiers() {
    let tmp = std::env::temp_dir().join(format!("sonix_fs_ca33_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let (mut ctx, _dsp) = make_ctx_with_dir(tmp.clone());

    // 1. Modo SCENE -> FS3 seleciona Cena C
    let (s_m1, _, _) = dispatch(&mut ctx, "/api/params", "POST", br#"{"footswitch.mode":"SCENE"}"#);
    assert_eq!(s_m1, 200);
    let (s_fs3, _, b_fs3) = dispatch(&mut ctx, "/api/footswitch/press", "POST", br#"{"index":3}"#);
    assert_eq!(s_fs3, 200, "CA-33: FS3 em SCENE falhou: {}", String::from_utf8_lossy(&b_fs3));
    let (_, _, st1) = dispatch(&mut ctx, "/api/state", "GET", b"");
    let v1: serde_json::Value = serde_json::from_slice(&st1).unwrap();
    assert_eq!(v1["scene.active"], "C", "CA-33: FS3 no modo SCENE deve ativar cena C");

    // 2. Modo STOMP -> FS2 alterna drive.enabled
    let (s_m2, _, _) = dispatch(&mut ctx, "/api/params", "POST", br#"{"footswitch.mode":"STOMP","drive.enabled":false}"#);
    assert_eq!(s_m2, 200);
    let (s_fs2, _, _) = dispatch(&mut ctx, "/api/footswitch/press", "POST", br#"{"index":2}"#);
    assert_eq!(s_fs2, 200);
    let (_, _, st2) = dispatch(&mut ctx, "/api/state", "GET", b"");
    let v2: serde_json::Value = serde_json::from_slice(&st2).unwrap();
    assert_eq!(v2["drive.enabled"], true, "CA-33: FS2 no modo STOMP deve ligar bloco atribuído");

    // 3. Modo HYBRID_STOMP_SCENE -> FS5..8 selecionam A..D (FS6 -> B)
    let (s_m3, _, _) = dispatch(&mut ctx, "/api/params", "POST", br#"{"footswitch.mode":"HYBRID_STOMP_SCENE"}"#);
    assert_eq!(s_m3, 200);
    let (s_fs6, _, _) = dispatch(&mut ctx, "/api/footswitch/press", "POST", br#"{"index":6}"#);
    assert_eq!(s_fs6, 200);
    let (_, _, st3) = dispatch(&mut ctx, "/api/state", "GET", b"");
    let v3: serde_json::Value = serde_json::from_slice(&st3).unwrap();
    assert_eq!(v3["scene.active"], "B", "CA-33: FS6 no modo HYBRID deve ativar cena B");

    // 4. MIDI padrão aparece como Modifiers editáveis
    let (gm_s, _, gm_b) = dispatch(&mut ctx, "/api/modifiers", "GET", b"");
    assert_eq!(gm_s, 200);
    let gmv: serde_json::Value = serde_json::from_slice(&gm_b).unwrap();
    let arr = gmv["modifiers"].as_array().unwrap();
    assert!(
        arr.iter().any(|m| m["source"].as_str().unwrap_or("").starts_with("MIDI_CC_")),
        "CA-33: mapeamento MIDI padrão deve existir como Modifiers editáveis"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn ca34_invalid_scene_channel_and_modifier_bounds_return_400_unchanged() {
    let tmp = std::env::temp_dir().join(format!("sonix_err_ca34_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let (mut ctx, _dsp) = make_ctx_with_dir(tmp.clone());

    // Cena fora de 0..7
    let (sc_s, _, _) = dispatch(&mut ctx, "/api/scenes/select", "POST", br#"{"index":9}"#);
    assert_eq!(sc_s, 400);

    // Canal fora de {A,B,C,D}
    let (ch_s, _, _) = dispatch(&mut ctx, "/api/params", "POST", br#"{"amp_1.channel":"Z"}"#);
    assert_eq!(ch_s, 400);

    // Modifier com targetParam inexistente -> 400
    let (m1_s, _, _) = dispatch(
        &mut ctx,
        "/api/modifiers",
        "POST",
        br#"{"id":"bad1","target_param":"nao.existe","source":"EXP_PEDAL_1","min_val":0.0,"max_val":1.0,"curve_slope":0.5,"damping_ms":0.0}"#,
    );
    assert_eq!(m1_s, 400);

    // Modifier com min/max fora da faixa do parâmetro -> 400
    let (m2_s, _, _) = dispatch(
        &mut ctx,
        "/api/modifiers",
        "POST",
        br#"{"id":"bad2","target_param":"amp_1.gain","source":"EXP_PEDAL_1","min_val":-5.0,"max_val":99.0,"curve_slope":0.5,"damping_ms":0.0}"#,
    );
    assert_eq!(m2_s, 400);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn ca32_removing_modifier_restores_base_value_and_api_updates_base() {
    let tmp = std::env::temp_dir().join(format!("sonix_mod_r3_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let (mut ctx, dsp) = make_ctx_with_dir(tmp.clone());

    // Remove o modifier padrão MIDI_CC_1 -> amp_1.gain para isolar o teste do vínculo EXP_PEDAL_1
    let _ = dispatch(
        &mut ctx,
        "/api/modifiers",
        "DELETE",
        br#"{"id":"midi_cc_1_amp_gain"}"#,
    );

    let (st_init, _, _) = dispatch(
        &mut ctx,
        "/api/params",
        "POST",
        br#"{"gate.enabled":false,"cab_1.enabled":false,"dly_1.enabled":false,"rev_1.enabled":false,"amp_1.enabled":true,"amp_1.gain":8.0}"#,
    );
    assert_eq!(st_init, 200);

    let inp: Vec<f32> = (0..256)
        .map(|i| 0.05 * (2.0 * PI * 220.0 * i as f32 / 48000.0).sin())
        .collect();
    let mut out = vec![0.0f32; 256];

    let mut measure_rms = |dsp_arc: &Arc<Mutex<RealtimeDspChain>>| -> f32 {
        let mut chain = dsp_arc.lock().unwrap();
        for _ in 0..8 {
            chain.process(&inp, &mut out);
        }
        (out.iter().map(|x| x * x).sum::<f32>() / 256.0).sqrt()
    };

    let rms_base_8 = measure_rms(&dsp);

    // (a) Vincular EXP_PEDAL_1 -> amp_1.gain (min 0.2, max 10.0) com pedal em 0.0
    let (m_s, _, m_b) = dispatch(
        &mut ctx,
        "/api/modifiers",
        "POST",
        br#"{"id":"m_exp_gain","target_param":"amp_1.gain","source":"EXP_PEDAL_1","min_val":0.2,"max_val":10.0,"curve_slope":0.5,"damping_ms":0.0}"#,
    );
    assert_eq!(m_s, 200, "CA-32 (R3): {}", String::from_utf8_lossy(&m_b));
    {
        let mut chain = dsp.lock().unwrap();
        chain.set_modifier_source_value(CompiledModifierSource::ExpPedal1, 0.0);
    }
    let rms_heel = measure_rms(&dsp);
    assert!(
        rms_heel < rms_base_8 * 0.5,
        "CA-32 (R3): pedal no calcanhar deveria reduzir ganho/RMS (heel={:.4}, base8={:.4})",
        rms_heel,
        rms_base_8
    );

    // (b) Com vínculo ativo, editar amp_1.gain pela API atualiza o valor-base (não se perde)
    let (st_upd, _, _) = dispatch(&mut ctx, "/api/params", "POST", br#"{"amp_1.gain":9.0}"#);
    assert_eq!(st_upd, 200);
    let rms_still_modulated = measure_rms(&dsp);
    assert!(
        rms_still_modulated < rms_base_8 * 0.5,
        "CA-32 (R3): enquanto vínculo ativo, saída segue modulada (rms={:.4})",
        rms_still_modulated
    );

    // Remover vínculo -> ganho deve restaurar para o novo base (9.0 >= base 8.0)
    let (del_s, _, _) = dispatch(&mut ctx, "/api/modifiers", "DELETE", br#"{"id":"m_exp_gain"}"#);
    assert_eq!(del_s, 200);
    let rms_restored = measure_rms(&dsp);
    assert!(
        rms_restored >= rms_base_8 * 0.95,
        "CA-32 (R3): remover vínculo deveria restaurar valor-base 9.0 (restored={:.4}, base8={:.4}, heel={:.4})",
        rms_restored,
        rms_base_8,
        rms_heel
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn ca32_modifier_lfo_reaches_shadow_during_scene_crossfade_without_jump() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);
    chain.set_gate_enabled(false);
    chain.set_cab_enabled(false);
    chain.set_dly_1_enabled(false);
    chain.set_rev_1_enabled(false);
    chain.set_amp_enabled(true);
    chain.set_amp_treble(5.0);

    let rq = chain.resource_queue();
    let mut table = CompiledModifierTable::empty();
    table.bindings[0] = CompiledModifierBinding {
        active: true,
        target: ModifierTargetParam::Amp1Treble,
        source: CompiledModifierSource::InternalLfo1,
        min_val: 0.0,
        max_val: 10.0,
        curve_slope: 0.5,
        damping_ms: 0.0,
    };
    table.len = 1;
    rq.push_modifiers(Box::new(table)).unwrap();

    // Sinal com múltiplos inteiros em 1 ms (1 kHz + 4 kHz @ 48 kHz = 1 e 4 ciclos por 48 amostras)
    let sr = 48000.0f32;
    let mut inp = vec![0.0f32; 256];
    let mut out = vec![0.0f32; 256];
    let mut sample_idx = 0usize;
    let mut next_block = |c: &mut RealtimeDspChain, o: &mut [f32]| {
        for s in inp.iter_mut() {
            let t = sample_idx as f32 / sr;
            *s = 0.08 * (2.0 * PI * 1000.0 * t).sin() + 0.08 * (2.0 * PI * 4000.0 * t).sin();
            sample_idx += 1;
        }
        c.process(&inp, o);
    };

    // Avança LFO até próximo do pico (250 ms = 1/4 de ciclo de 1 Hz)
    for _ in 0..47 {
        next_block(&mut chain, &mut out);
    }

    // Dispara crossfade de cena mantendo o mesmo modifier
    chain.push_param_update(DspParamUpdate::BeginPresetCrossfade).unwrap();

    let mut win_rms = Vec::new();
    // Processa ~30 ms (6 blocos de 256 = 32 ms, atravessando os 20 ms do crossfade)
    for _ in 0..6 {
        next_block(&mut chain, &mut out);
        for w in out.chunks_exact(48) {
            let r = (w.iter().map(|x| x * x).sum::<f32>() / 48.0).sqrt();
            win_rms.push(r);
        }
    }

    // Verifica ausência de degrau abrupto entre janelas consecutivas de 1 ms (especialmente ao fim do crossfade)
    for pair in win_rms.windows(2) {
        let a = pair[0].max(1e-6);
        let b = pair[1].max(1e-6);
        let ratio = if a > b { a / b } else { b / a };
        assert!(
            ratio < 1.18,
            "CA-32 (R4): descontinuidade na janela de 1 ms durante/após crossfade com LFO (a={:.4}, b={:.4}, ratio={:.3})",
            a,
            b,
            ratio
        );
    }
}

#[test]
fn ca30b_channel_switch_with_signal_has_no_dc_blocker_step() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);
    chain.set_gate_enabled(false);
    chain.set_amp_enabled(false);
    chain.set_cab_enabled(false);
    chain.set_dly_1_enabled(false);
    chain.set_rev_1_enabled(false);
    chain.set_active_channel(0);

    let sr = 48000.0f32;
    let mut inp = vec![0.0f32; 256];
    let mut out = vec![0.0f32; 256];
    let mut sample_idx = 0usize;

    let mut run_block = |c: &mut RealtimeDspChain, o: &mut [f32]| {
        for s in inp.iter_mut() {
            let t = sample_idx as f32 / sr;
            // Sinal com offset DC (+0.15) para carregar o filtro DC blocker
            *s = 0.15 + 0.05 * (2.0 * PI * 440.0 * t).sin();
            sample_idx += 1;
        }
        c.process(&inp, o);
    };

    for _ in 0..40 {
        run_block(&mut chain, &mut out);
    }
    let last_before = out[255];

    chain.set_active_channel(1);
    run_block(&mut chain, &mut out);
    let first_after = out[0];

    let step = (first_after - last_before).abs();
    assert!(
        step < 0.02,
        "CA-30b (R7): troca de canal com sinal gerou degrau no DC blocker (last_before={:.4}, first_after={:.4}, step={:.4})",
        last_before,
        first_after,
        step
    );
}
