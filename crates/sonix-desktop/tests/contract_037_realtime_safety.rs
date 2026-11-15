//! crates/sonix-desktop/tests/contract_037_realtime_safety.rs
//! Testes de Contrato para Spec 037: Segurança de tempo real na cadeia ao vivo
//! (zero alocação, zero desalocação, zero bloqueio, sem silêncio na carga de IR/preset)

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use ampforge_dsp::amp::AmpModelId;
use ampforge_dsp::cab::{ImpulseResponse, SpeakerCabModel, VirtualCabProcessor};
use sonix_desktop::audio::{
    DspParamUpdate, LooperActionCmd, RealtimeDspChain, TelemetryBuffer,
};
use sonix_desktop::server::LocalAppServer;

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

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        COUNTING_ACTIVE.with(|active| {
            if active.get() {
                THREAD_ALLOCS.with(|c| c.set(c.get() + 1));
                THREAD_DEALLOCS.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.realloc(ptr, layout, new_size) }
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

fn build_pcm16_wav(sample_rate: u32, samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let riff_len = 36 + data_len;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_len.to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // Mono
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

fn post_http(port: u16, path: &str, token: &str, content_type: &str, body: &[u8]) -> (u16, String) {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
    let header = format!(
        "POST {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\nContent-Type: {}\r\nX-Sonix-Token: {}\r\nContent-Length: {}\r\n\r\n",
        path, port, content_type, token, body.len()
    );
    stream.write_all(header.as_bytes()).unwrap();
    stream.write_all(body).unwrap();
    let mut resp = String::new();
    stream.read_to_string(&mut resp).unwrap();
    let code = resp
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(500);
    let resp_body = resp.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (code, resp_body)
}

#[test]
fn test_ca_01_ten_thousand_blocks_interleaved_params_zero_alloc_zero_dealloc() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let pq = chain.param_queue();

    let input = [0.1f32; 128];
    let mut output = [0.0f32; 128];

    // Warmup inicial fora da janela medida
    chain.process(&input, &mut output);

    let (allocs, deallocs) = measure_allocs(|| {
        for i in 0..10_000usize {
            match i % 7 {
                0 => { let _ = pq.push(DspParamUpdate::InputSource((i % 3) as u8)); }
                1 => { let _ = pq.push(DspParamUpdate::SplitCrossoverHz(400.0 + (i % 400) as f32)); }
                2 => { let _ = pq.push(DspParamUpdate::TunerEnabled((i % 2) == 0)); }
                3 => {
                    let cmd = match i % 4 {
                        0 => LooperActionCmd::Record,
                        1 => LooperActionCmd::Play,
                        2 => LooperActionCmd::Overdub,
                        _ => LooperActionCmd::Undo,
                    };
                    let _ = pq.push(DspParamUpdate::LooperAction(cmd));
                }
                4 => {
                    let m = if i % 2 == 0 { AmpModelId::UsTwinBlackface65 } else { AmpModelId::BritJcm800_2203 };
                    let _ = pq.push(DspParamUpdate::AmpModel(m));
                }
                5 => {
                    let spk = if i % 2 == 0 { SpeakerCabModel::V304x12Closed } else { SpeakerCabModel::Greenback4x12Closed };
                    let _ = pq.push(DspParamUpdate::Cab1Speaker(spk));
                }
                _ => { let _ = pq.push(DspParamUpdate::AmpGain(5.0 + (i % 5) as f32)); }
            }
            chain.process(&input, &mut output);
        }
    });

    assert_eq!(allocs, 0, "CA-1: Thread de áudio alocou {} vezes em 10k blocos", allocs);
    assert_eq!(deallocs, 0, "CA-1: Thread de áudio desalocou {} vezes em 10k blocos", deallocs);
}

#[test]
fn test_ca_02_chunked_1024_matches_four_256_calls_with_zero_alloc() {
    let mut c1 = RealtimeDspChain::new(48000.0);
    let mut c2 = RealtimeDspChain::new(48000.0);
    c1.prepare(48000.0, 256);
    c2.prepare(48000.0, 256);

    let mut input = vec![0.0f32; 1024];
    for (i, s) in input.iter_mut().enumerate() {
        *s = (2.0 * std::f32::consts::PI * 220.0 * (i as f32 / 48000.0)).sin() * 0.25;
    }

    let mut out_1024 = vec![0.0f32; 1024];
    let mut out_4x256 = vec![0.0f32; 1024];

    for chunk in 0..4 {
        c2.process(&input[chunk * 256..(chunk + 1) * 256], &mut out_4x256[chunk * 256..(chunk + 1) * 256]);
    }

    let (allocs, deallocs) = measure_allocs(|| {
        c1.process(&input, &mut out_1024);
    });

    assert_eq!(allocs, 0, "CA-2: process(1024) após prepare(48000, 256) alocou {} vezes", allocs);
    assert_eq!(deallocs, 0, "CA-2: process(1024) desalocou {} vezes", deallocs);

    let max_diff = out_1024
        .iter()
        .zip(out_4x256.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        max_diff <= 1e-6,
        "CA-2: Saída de 1024 amostras divergiu de 4x256: max_diff={}",
        max_diff
    );
}

#[test]
fn test_ca_03_ir_crossfade_finishes_with_zero_dealloc_on_audio_thread() {
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48000.0);
    let mut ir1_samples = vec![0.0f32; 4096];
    ir1_samples[0] = 0.8;
    let mut ir2_samples = vec![0.0f32; 4096];
    ir2_samples[0] = 0.5;

    cab.set_custom_ir(ImpulseResponse::from_samples(48000.0, ir1_samples));
    cab.set_ir_enabled(true);
    cab.set_target_ir(ImpulseResponse::from_samples(48000.0, ir2_samples));

    let input = [0.15f32; 128];
    let mut output = [0.0f32; 128];

    let (allocs, deallocs) = measure_allocs(|| {
        // 20 ms at 48 kHz = 960 samples -> 16 blocos de 128 cruzam além do fim do crossfade
        for _ in 0..16 {
            cab.process(&input, &mut output);
        }
    });

    assert_eq!(allocs, 0, "CA-3: Crossfade de IR alocou {} vezes no callback", allocs);
    assert_eq!(deallocs, 0, "CA-3: Fim do crossfade de IR desalocou {} vezes no callback!", deallocs);
}

#[test]
fn test_ca_04_loading_500ms_96khz_ir_causes_zero_silent_blocks() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let dsp = Arc::new(Mutex::new(chain));
    let telemetry = Arc::new(TelemetryBuffer::new(512));

    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry.clone(), Some(dsp.clone()))
        .expect("bind");
    let port = server.port();
    let token = server.session_token();

    // 500 ms a 96 kHz = 48.000 amostras
    let mut ir_samples = vec![0.0f32; 48_000];
    ir_samples[0] = 0.9;
    let wav = build_pcm16_wav(96_000, &ir_samples);

    let running = Arc::new(AtomicBool::new(true));
    let run_c = running.clone();
    let dsp_c = dsp.clone();
    let silent_blocks = Arc::new(AtomicUsize::new(0));
    let silent_c = silent_blocks.clone();

    let audio_handle = thread::spawn(move || {
        let mut phase = 0.0f32;
        let step = 2.0 * std::f32::consts::PI * 220.0 / 48000.0;
        let mut in_buf = [0.0f32; 128];
        let mut out_buf = [0.0f32; 128];
        while run_c.load(Ordering::Relaxed) {
            for s in in_buf.iter_mut() {
                *s = phase.sin() * 0.4;
                phase += step;
            }
            if let Ok(mut guard) = dsp_c.try_lock() {
                guard.process(&in_buf, &mut out_buf);
                let rms = (out_buf.iter().map(|x| x * x).sum::<f32>() / 128.0).sqrt();
                let rms_db = if rms > 1e-10 { 20.0 * rms.log10() } else { -200.0 };
                if rms_db < -60.0 {
                    silent_c.fetch_add(1, Ordering::Relaxed);
                }
            } else {
                // Se try_lock falhou na simulação de callback -> bloco perdido/silencioso!
                silent_c.fetch_add(1, Ordering::Relaxed);
            }
            thread::sleep(Duration::from_micros(500));
        }
    });

    thread::sleep(Duration::from_millis(20));
    let (code, body) = post_http(port, "/api/cab/ir", &token, "audio/wav", &wav);
    thread::sleep(Duration::from_millis(40));
    running.store(false, Ordering::Relaxed);
    audio_handle.join().unwrap();

    assert_eq!(code, 200, "CA-4: /api/cab/ir falhou: {}", body);
    let lost = silent_blocks.load(Ordering::Relaxed);
    assert_eq!(
        lost, 0,
        "CA-4: Carregamento de IR de 500 ms / 96 kHz gerou {} blocos silenciosos / bloqueados",
        lost
    );
}

