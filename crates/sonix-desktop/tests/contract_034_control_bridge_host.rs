//! crates/sonix-desktop/tests/contract_034_control_bridge_host.rs
//! Testes de Contrato para Spec 034: Ponte de Controle e Host de Áudio em Tempo Real

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use ampforge_dsp::amp::AmpModelId;
use ampforge_dsp::fx::delay::{DelayModel, DelayProcessor};
use sonix_desktop::audio::{RealtimeDspChain, TelemetryBuffer};
use sonix_desktop::server::LocalAppServer;

fn http_request(
    port: u16,
    method: &str,
    path: &str,
    token: &str,
    body: Option<&str>,
) -> (u16, String) {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
    let mut req = format!(
        "{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\nX-Sonix-Token: {}\r\nContent-Type: application/json\r\n",
        method, path, port, token
    );
    if let Some(b) = body {
        req.push_str(&format!("Content-Length: {}\r\n\r\n{}", b.len(), b));
    } else {
        req.push_str("\r\n");
    }
    stream.write_all(req.as_bytes()).unwrap();
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
fn test_ca_01_param_under_lock_contention_is_queued_and_never_dropped() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let dsp = Arc::new(Mutex::new(chain));
    let telemetry = Arc::new(TelemetryBuffer::new(512));

    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry, Some(dsp.clone()))
        .expect("bind");
    let port = server.port();
    let token = server.session_token();

    // Segura o lock por 15 ms enquanto envia POST /api/params amp_1.model="twin_ab763"
    let guard = dsp.lock().unwrap();
    let (code, body) = http_request(
        port,
        "POST",
        "/api/params",
        &token,
        Some(r#"{"amp_1.model":"twin_ab763"}"#),
    );
    assert_eq!(code, 200, "CA-1: POST falhou: {}", body);
    thread::sleep(Duration::from_millis(10));
    drop(guard);

    // Drena e processa 25 ms de áudio (para completar o crossfade de 20 ms do modelo)
    let in_buf = [0.01f32; 128];
    let mut out_buf = [0.0f32; 128];
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(45) {
        let mut d = dsp.lock().unwrap();
        d.process(&in_buf, &mut out_buf);
        if d.amp().model_id == AmpModelId::UsTwinBlackface65 {
            break;
        }
    }

    let final_model = dsp.lock().unwrap().amp().model_id;
    assert_eq!(
        final_model,
        AmpModelId::UsTwinBlackface65,
        "CA-1: Parâmetro amp_1.model foi descartado sob contenção!"
    );
}

#[test]
fn test_ca_02_looper_undo_with_other_param_executes_exactly_once() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let dsp = Arc::new(Mutex::new(chain));
    let telemetry = Arc::new(TelemetryBuffer::new(512));

    // Grava base (record -> play) e overdub (overdub) deixando 1 camada prévia + overdub em andamento
    {
        let mut d = dsp.lock().unwrap();
        let sig = [0.2f32; 128];
        let mut out = [0.0f32; 128];
        d.trigger_looper_action("record");
        d.process(&sig, &mut out);
        d.trigger_looper_action("play");
        d.process(&sig, &mut out);
        d.trigger_looper_action("overdub");
        d.process(&sig, &mut out);
    }

    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry, Some(dsp.clone()))
        .expect("bind");
    let port = server.port();
    let token = server.session_token();

    // Envia amp_1.bright (que antes ia fora da fila e reaplicava tudo sob try_lock!) + looper.action=undo
    let (code, body) = http_request(
        port,
        "POST",
        "/api/params",
        &token,
        Some(r#"{"amp_1.bright":true,"looper.action":"undo"}"#),
    );
    assert_eq!(code, 200, "CA-2: POST falhou: {}", body);

    let drained = dsp.lock().unwrap().drain_param_queue();
    assert!(drained >= 2, "CA-2: Esperado enfileiramento de ambos parâmetros (drained={})", drained);

    // Em Overdub com can_undo=true: 1 único undo leva a Play preservando can_undo=true.
    // Dupla execução faria Play -> can_undo=false!
    let mut d = dsp.lock().unwrap();
    let in_buf = [0.0f32; 128];
    let mut out_buf = [0.0f32; 128];
    d.process(&in_buf, &mut out_buf);
    assert_eq!(
        d.looper().state(),
        ampforge_dsp::looper::LooperState::Play,
        "CA-2: após 1 undo em Overdub, looper deveria estar em Play"
    );
    assert!(
        d.looper().can_undo(),
        "CA-2: undo duplicado teria limpado can_undo (esperado true após 1º undo)"
    );
    drop(d);

}

