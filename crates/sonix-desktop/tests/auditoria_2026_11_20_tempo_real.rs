//! Testes de auditoria 2026-11-20 — Tempo real no host desktop (AUD-02, AUD-06, AUD-07)

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use ampforge_dsp::denormal::mxcsr_atual;
use sonix_desktop::audio::RealtimeDspChain;

struct CountingAllocator;

thread_local! {
    static COUNTING_ACTIVE: Cell<bool> = const { Cell::new(false) };
    static THREAD_ALLOCS: Cell<usize> = const { Cell::new(0) };
    static THREAD_DEALLOCS: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        COUNTING_ACTIVE.with(|active| {
            if active.get() {
                THREAD_ALLOCS.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        COUNTING_ACTIVE.with(|active| {
            if active.get() {
                THREAD_DEALLOCS.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

fn measure_allocs<F: FnOnce()>(f: F) -> (usize, usize) {
    THREAD_ALLOCS.with(|c| c.set(0));
    THREAD_DEALLOCS.with(|c| c.set(0));
    COUNTING_ACTIVE.with(|a| a.set(true));
    f();
    COUNTING_ACTIVE.with(|a| a.set(false));
    let allocs = THREAD_ALLOCS.with(|c| c.get());
    let deallocs = THREAD_DEALLOCS.with(|c| c.get());
    (allocs, deallocs)
}

#[test]
fn aud_02_process_roda_com_ftz_daz() {
    let mut chain = RealtimeDspChain::new(48_000.0);
    chain.prepare(48_000.0, 128);
    let before = mxcsr_atual();

    let mut in_buf = vec![0.0_f32; 128];
    let mut out_buf = vec![0.0_f32; 128];
    for (i, s) in in_buf.iter_mut().enumerate() {
        *s = (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / 48_000.0)).sin() * 0.25;
    }

    chain.process(&in_buf, &mut out_buf);
    let after = mxcsr_atual();

    assert_eq!(
        chain.mxcsr_observado() & 0x8040,
        0x8040,
        "AUD-02: process() deve rodar com FTZ e DAZ (0x8040) ligados, observou 0x{:04x}",
        chain.mxcsr_observado()
    );
    assert_eq!(
        after, before,
        "AUD-02: MXCSR do thread chamador não deve ser alterado após process()"
    );
}

#[test]
fn aud_02_zero_alloc() {
    let mut chain = RealtimeDspChain::new(48_000.0);
    chain.prepare(48_000.0, 128);
    let in_buf = vec![0.01_f32; 128];
    let mut out_buf = vec![0.0_f32; 128];

    // Warmup fora da contagem
    for _ in 0..8 {
        chain.process(&in_buf, &mut out_buf);
    }

    let (allocs, deallocs) = measure_allocs(|| {
        for _ in 0..64 {
            chain.process(&in_buf, &mut out_buf);
        }
    });

    assert_eq!(allocs, 0, "AUD-02 zero_alloc falhou: {} alocações", allocs);
    assert_eq!(deallocs, 0, "AUD-02 zero_alloc falhou: {} desalocações", deallocs);
}

#[test]
fn aud_06_sem_contencao_em_uso_normal() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};
    use ampforge_dsp::tuner::TunerOutputMode;
    use sonix_desktop::api::{dispatch, ApiContext, AppState};
    use sonix_desktop::audio::{
        DesktopAudioHost, RealtimeDspChain, RealtimeDuplexEngine, TelemetryBuffer,
    };

    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    chain.set_tuner_output_mode(TunerOutputMode::Bypass);
    let dsp = Arc::new(Mutex::new(chain));
    let duplex = Arc::new(RealtimeDuplexEngine::with_target_period(16384, 128));
    let telemetry = Arc::new(TelemetryBuffer::new(512));
    let app_state = Arc::new(Mutex::new(AppState::default()));
    let audio_telemetry = Arc::new(Mutex::new(None));

    // Passa param_queue = None propositalmente para verificar que ApiContext::new
    // extrai a fila SPSC da cadeia e nunca cai em lock/try_lock no caminho HTTP.
    let ctx = ApiContext::new(
        audio_telemetry,
        app_state,
        telemetry.clone(),
        Some(dsp.clone()),
        None,
        None,
        None,
    );

    let running = Arc::new(AtomicBool::new(true));
    let run_cb = running.clone();
    let duplex_cb = duplex.clone();
    let dsp_cb = dsp.clone();
    let tel_cb = telemetry.clone();

    let cb_thread = thread::spawn(move || {
        let mut data = vec![0.0f32; 256]; // 128 frames * 2 canais
        let mut mono_in = vec![0.0f32; 128];
        let mut mono_out = vec![0.0f32; 128];
        let mut phase = 0.0f32;
        let step = 2.0 * std::f32::consts::PI * 220.0 / 48000.0;
        let mut block_in = [0.0f32; 128];
        while run_cb.load(Ordering::Relaxed) {
            for s in block_in.iter_mut() {
                *s = phase.sin() * 0.25;
                phase += step;
            }
            duplex_cb.push_captured_samples(&block_in);
            DesktopAudioHost::process_output_callback_f32(
                &mut data,
                2,
                48000.0,
                &mut mono_in,
                &mut mono_out,
                &duplex_cb,
                &dsp_cb,
                &tel_cb,
                None,
            );
            thread::yield_now();
        }
    });

    let start = Instant::now();
    let scenes = ["A", "B", "C", "D", "E", "F", "G", "H"];
    let mut tick = 0usize;
    let mut scene_idx = 0usize;
    while start.elapsed() < Duration::from_secs(2) {
        let gain = 3.0 + ((tick % 50) as f32) * 0.1;
        let param_body = format!(
            r#"{{"amp_1.gain":{:.2},"tuner.enabled":true,"tuner.output":"bypass"}}"#,
            gain
        );
        let (c_p, _, b_p) = dispatch(&ctx, "/api/params", "POST", param_body.as_bytes());
        assert_eq!(c_p, 200, "POST /api/params falhou: {}", String::from_utf8_lossy(&b_p));

        let (c_s, _, b_s) = dispatch(&ctx, "/api/scope?mode=waveform", "GET", b"");
        assert_eq!(c_s, 200, "GET /api/scope falhou: {}", String::from_utf8_lossy(&b_s));

        let (c_t, _, b_t) = dispatch(&ctx, "/api/tuner", "GET", b"");
        assert_eq!(c_t, 200, "GET /api/tuner falhou: {}", String::from_utf8_lossy(&b_t));

        if scene_idx < 8 && tick % 15 == 0 {
            let scene_body = format!(r#"{{"scene":"{}"}}"#, scenes[scene_idx]);
            let (c_sc, _, b_sc) = dispatch(&ctx, "/api/scenes/select", "POST", scene_body.as_bytes());
            assert_eq!(c_sc, 200, "POST /api/scenes/select falhou: {}", String::from_utf8_lossy(&b_sc));
            scene_idx += 1;
        }

        tick += 1;
        thread::sleep(Duration::from_millis(16));
    }

    while scene_idx < 8 {
        let scene_body = format!(r#"{{"scene":"{}"}}"#, scenes[scene_idx]);
        let (c_sc, _, b_sc) = dispatch(&ctx, "/api/scenes/select", "POST", scene_body.as_bytes());
        assert_eq!(c_sc, 200, "POST /api/scenes/select falhou: {}", String::from_utf8_lossy(&b_sc));
        scene_idx += 1;
    }

    running.store(false, Ordering::Relaxed);
    cb_thread.join().unwrap();

    let contentions = telemetry.dsp_contention_silence_blocks();
    assert_eq!(
        contentions, 0,
        "AUD-06: detectados {} blocos com contenção de try_lock em uso normal",
        contentions
    );
}

#[test]
fn aud_06_contencao_com_mute_da_silencio() {
    use std::sync::{Arc, Mutex};
    use ampforge_dsp::tuner::TunerOutputMode;
    use sonix_desktop::audio::{
        DesktopAudioHost, RealtimeDspChain, RealtimeDuplexEngine, TelemetryBuffer,
    };

    let duplex = RealtimeDuplexEngine::with_target_period(2048, 128);
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    chain.set_tuner_enabled(true);
    chain.set_tuner_output_mode(TunerOutputMode::Mute);
    let dsp = Arc::new(Mutex::new(chain));
    let tel = Arc::new(TelemetryBuffer::new(256));

    let in_sig = vec![0.4f32; 256];
    let mut data_f32 = vec![0.0f32; 256];
    let mut data_i16 = vec![0i16; 256];
    let mut mono_in = vec![0.0f32; 128];
    let mut mono_out = vec![0.0f32; 128];

    // 1 bloco livre para sincronizar telemetria de mute
    duplex.push_captured_samples(&in_sig);
    DesktopAudioHost::process_output_callback_f32(
        &mut data_f32,
        2,
        48000.0,
        &mut mono_in,
        &mut mono_out,
        &duplex,
        &dsp,
        &tel,
        None,
    );

    // Subcaso A: contenção com afinador em Mute -> saída zero em F32 e I16
    duplex.push_captured_samples(&in_sig);
    {
        let _held = dsp.lock().unwrap();
        DesktopAudioHost::process_output_callback_f32(
            &mut data_f32,
            2,
            48000.0,
            &mut mono_in,
            &mut mono_out,
            &duplex,
            &dsp,
            &tel,
            None,
        );
        assert!(
            tel.dsp_contention_silence_blocks() >= 1,
            "AUD-06: contenção com mute deveria registrar incremento"
        );
        assert!(
            data_f32.iter().all(|&s| s == 0.0),
            "AUD-06: sinal seco vazou sob contenção com afinador em Mute (F32 max={:?})",
            data_f32.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
        );

        duplex.push_captured_samples(&in_sig);
        DesktopAudioHost::process_output_callback_i16(
            &mut data_i16,
            2,
            48000.0,
            &mut mono_in,
            &mut mono_out,
            &duplex,
            &dsp,
            &tel,
            None,
        );
        assert!(
            data_i16.iter().all(|&s| s == 0),
            "AUD-06: sinal seco vazou sob contenção com afinador em Mute (I16)"
        );
    }

    // Subcaso B: desliga afinador e ativa Standby -> saída zero sob contenção
    {
        let mut d = dsp.lock().unwrap();
        d.set_tuner_enabled(false);
        d.set_amp_standby(true);
    }
    duplex.push_captured_samples(&in_sig);
    DesktopAudioHost::process_output_callback_f32(
        &mut data_f32,
        2,
        48000.0,
        &mut mono_in,
        &mut mono_out,
        &duplex,
        &dsp,
        &tel,
        None,
    );
    duplex.push_captured_samples(&in_sig);
    {
        let _held = dsp.lock().unwrap();
        DesktopAudioHost::process_output_callback_f32(
            &mut data_f32,
            2,
            48000.0,
            &mut mono_in,
            &mut mono_out,
            &duplex,
            &dsp,
            &tel,
            None,
        );
        assert!(
            data_f32.iter().all(|&s| s == 0.0),
            "AUD-06: sinal seco vazou sob contenção com amp em Standby (max={:?})",
            data_f32.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
        );
    }
}

#[test]
fn aud_07_sem_vazamento() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use ampforge_dsp::cab::ImpulseResponse;
    use sonix_desktop::audio::{DspParamUpdate, PreparedDspResource, RealtimeDspChain};

    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    assert!(chain.param_queue().push(DspParamUpdate::CabIrEnabled(true)).is_ok());
    let rq = chain.resource_queue();
    let input = [0.1f32; 128];
    let mut output = [0.0f32; 128];

    // Pausa o worker reaper para forçar enchimento de retired_queue + pendentes + backpressure
    chain.set_reaper_paused_for_test(true);

    let drops = Arc::new(AtomicUsize::new(0));
    for _ in 0..10_000 {
        let ir = ImpulseResponse::from_samples(48000.0, vec![0.5; 256])
            .with_drop_counter(drops.clone());
        match rq.push(PreparedDspResource::CabinetIr { slot: 0, ir }) {
            Ok(()) => {}
            Err(rejected) => {
                // Sob backpressure, produtor descarta fora da thread RT em vez de vazar no callback RT.
                drop(rejected);
            }
        }
        let (allocs, deallocs) = measure_allocs(|| {
            chain.process(&input, &mut output);
        });
        assert_eq!(allocs, 0, "AUD-07: process alocou durante troca sob pressão");
        assert_eq!(deallocs, 0, "AUD-07: process desalocou durante troca sob pressão");
    }

    assert_eq!(
        chain.retired_drop_count(),
        0,
        "AUD-07: reaper deveria estar pausado enquanto acumulava aposentados"
    );
    assert!(
        chain.retired_pending_len() > 0,
        "AUD-07: pendentes deveria ter acumulado recursos com o reaper pausado"
    );

    // Retoma o worker reaper e drena pendentes + fila de entrada
    chain.set_reaper_paused_for_test(false);
    for _ in 0..100 {
        chain.process(&input, &mut output);
        thread::sleep(Duration::from_millis(25));
        if chain.retired_pending_len() == 0 {
            break;
        }
    }

    // Empurra uma IR não rastreada para liberar a última IR rastreada ainda ativa no gabinete
    let untracked = ImpulseResponse::from_samples(48000.0, vec![0.25; 256]);
    let mut pushed_untracked = false;
    let mut opt_ir = Some(PreparedDspResource::CabinetIr { slot: 0, ir: untracked });
    for _ in 0..100 {
        if let Some(res) = opt_ir.take() {
            match rq.push(res) {
                Ok(()) => {
                    pushed_untracked = true;
                    break;
                }
                Err(rejected) => {
                    opt_ir = Some(rejected);
                    chain.process(&input, &mut output);
                    thread::sleep(Duration::from_millis(25));
                }
            }
        }
    }
    assert!(pushed_untracked, "AUD-07: não conseguiu enviar IR final não rastreada após retomar reaper");

    // 24 blocos = 64 ms > crossfade de 20 ms para aposentar a última IR ativa
    for _ in 0..24 {
        chain.process(&input, &mut output);
    }

    let mut dropped = drops.load(Ordering::Acquire);
    for _ in 0..75 {
        if dropped == 10_000 {
            break;
        }
        chain.process(&input, &mut output);
        thread::sleep(Duration::from_millis(20));
        dropped = drops.load(Ordering::Acquire);
    }

    assert_eq!(
        dropped, 10_000,
        "AUD-07: vazamento de recursos aposentados (dropped={} de 10000, pending={})",
        dropped,
        chain.retired_pending_len()
    );
}

#[test]
fn aud_07_zero_alloc() {
    use ampforge_dsp::cab::ImpulseResponse;
    use sonix_desktop::audio::{DspParamUpdate, PreparedDspResource, RealtimeDspChain};

    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    assert!(chain.param_queue().push(DspParamUpdate::CabIrEnabled(true)).is_ok());
    let rq = chain.resource_queue();
    let input = [0.2f32; 128];
    let mut output = [0.0f32; 128];
    chain.process(&input, &mut output);

    // Fase 1: trocas normais com reaper ativo
    let (allocs_norm, deallocs_norm) = measure_allocs(|| {
        for _ in 0..32 {
            // Alocação proposital do produtor ficaria fora em caso real; aqui medimos apenas chain.process
            chain.process(&input, &mut output);
        }
    });
    assert_eq!(allocs_norm, 0);
    assert_eq!(deallocs_norm, 0);

    for _ in 0..12 {
        let ir = ImpulseResponse::from_samples(48000.0, vec![0.4; 256]);
        let _ = rq.push(PreparedDspResource::CabinetIr { slot: 0, ir });
        let (a, d) = measure_allocs(|| {
            for _ in 0..24 {
                chain.process(&input, &mut output);
            }
        });
        assert_eq!(a, 0, "AUD-07: chain.process alocou na troca normal");
        assert_eq!(d, 0, "AUD-07: chain.process desalocou na troca normal");
    }

    // Fase 2: saturação (reaper pausado -> pendentes + backpressure)
    chain.set_reaper_paused_for_test(true);
    for _ in 0..300 {
        let ir = ImpulseResponse::from_samples(48000.0, vec![0.4; 256]);
        let _ = rq.push(PreparedDspResource::CabinetIr { slot: 0, ir });
        let (a, d) = measure_allocs(|| {
            chain.process(&input, &mut output);
        });
        assert_eq!(a, 0, "AUD-07: chain.process alocou sob saturação");
        assert_eq!(d, 0, "AUD-07: chain.process desalocou sob saturação");
    }
    chain.set_reaper_paused_for_test(false);
}

#[test]
fn aud_07_sem_unpark_em_audio_rs() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let audio_rs_path = std::path::Path::new(manifest_dir).join("src").join("audio.rs");
    let cab_rs_path = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("ampforge-dsp")
        .join("src")
        .join("cab.rs");

    let audio_src = std::fs::read_to_string(&audio_rs_path).expect("read audio.rs");
    let audio_non_test = audio_src.split("#[cfg(test)]").next().unwrap_or(&audio_src);
    assert!(
        !audio_non_test.contains("unpark("),
        "AUD-07: encontrado 'unpark(' em audio.rs (proibido no thread RT)"
    );
    assert!(
        !audio_non_test.contains("mem::forget("),
        "AUD-07: encontrado 'mem::forget(' em audio.rs (proibido vazar recursos)"
    );

    let cab_src = std::fs::read_to_string(&cab_rs_path).expect("read cab.rs");
    let cab_non_test = cab_src.split("#[cfg(test)]").next().unwrap_or(&cab_src);
    assert!(
        !cab_non_test.contains("mem::forget("),
        "AUD-07: encontrado 'mem::forget(' em cab.rs (proibido vazar IR aposentada)"
    );
}