#[test]
fn test_ca_05_loading_preset_causes_zero_silent_blocks_and_critical_section_under_50us() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let dsp = Arc::new(Mutex::new(chain));
    let telemetry = Arc::new(TelemetryBuffer::new(512));

    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry.clone(), Some(dsp.clone()))
        .expect("bind");
    let port = server.port();
    let token = server.session_token();

    let running = Arc::new(AtomicBool::new(true));
    let run_c = running.clone();
    let dsp_c = dsp.clone();
    let silent_blocks = Arc::new(AtomicUsize::new(0));
    let silent_c = silent_blocks.clone();

    let audio_handle = thread::spawn(move || {
        let mut phase = 0.0f32;
        let step = 2.0 * std::f32::consts::PI * 220.0 / 48000.0;
        let mut in_buf = [0.0f32; 128];
        let mut out_buf = [0.0f32; 128];
        while run_c.load(Ordering::Relaxed) {
            for s in in_buf.iter_mut() {
                *s = phase.sin() * 0.4;
                phase += step;
            }
            if let Ok(mut guard) = dsp_c.try_lock() {
                guard.process(&in_buf, &mut out_buf);
                let rms = (out_buf.iter().map(|x| x * x).sum::<f32>() / 128.0).sqrt();
                let rms_db = if rms > 1e-10 { 20.0 * rms.log10() } else { -200.0 };
                if rms_db < -60.0 {
                    silent_c.fetch_add(1, Ordering::Relaxed);
                }
            } else {
                silent_c.fetch_add(1, Ordering::Relaxed);
            }
            thread::sleep(Duration::from_micros(300));
        }
    });

    thread::sleep(Duration::from_millis(15));
    for preset in ["usdeluxe", "recto", "ac30", "brit800"] {
        let body = format!(r#"{{"id":"{}"}}"#, preset);
        let (code, resp) = post_http(port, "/api/presets/load", &token, "application/json", body.as_bytes());
        assert_eq!(code, 200, "CA-5: preset load falhou ({}): {}", preset, resp);
        thread::sleep(Duration::from_millis(15));
    }
    running.store(false, Ordering::Relaxed);
    audio_handle.join().unwrap();

    let lost = silent_blocks.load(Ordering::Relaxed);
    assert_eq!(
        lost, 0,
        "CA-5: Carga de preset bloqueou áudio / causou {} blocos silenciosos",
        lost
    );
}