#[test]
fn test_ca_07_two_threads_ten_thousand_params_each_zero_loss() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let pq = chain.param_queue();

    let total_consumed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let done_producing = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let done_c = done_producing.clone();
    let tot_c = total_consumed.clone();
    let pq_consumer = pq.clone();

    let consumer = thread::spawn(move || {
        while !done_c.load(Ordering::Acquire) {
            while let Some(_) = pq_consumer.pop() {
                tot_c.fetch_add(1, Ordering::Relaxed);
            }
            thread::yield_now();
        }
        while let Some(_) = pq_consumer.pop() {
            tot_c.fetch_add(1, Ordering::Relaxed);
        }
    });

    let pq1 = pq.clone();
    let pq2 = pq.clone();
    use std::sync::atomic::Ordering;

    let t1 = thread::spawn(move || {
        let mut pushed = 0;
        while pushed < 10_000 {
            if pq1.push(sonix_desktop::audio::DspParamUpdate::AmpGain((pushed % 10) as f32)).is_ok() {
                pushed += 1;
            } else {
                thread::yield_now();
            }
        }
    });
    let t2 = thread::spawn(move || {
        let mut pushed = 0;
        while pushed < 10_000 {
            if pq2.push(sonix_desktop::audio::DspParamUpdate::AmpBass((pushed % 10) as f32)).is_ok() {
                pushed += 1;
            } else {
                thread::yield_now();
            }
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();
    done_producing.store(true, Ordering::Release);
    consumer.join().unwrap();

    assert_eq!(
        total_consumed.load(Ordering::Relaxed),
        20_000,
        "CA-7: Esperado recebimento exato de 20.000 updates entre 2 produtores concorrentes"
    );
}

#[test]
fn test_ca_08_initial_delay_mode_matches_delay_processor() {
    let telemetry = Arc::new(TelemetryBuffer::new(512));
    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry, None).expect("bind");
    let port = server.port();
    let token = server.session_token();

    let (code, body) = http_request(port, "GET", "/api/state", &token, None);
    assert_eq!(code, 200);
    let val: serde_json::Value = serde_json::from_str(&body).unwrap();
    let state_mode = val["dly_1.mode"].as_str().unwrap_or("");
    let proc_mode = DelayProcessor::new(DelayModel::MultiHeadTapeEcho).re201_mode().to_string();

    assert_eq!(
        state_mode, proc_mode,
        "CA-8: dly_1.mode inicial ({}) diverge de DelayProcessor::re201_mode() ({})",
        state_mode, proc_mode
    );
}

#[test]
fn test_ca_10_full_queue_returns_503_without_mutating_state() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let pq = chain.param_queue();
    let dsp = Arc::new(Mutex::new(chain));
    let telemetry = Arc::new(TelemetryBuffer::new(512));

    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry, Some(dsp))
        .expect("bind");
    let port = server.port();
    let token = server.session_token();

    // Satura a fila inteira sem drenar
    while pq.push(sonix_desktop::audio::DspParamUpdate::AmpBass(5.0)).is_ok() {}

    let (code, body) = http_request(
        port,
        "POST",
        "/api/params",
        &token,
        Some(r#"{"amp_1.gain":2.3}"#),
    );
    assert_eq!(code, 503, "CA-10: Fila cheia deveria retornar 503, retornou {} ({})", code, body);
    assert!(
        body.contains("retry_after_ms"),
        "CA-10: Resposta 503 deve informar retry_after_ms: {}",
        body
    );

    let (get_code, state_body) = http_request(port, "GET", "/api/state", &token, None);
    assert_eq!(get_code, 200);
    let val: serde_json::Value = serde_json::from_str(&state_body).unwrap();
    let gain = val["amp_1.gain"].as_f64().unwrap();
    assert!(
        (gain - 2.3).abs() > 0.01,
        "CA-10: AppState foi mutado para {} mesmo com comando rejeitado!",
        gain
    );
}

