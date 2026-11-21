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