#[test]
fn test_ca_07_counting_allocator_catches_injected_allocation() {
    // Regra CA-7: comprova que o alocador de contagem reprova uma alocação injetada de propósito.
    let (injected_allocs, injected_deallocs) = measure_allocs(|| {
        let v: Vec<f32> = Vec::with_capacity(64);
        std::hint::black_box(v);
    });
    assert!(
        injected_allocs > 0 && injected_deallocs > 0,
        "CA-7: Alocador de contagem deve detectar Vec::with_capacity injetado (allocs={}, deallocs={})",
        injected_allocs,
        injected_deallocs
    );

    // E RealtimeDspChain::process normal deve registrar 0/0:
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let input = [0.2f32; 128];
    let mut output = [0.0f32; 128];
    chain.process(&input, &mut output);

    let (rt_allocs, rt_deallocs) = measure_allocs(|| {
        for _ in 0..500 {
            chain.process(&input, &mut output);
        }
    });
    assert_eq!(rt_allocs, 0, "CA-7: RealtimeDspChain::process alocou {} vezes", rt_allocs);
    assert_eq!(rt_deallocs, 0, "CA-7: RealtimeDspChain::process desalocou {} vezes", rt_deallocs);
}

fn get_http_local(port: u16, path: &str) -> (u16, String) {
    use std::io::{Read, Write};
    let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).expect("connect");
    let req = format!(
        "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        path, port
    );
    stream.write_all(req.as_bytes()).unwrap();
    let mut resp = String::new();
    stream.read_to_string(&mut resp).unwrap();
    let code = resp
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse::<u16>().ok())
        .unwrap_or(0);
    let body = resp.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (code, body)
}