#[test]
fn test_ca_05_large_block_over_4096_processed_without_truncation() {
    let mut c1 = RealtimeDspChain::new(48000.0);
    let mut c2 = RealtimeDspChain::new(48000.0);
    c1.prepare(48000.0, 256);
    c2.prepare(48000.0, 256);

    let len = 4352usize; // > 4096 (17 x 256)
    let input: Vec<f32> = (0..len)
        .map(|i| (2.0 * std::f32::consts::PI * 220.0 * (i as f32 / 48000.0)).sin() * 0.2)
        .collect();
    let mut out_full = vec![0.0f32; len];
    let mut out_chunked = vec![0.0f32; len];

    c1.process(&input, &mut out_full);
    for chunk in 0..(len / 256) {
        let s = chunk * 256;
        c2.process(&input[s..s + 256], &mut out_chunked[s..s + 256]);
    }

    let tail_energy: f32 = out_full[4096..].iter().map(|x| x * x).sum();
    assert!(
        tail_energy > 1e-4,
        "CA-5: amostras além de 4096 foram silenciadas/truncadas (energy={})",
        tail_energy
    );
    let max_diff = out_full
        .iter()
        .zip(out_chunked.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        max_diff <= 1e-6,
        "CA-5: processamento chunked >4096 divergiu de blocos de 256: {}",
        max_diff
    );
}

#[test]
fn test_ca_06_contention_bypasses_and_increments_counter() {
    use sonix_desktop::audio::{DesktopAudioHost, DspChainSlot, RealtimeDuplexEngine};

    let duplex = RealtimeDuplexEngine::with_target_period(2048, 128);
    let in_sig = vec![0.35f32; 256];
    duplex.push_captured_samples(&in_sig);

    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let slot = DspChainSlot::new(chain);
    let mut rt_chain = slot.take_for_audio_thread().expect("posse exclusiva RT");
    let tel = Arc::new(TelemetryBuffer::new(256));

    let mut data = vec![0.0f32; 256];
    let mut mono_in = vec![0.0f32; 128];
    let mut mono_out = vec![0.0f32; 128];

    // Spec 093 CA-2 (revoga dry-bypass da Spec 034 CA-6): callback possui &mut RealtimeDspChain sem Mutex/try_lock.
    DesktopAudioHost::process_output_callback_f32(
        &mut data,
        2,
        48000.0,
        &mut mono_in,
        &mut mono_out,
        &duplex,
        &mut rt_chain,
        &tel,
        None,
    );

    assert_eq!(
        tel.dsp_contention_silence_blocks(),
        0,
        "Spec 093 CA-2: sem Mutex no callback RT, contenção deve ser sempre 0"
    );
}

#[test]
fn test_ca_09_sample_rate_and_period_propagate_to_chain_and_telemetry() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(96000.0, 256);
    assert_eq!(
        f32::from_bits(
            chain
                .shared_sample_rate_bits()
                .load(std::sync::atomic::Ordering::Relaxed)
        ),
        96000.0
    );

    let telemetry = Arc::new(TelemetryBuffer::new(512));
    let dsp = Arc::new(Mutex::new(chain));
    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry, Some(dsp)).expect("bind");

    let (code, body) = http_request(server.port(), "GET", "/api/audio/status", &server.session_token(), None);
    assert_eq!(code, 200);
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["sample_rate"].as_u64(), Some(96000), "CA-9: {}", body);
}

