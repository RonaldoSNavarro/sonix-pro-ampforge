use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

use sonix_desktop::api::{
    compile_modifiers_table, dispatch, ApiContext, AppState, CalibrationSession,
};
use sonix_desktop::audio::{
    AudioDeviceEnumerator, DesktopAudioDevice, RealtimeDspChain,
    RealtimeDuplexEngine, TelemetryBuffer,
};
use sonix_desktop::midi::{DesktopMidiHost, MidiRouter};

struct CountingAllocator;
thread_local! {
    static TRACK_ALLOC: Cell<bool> = const { Cell::new(false) };
    static ALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
    static DEALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        TRACK_ALLOC.with(|t| {
            if t.get() {
                ALLOC_COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        TRACK_ALLOC.with(|t| {
            if t.get() {
                DEALLOC_COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.dealloc(ptr, layout) }
    }
}

struct DummyAudioEnum;
impl AudioDeviceEnumerator for DummyAudioEnum {
    fn enumerate_devices(&self) -> Vec<DesktopAudioDevice> {
        vec![]
    }
}

fn make_runtime_ctx() -> (ApiContext, Arc<Mutex<RealtimeDspChain>>) {
    let mut chain = RealtimeDspChain::new(48_000.0);
    chain.prepare(48_000.0, 256);
    let mut st = AppState::default();
    st.amp_standby = false;
    chain.set_amp_standby(false);
    chain.set_compiled_modifiers(compile_modifiers_table(&st.modifiers));

    let dsp = Arc::new(Mutex::new(chain));
    let (param_q, res_q, sr_bits, algo_lat, pitch_wet_bits, b_plus_bits) = {
        let c = dsp.lock().unwrap();
        (
            c.param_queue(),
            c.resource_queue(),
            c.shared_sample_rate_bits(),
            c.shared_algo_latency_samples(),
            c.shared_pitch_wet_latency_ms_bits(),
            c.shared_b_plus_v_bits(),
        )
    };
    let ctx = ApiContext {
        audio_telemetry: Arc::new(Mutex::new(None)),
        app_state: Arc::new(Mutex::new(st)),
        telemetry: Arc::new(TelemetryBuffer::new(128)),
        dsp_chain: Some(dsp.clone()),
        param_queue: Some(param_q),
        resource_queue: Some(res_q),
        shared_adc_clips: None,
        shared_sample_rate_bits: Some(sr_bits),
        shared_algo_latency_samples: Some(algo_lat),
        shared_pitch_wet_latency_ms_bits: Some(pitch_wet_bits),
        shared_b_plus_v_bits: Some(b_plus_bits),
        clean_input_ring: None,
        scope_output_ring: None,
        calibration_store: None,
        active_device_id: Arc::new(Mutex::new(None)),
        calibration_session: Arc::new(Mutex::new(CalibrationSession::default())),
        device_enumerator: Arc::new(DummyAudioEnum),
        duplex_engine: Some(Arc::new(RealtimeDuplexEngine::new(4096))),
        custom_presets_dir: None,
        data_dir: None,
    };
    (ctx, dsp)
}

fn rms(buf: &[f32]) -> f32 {
    (buf.iter().map(|&x| x * x).sum::<f32>() / buf.len().max(1) as f32).sqrt()
}

#[test]
fn aud_17_porta_inexistente_falha() {
    let mut host = DesktopMidiHost::new();
    let res = host.connect_port("porta_que_nao_existe_12345");
    assert!(
        res.is_err(),
        "AUD-17: connect_port em porta inexistente deve falhar"
    );
    let err = res.unwrap_err();
    assert!(
        err.contains("porta MIDI não encontrada"),
        "AUD-17: mensagem esperada 'porta MIDI não encontrada: ...', obtido: {err}"
    );
}

#[test]
fn aud_17_cc_chega_ao_motor() {
    let (ctx, dsp) = make_runtime_ctx();
    let mut router = MidiRouter::new(ctx);

    let input: Vec<f32> = (0..256)
        .map(|i| 0.25 * (2.0 * PI * 220.0 * (i as f32) / 48_000.0).sin())
        .collect();
    let mut out_low = [0.0_f32; 256];
    let mut out_high = [0.0_f32; 256];

    router.on_message(&[0xB0, 1, 0]);
    {
        let mut c = dsp.lock().unwrap();
        for _ in 0..8 {
            c.process(&input, &mut out_low);
        }
    }

    router.on_message(&[0xB0, 1, 127]);
    {
        let mut c = dsp.lock().unwrap();
        for _ in 0..8 {
            c.process(&input, &mut out_high);
        }
    }

    let r_low = rms(&out_low).max(1e-9);
    let r_high = rms(&out_high).max(1e-9);
    let diff_db = (20.0 * (r_high / r_low).log10()).abs();
    assert!(
        diff_db >= 3.0,
        "AUD-17: CC 1 (0 vs 127) deve mudar saída RMS em >= 3 dB, obtido {diff_db:.2} dB (r_low={r_low}, r_high={r_high})"
    );
}

#[test]
fn aud_17_program_change_carrega_preset() {
    let (mut ctx, _dsp) = make_runtime_ctx();
    let dir = std::env::temp_dir().join("sonix_aud_17_pc_test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    ctx.custom_presets_dir = Some(dir.clone());

    let (_, _, list_body) = dispatch(&ctx, "/api/presets", "GET", b"");
    let list_json: serde_json::Value = serde_json::from_slice(&list_body).unwrap();
    let presets = list_json["presets"].as_array().unwrap();
    let p0 = presets[0]["id"].as_str().unwrap().to_string();
    let p1 = presets[1]["id"].as_str().unwrap().to_string();

    let sl_body = serde_json::json!({
        "id": "sl_aud_17",
        "name": "AUD 17 Setlist",
        "presets": [p0, p1]
    });
    let (sc, _, sb) = dispatch(&ctx, "/api/setlist", "POST", sl_body.to_string().as_bytes());
    assert_eq!(sc, 200, "falha ao criar setlist: {}", String::from_utf8_lossy(&sb));

    let mut router = MidiRouter::new(ctx.clone());
    router.on_message(&[0xC0, 1]);

    let st = ctx.app_state.lock().unwrap();
    assert_eq!(
        st.active_preset_id, p1,
        "AUD-17: Program Change 1 deve carregar o 2º preset do setlist ativo"
    );
    drop(st);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn aud_17_cc69_seleciona_cena() {
    let (ctx, _dsp) = make_runtime_ctx();
    let mut router = MidiRouter::new(ctx.clone());

    router.on_message(&[0xB0, 69, 3]);
    let scene = ctx.app_state.lock().unwrap().scene_active.clone();
    assert_eq!(scene, "D", "AUD-17: CC 69 valor 3 deve selecionar cena D");
}

#[test]
fn aud_17_clock_120bpm() {
    let (ctx, dsp) = make_runtime_ctx();
    let mut router = MidiRouter::new(ctx.clone());

    // 25 ticks (24 intervalos) espaçados de 20.833 ms = 20833 us -> 120.0 BPM
    let mut t_us = 1_000_000_u64;
    for _ in 0..25 {
        router.on_message_at(&[0xF8], t_us);
        t_us += 20_833;
    }

    let mut c = dsp.lock().unwrap();
    let mut out = [0.0_f32; 64];
    c.process(&[0.0_f32; 64], &mut out);
    let bpm = c.dly_1_tempo_bpm();
    assert!(
        (bpm - 120.0).abs() <= 0.5,
        "AUD-17: MIDI Clock de 20.833 ms deve calcular 120 ± 0.5 BPM, obtido {bpm:.3}"
    );
}

#[test]
fn aud_17_zero_alloc() {
    let (ctx, dsp) = make_runtime_ctx();
    let mut router = MidiRouter::new(ctx);
    router.on_message(&[0xB0, 1, 90]);

    let inp = [0.1_f32; 64];
    let mut out = [0.0_f32; 64];
    {
        let mut c = dsp.lock().unwrap();
        for _ in 0..8 {
            c.process(&inp, &mut out);
        }
        ALLOC_COUNT.with(|a| a.set(0));
        DEALLOC_COUNT.with(|d| d.set(0));
        TRACK_ALLOC.with(|t| t.set(true));
        for _ in 0..32 {
            c.process(&inp, &mut out);
        }
        TRACK_ALLOC.with(|t| t.set(false));
    }
    assert_eq!(ALLOC_COUNT.with(|a| a.get()), 0);
    assert_eq!(DEALLOC_COUNT.with(|d| d.get()), 0);
}