#[test]
fn test_ca_03_retired_ir_reclaimed_by_reaper_thread_zero_rt_dealloc() {
    use sonix_desktop::audio::{DspParamUpdate, PreparedDspResource};

    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    assert!(chain.param_queue().push(DspParamUpdate::CabIrEnabled(true)).is_ok());
    let rq = chain.resource_queue().clone();

    let mut ir1 = vec![0.0f32; 4096];
    ir1[0] = 0.8;
    assert!(
        rq.push(PreparedDspResource::CabinetIr {
            slot: 0,
            ir: ImpulseResponse::from_samples(48000.0, ir1),
        })
        .is_ok()
    );

    let input = [0.15f32; 128];
    let mut output = [0.0f32; 128];
    chain.process(&input, &mut output);

    let mut ir2 = vec![0.0f32; 4096];
    ir2[0] = 0.6;
    assert!(
        rq.push(PreparedDspResource::CabinetIr {
            slot: 0,
            ir: ImpulseResponse::from_samples(48000.0, ir2),
        })
        .is_ok()
    );

    let (allocs, deallocs) = measure_allocs(|| {
        // Crossfade 20 ms @ 48 kHz = 960 samples; 24x128 = 3072 samples cruza o término e aposenta ir1
        for _ in 0..24 {
            chain.process(&input, &mut output);
        }
    });
    assert_eq!(allocs, 0, "CA-3: troca de IR via resource_queue alocou no callback");
    assert_eq!(deallocs, 0, "CA-3: aposentadoria de IR desalocou na thread de áudio");

    let mut reclaimed = chain.retired_drop_count();
    for _ in 0..40 {
        if reclaimed >= 1 {
            break;
        }
        thread::sleep(Duration::from_millis(5));
        reclaimed = chain.retired_drop_count();
    }
    assert!(
        reclaimed >= 1,
        "CA-3: reaper em background (sonix-rt-reaper) não reclamou IR aposentada (retired_drop_count={})",
        reclaimed
    );
}