#[test]
fn test_ca_11_all_ui_dispatchable_params_apply_through_param_queue() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let dsp = Arc::new(Mutex::new(chain));
    let telemetry = Arc::new(TelemetryBuffer::new(512));
    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry, Some(dsp.clone()))
        .expect("bind");
    let token = server.session_token();

    let payload = serde_json::json!({
        "amp_1.model": "jcm800_2203",
        "amp_1.gain": 6.5,
        "amp_1.preamp_tube": "12at7",
        "amp_1.power_tube": "kt88",
        "amp_1.rectifier": "gz34",
        "amp_1.tonestack": "fender",
        "amp_2.enabled": true,
        "amp_2.model": "dual_rect",
        "cab_1.mic1.model": "r121",
        "cab_1.mic2.model": "u87",
        "cab_2.enabled": true,
        "cab_2.speaker": "v30_4x12",
        "split.mode": "crossover",
        "rev_1.predelay_ms": 24.0,
        "rev_1.damping": 4.5,
        "rev_1.spillover": true,
        "dly_1.mode": "2",
        "dly_1.repeat_rate": 4.2
    })
    .to_string();

    let (code, body) = http_request(server.port(), "POST", "/api/params", &token, Some(&payload));
    assert_eq!(code, 200, "CA-11: POST /api/params falhou: {}", body);

    let mut guard = dsp.lock().unwrap();
    let drained = guard.drain_param_queue();
    assert_eq!(
        drained, 19,
        "CA-11: os 18 parâmetros de controle UI + 1 auto-pareamento cab_1.speaker devem ser enfileirados em DspParamUpdate (drained={})",
        drained
    );
}

#[test]
fn test_ca_03_simulated_host_480_frames_10s_zero_fill() {
    use sonix_desktop::audio::SimulatedAudioHost;

    let mut host = SimulatedAudioHost::new(48000, 480);
    // Aquecimento de 10 blocos
    for _ in 0..10 {
        let _ = host.tick_sine_1khz();
    }
    host.reset_zero_fill_counter();

    // 1000 blocos x 480 frames = 480.000 amostras = 10 s a 48 kHz
    for _ in 0..1000 {
        let out = host.tick_sine_1khz();
        assert_eq!(out.len(), 480);
    }
    assert_eq!(
        host.zero_fill_samples(),
        0,
        "CA-3: host simulado a 480 frames teve zero-fill após aquecimento!"
    );
}

#[test]
fn test_ca_04_tuner_at_44100_hz_detects_a2_within_1_cent() {
    use ampforge_dsp::tuner::TunerProcessor;
    use sonix_desktop::audio::SimulatedAudioHost;

    let mut host = SimulatedAudioHost::new(44100, 256);
    let ring = host.clean_input_ring();
    assert_eq!(
        ring.sample_rate() as u32,
        44100,
        "CA-4: clean_input_ring deveria acompanhar 44100 Hz"
    );

    // 20 blocos de 256 = 5120 amostras de A2 = 110.00 Hz a 44100 Hz
    for _ in 0..20 {
        host.push_sine_to_chain(110.0, 256);
    }

    let mut buf = vec![0.0f32; 2048];
    ring.read_latest(&mut buf);
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(ring.sample_rate());
    tuner.set_a4_reference_hz(440.0);
    let res = tuner.detect_pitch(&buf);
    assert_eq!(res.note_name, "A2", "CA-4: esperado A2 a 44100 Hz, veio {:?}", res);
    assert!(
        res.cents_deviation.abs() <= 1.0,
        "CA-4: desvio em cents fora de +-1.0: {}",
        res.cents_deviation
    );
}

#[test]
fn test_ca_05_8192_frame_callback_overwrites_entire_output_buffer() {
    use sonix_desktop::audio::{DesktopAudioHost, RealtimeDuplexEngine};

    let duplex = RealtimeDuplexEngine::with_target_period(32768, 256);
    let input_sig: Vec<f32> = (0..16384)
        .map(|i| (2.0 * std::f32::consts::PI * 330.0 * (i as f32 / 48000.0)).sin() * 0.25)
        .collect();
    duplex.push_captured_samples(&input_sig);

    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);
    let mut dsp = chain;
    let tel = Arc::new(TelemetryBuffer::new(512));

    let sentinel = 999.0f32;
    let mut data = vec![sentinel; 8192 * 2];
    let mut mono_in = vec![0.0f32; 4096];
    let mut mono_out = vec![0.0f32; 4096];

    DesktopAudioHost::process_output_callback_f32(
        &mut data,
        2,
        48000.0,
        &mut mono_in,
        &mut mono_out,
        &duplex,
        &mut dsp,
        &tel,
        None,
    );

    assert!(
        data.iter().all(|&x| (x - sentinel).abs() > 1e-3 && x.is_finite()),
        "CA-5: callback de 8192 frames deixou frames não escritos ou sentinelas!"
    );
}