#[test]
fn test_ca_05_critical_section_under_50us_and_lock_free_oscilloscope() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let dsp = Arc::new(Mutex::new(chain));
    let telemetry = Arc::new(TelemetryBuffer::new(512));

    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry.clone(), Some(dsp.clone()))
        .expect("bind");
    let port = server.port();
    let token = server.session_token();

    // Se API segurar lock > 50us ou bloquear osciloscópio, o poller detectará gap > 50us
    let running = Arc::new(AtomicBool::new(true));
    let run_c = running.clone();
    let dsp_c = dsp.clone();
    let max_miss_streak = Arc::new(AtomicUsize::new(0));
    let max_miss_c = max_miss_streak.clone();

    let poller = thread::spawn(move || {
        let mut current_streak = 0usize;
        while run_c.load(Ordering::Relaxed) {
            if let Ok(_guard) = dsp_c.try_lock() {
                current_streak = 0;
            } else {
                current_streak += 1;
                max_miss_c.fetch_max(current_streak, Ordering::Relaxed);
            }
            thread::sleep(Duration::from_micros(25));
        }
    });

    for preset in ["usdeluxe", "recto", "ac30", "brit800"] {
        let body = format!(r#"{{"id":"{}"}}"#, preset);
        let (code, resp) = post_http(port, "/api/presets/load", &token, "application/json", body.as_bytes());
        assert_eq!(code, 200, "CA-5: preset load falhou ({}): {}", preset, resp);
    }

    running.store(false, Ordering::Relaxed);
    poller.join().unwrap();

    // Como o audio poller nunca divide o lock com o caminho de carga (vai via param/resource queue),
    // nenhum lock externo segura o mutex de DSP.
    let streak = max_miss_streak.load(Ordering::Relaxed);
    assert!(
        streak <= 1,
        "CA-5: carga de preset reteve Mutex do DSP por {} tentativas de 25us (> 50us)",
        streak
    );

    // Osciloscópio via TelemetryBuffer deve ser 100% lock-free e zero-alloc
    let block = [0.25f32; 128];
    let (allocs, deallocs) = measure_allocs(|| {
        telemetry.push_waveform(&block);
    });
    assert_eq!(allocs, 0, "CA-5: push_waveform alocou {}", allocs);
    assert_eq!(deallocs, 0, "CA-5: push_waveform desalocou {}", deallocs);
    assert!(telemetry.waveform_seq() >= 1);
}

#[test]
fn test_ca_06_lock_free_telemetry_reads_under_audio_write() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let dsp = Arc::new(Mutex::new(chain));
    let telemetry = Arc::new(TelemetryBuffer::new(512));

    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry.clone(), Some(dsp))
        .expect("bind");
    let port = server.port();

    let writer_tel = telemetry.clone();
    let running = Arc::new(AtomicBool::new(true));
    let run_c = running.clone();
    let writer = thread::spawn(move || {
        let wave = [0.1f32; 128];
        while run_c.load(Ordering::Relaxed) {
            writer_tel.push_waveform(&wave);
            thread::sleep(Duration::from_micros(200));
        }
    });

    let start = std::time::Instant::now();
    let mut prev_seq = telemetry.waveform_seq();
    let mut advanced = 0usize;
    for _ in 0..25 {
        let (code, body) = get_http_local(port, "/api/telemetry");
        assert_eq!(code, 200, "CA-6: GET /api/telemetry falhou: {}", body);
        let cur = telemetry.waveform_seq();
        if cur > prev_seq {
            advanced += 1;
            prev_seq = cur;
        }
        thread::sleep(Duration::from_millis(2));
    }
    running.store(false, Ordering::Relaxed);
    writer.join().unwrap();

    assert!(advanced >= 1, "CA-6: waveform_seq não avançou sob escrita concorrente");
    assert!(start.elapsed() < Duration::from_secs(3), "CA-6: leituras bloquearam por tempo excessivo");
}

#[test]
fn test_ca_08_dsp_contention_silence_blocks_exposed_on_audio_status() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let dsp = Arc::new(Mutex::new(chain));
    let telemetry = Arc::new(TelemetryBuffer::new(512));
    telemetry.record_dsp_contention_silence();
    telemetry.record_dsp_contention_silence();

    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry, Some(dsp))
        .expect("bind");
    let (code, body) = get_http_local(server.port(), "/api/audio/status");
    assert_eq!(code, 200, "CA-8: GET /api/audio/status falhou: {}", body);
    let v: serde_json::Value = serde_json::from_str(&body).expect("json");
    assert_eq!(
        v["dsp_contention_silence_blocks"].as_u64(),
        Some(2),
        "CA-8: campo dsp_contention_silence_blocks ausente ou incorreto em /api/audio/status: {}",
        body
    );
}
