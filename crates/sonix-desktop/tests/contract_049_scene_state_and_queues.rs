//! crates/sonix-desktop/tests/contract_049_scene_state_and_queues.rs
//! Testes de Contrato para Spec 049 (Parte A):
//! - CA-1: Looper fora do crossfade de cena (continuidade de RMS e posição)
//! - CA-2: Sem eco antigo após troca de cena com spillover desligado
//! - CA-3: Spillover ligado mantém cauda e decai sem degrau; desligado cai abaixo de -80 dBFS em até 30 ms
//! - CA-7: Fila de recursos com vários produtores sem sobrescrita/double free
//! - CA-8: Cena inválida responde 400 e não muta estado
//! - CA-9: Estado reflete toda chave aceita por dispatch
//! - CA-14: Telemetria sem lock da cadeia (nem try_lock) e zero contenção

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{
    DspParamUpdate, LockFreeResourceQueue, LooperActionCmd, PreparedDspResource, RealtimeDspChain,
    RealtimeDuplexEngine, TelemetryBuffer,
};

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

#[test]
fn ca7_lock_free_resource_queue_multi_producer_delivers_all_unique_items_and_zero_consumer_drops() {
    let queue = LockFreeResourceQueue::new();
    let retired_bin = Arc::new(Mutex::new(Vec::<PreparedDspResource>::with_capacity(2048)));
    let consumer_done = Arc::new(AtomicBool::new(false));
    let consumer_deallocs = Arc::new(AtomicUsize::new(usize::MAX));
    let reaper_deallocs = Arc::new(AtomicUsize::new(0));

    let q_p1 = queue.clone();
    let p1 = thread::spawn(move || {
        for id in 0..1000usize {
            let mut item = PreparedDspResource::DiSamples(vec![id as f32; 32]);
            while let Err(returned) = q_p1.push(item) {
                item = returned;
                thread::yield_now();
            }
        }
    });

    let q_p2 = queue.clone();
    let p2 = thread::spawn(move || {
        for id in 1000..2000usize {
            let mut item = PreparedDspResource::DiSamples(vec![id as f32; 32]);
            while let Err(returned) = q_p2.push(item) {
                item = returned;
                thread::yield_now();
            }
        }
    });

    let q_cons = queue.clone();
    let bin_cons = retired_bin.clone();
    let done_cons = consumer_done.clone();
    let cons_dealloc_ref = consumer_deallocs.clone();

    let consumer = thread::spawn(move || {
        let mut seen = [0u16; 2000];
        let mut received = 0usize;
        let start = Instant::now();

        let (_allocs, deallocs) = measure_allocs(|| {
            while received < 2000 && start.elapsed() < Duration::from_secs(5) {
                if let Some(res) = q_cons.pop() {
                    if let PreparedDspResource::DiSamples(ref v) = res {
                        let id = v[0].round() as usize;
                        assert!(id < 2000, "ID fora da faixa: {}", id);
                        seen[id] += 1;
                    }
                    if let Ok(mut guard) = bin_cons.lock() {
                        guard.push(res);
                    }
                    received += 1;
                } else {
                    thread::yield_now();
                }
            }
        });

        cons_dealloc_ref.store(deallocs, Ordering::SeqCst);
        done_cons.store(true, Ordering::SeqCst);

        assert_eq!(received, 2000, "CA-7: consumidor recebeu {} de 2000 itens", received);
        for (id, &count) in seen.iter().enumerate() {
            assert_eq!(count, 1, "CA-7: ID {} recebido {} vezes (esperado 1)", id, count);
        }
    });

    let bin_reaper = retired_bin.clone();
    let done_reaper = consumer_done.clone();
    let reaper_dealloc_ref = reaper_deallocs.clone();

    let reaper = thread::spawn(move || {
        let mut total_dropped = 0usize;
        let mut total_deallocs = 0usize;
        loop {
            let mut batch = Vec::new();
            if let Ok(mut guard) = bin_reaper.lock() {
                batch.append(&mut *guard);
            }
            if !batch.is_empty() {
                let count = batch.len();
                let (_a, d) = measure_allocs(|| {
                    drop(batch);
                });
                total_dropped += count;
                total_deallocs += d;
            } else if done_reaper.load(Ordering::SeqCst) {
                break;
            } else {
                thread::yield_now();
            }
        }
        reaper_dealloc_ref.store(total_deallocs, Ordering::SeqCst);
        assert_eq!(total_dropped, 2000, "CA-7: reaper dropou {} itens", total_dropped);
    });

    p1.join().expect("p1 falhou");
    p2.join().expect("p2 falhou");
    consumer.join().expect("consumidor falhou");
    reaper.join().expect("reaper falhou");

    assert_eq!(
        consumer_deallocs.load(Ordering::SeqCst),
        0,
        "CA-7: thread consumidora não pode desalocar recursos"
    );
    assert!(
        reaper_deallocs.load(Ordering::SeqCst) >= 2000,
        "CA-7: reaper deveria registrar >= 2000 desalocações (registrou {})",
        reaper_deallocs.load(Ordering::SeqCst)
    );
}

#[test]
fn ca14_telemetry_handler_does_not_take_dsp_chain_lock_in_source() {
    let api_src = include_str!("../src/api.rs");
    let start = api_src
        .find("\"/api/telemetry\" =>")
        .expect("CA-14: handler /api/telemetry não encontrado em api.rs");
    let rest = &api_src[start..];
    let end = rest
        .find("\"/api/devices\" =>")
        .unwrap_or_else(|| rest.len().min(1200));
    let arm = &rest[..end];

    assert!(
        !arm.contains("try_lock") && !arm.contains("dsp_chain") && !arm.contains("dsp_ref"),
        "CA-14: /api/telemetry não pode tomar lock da cadeia (nem try_lock). Trecho:\n{}",
        arm
    );
}

#[test]
fn ca14_telemetry_reads_b_plus_v_from_buffer_with_zero_contention() {
    let telemetry = Arc::new(TelemetryBuffer::new(1024));
    let chain = RealtimeDspChain::new(48000.0);
    let param_queue = chain.param_queue();

    let dsp_arc = Arc::new(Mutex::new(chain));
    let duplex = Arc::new(RealtimeDuplexEngine::new(256));

    let ctx = Arc::new(ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        telemetry.clone(),
        Some(dsp_arc.clone()),
        Some(param_queue),
        None,
        None,
    ));

    let stop = Arc::new(AtomicBool::new(false));
    let stop_reader = stop.clone();
    let ctx_reader = ctx.clone();

    let reader = thread::spawn(move || {
        let mut min_b_plus = f32::MAX;
        let mut max_b_plus = f32::MIN;
        let mut reads = 0usize;
        while !stop_reader.load(Ordering::Relaxed) {
            let (code, _, body) = dispatch(&ctx_reader, "/api/telemetry", "GET", &[]);
            assert_eq!(code, 200);
            let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
            let b_plus = v["b_plus_v"].as_f64().unwrap() as f32;
            min_b_plus = min_b_plus.min(b_plus);
            max_b_plus = max_b_plus.max(b_plus);
            reads += 1;
            thread::sleep(Duration::from_millis(1));
        }
        (reads, min_b_plus, max_b_plus)
    });

    // Simula callback de áudio por ~1.5s mantendo o lock da cadeia durante process()
    let in_buf = vec![0.8f32; 256];
    let mut out_buf = vec![0.0f32; 256];
    for _ in 0..300 {
        if let Ok(mut guard) = dsp_arc.try_lock() {
            guard.process(&in_buf, &mut out_buf);
            telemetry.update_b_plus_v(guard.current_b_plus_v());
            let _ = duplex.target_period();
        } else {
            telemetry.record_dsp_contention_silence();
        }
        thread::sleep(Duration::from_millis(5));
    }

    stop.store(true, Ordering::Relaxed);
    let (reads, min_b_plus, _max_b_plus) = reader.join().unwrap();

    assert!(reads >= 200, "CA-14: leitor rodou poucas vezes: {}", reads);
    assert!(
        min_b_plus.is_finite() && min_b_plus > 50.0 && min_b_plus < 500.0,
        "CA-14: b_plus_v inválido: {}",
        min_b_plus
    );
    assert_eq!(
        telemetry.dsp_contention_silence_blocks(),
        0,
        "CA-14: houve contenção no callback de áudio!"
    );
}

fn rms_of(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|&x| (x as f64) * (x as f64)).sum();
    (sum_sq / samples.len() as f64).sqrt() as f32
}

fn goertzel_db(samples: &[f32], freq_hz: f32, sample_rate: f32) -> f32 {
    if samples.is_empty() {
        return -120.0;
    }
    let n = samples.len() as f64;
    let omega = 2.0 * std::f64::consts::PI * (freq_hz as f64) / (sample_rate as f64);
    let coeff = 2.0 * omega.cos();
    let mut q1 = 0.0f64;
    let mut q2 = 0.0f64;
    for &x in samples {
        let q0 = coeff * q1 - q2 + (x as f64);
        q2 = q1;
        q1 = q0;
    }
    let mag_sq = (q1 * q1 + q2 * q2 - q1 * q2 * coeff).max(0.0);
    // Amplitude de senoide normalizada: 2 * sqrt(mag_sq) / N
    let amp = 2.0 * mag_sq.sqrt() / n;
    if amp <= 1e-12 {
        -120.0
    } else {
        20.0 * (amp as f32).log10()
    }
}

#[test]
fn ca1_looper_survives_scene_switch_and_keeps_advancing() {
    let mut chain = RealtimeDspChain::new(48000.0);
    let pq = chain.param_queue();
    // Bypass amp para sinal limpo previsível no looper
    let _ = pq.push(DspParamUpdate::AmpEnabled(false));
    let _ = pq.push(DspParamUpdate::CabEnabled(false));
    let _ = pq.push(DspParamUpdate::GateEnabled(false));
    let _ = pq.push(DspParamUpdate::DriveEnabled(false));
    let _ = pq.push(DspParamUpdate::LooperAction(LooperActionCmd::Record));

    let block = 256usize;
    let mut in_buf = vec![0.0f32; block];
    let mut out_buf = vec![0.0f32; block];

    // Grava 2 s de senoide 440 Hz
    let total_2s = 48000 * 2;
    let mut sample_idx = 0usize;
    while sample_idx < total_2s {
        for i in 0..block {
            let t = (sample_idx + i) as f32 / 48000.0;
            in_buf[i] = 0.3 * (2.0 * std::f32::consts::PI * 440.0 * t).sin();
        }
        chain.process(&in_buf, &mut out_buf);
        sample_idx += block;
    }

    let _ = pq.push(DspParamUpdate::LooperAction(LooperActionCmd::Play));
    in_buf.fill(0.0);

    // Toca 2 s antes da troca
    let mut before_samples = Vec::with_capacity(total_2s);
    while before_samples.len() < total_2s {
        chain.process(&in_buf, &mut out_buf);
        before_samples.extend_from_slice(&out_buf);
    }
    let rms_before = rms_of(&before_samples);
    assert!(rms_before > 0.05, "CA-1: looper deveria estar tocando antes da troca (rms={})", rms_before);

    // Avança 200 ms para progress_pct > 0
    for _ in 0..38 {
        chain.process(&in_buf, &mut out_buf);
    }
    let pct_before = chain.looper().progress_pct();
    assert!(pct_before > 1.0, "CA-1: progress_pct deveria estar > 1% antes da troca: {}", pct_before);

    // Troca de cena
    let _ = pq.push(DspParamUpdate::BeginPresetCrossfade);
    let _ = pq.push(DspParamUpdate::AmpGain(9.0));

    // Processa 1 bloco para aplicar a troca
    chain.process(&in_buf, &mut out_buf);
    let loop_len_after = chain.looper().loop_length_samples();
    let pct_after = chain.looper().progress_pct();
    assert!(loop_len_after > 0, "CA-1: loop_length zerou na troca de cena!");
    assert!(
        pct_after >= pct_before || pct_after > 0.0,
        "CA-1: looper_position voltou a 0 (antes={}, depois={})",
        pct_before,
        pct_after
    );

    // Toca 2 s depois da troca
    let mut after_samples = Vec::with_capacity(total_2s);
    while after_samples.len() < total_2s {
        chain.process(&in_buf, &mut out_buf);
        after_samples.extend_from_slice(&out_buf);
    }
    let rms_after = rms_of(&after_samples);
    assert!(
        rms_after >= 0.90 * rms_before,
        "CA-1: RMS pós-troca ({}) ficou abaixo de 90% do RMS pré-troca ({})",
        rms_after,
        rms_before
    );
}

#[test]
fn ca2_no_stale_delay_echo_returns_after_scene_roundtrip_with_spillover_off() {
    let mut chain = RealtimeDspChain::new(48000.0);
    let pq = chain.param_queue();
    let _ = pq.push(DspParamUpdate::AmpEnabled(false));
    let _ = pq.push(DspParamUpdate::CabEnabled(false));
    let _ = pq.push(DspParamUpdate::GateEnabled(false));
    let _ = pq.push(DspParamUpdate::DriveEnabled(false));
    let _ = pq.push(DspParamUpdate::Dly1Enabled(true));
    let _ = pq.push(DspParamUpdate::Dly1Spillover(false));
    let _ = pq.push(DspParamUpdate::Dly1TimeMs(300.0));
    let _ = pq.push(DspParamUpdate::Dly1Intensity(70.0));
    let _ = pq.push(DspParamUpdate::Dly1Mix(50.0));

    let block = 256usize;
    let mut in_buf = vec![0.0f32; block];
    let mut out_buf = vec![0.0f32; block];

    // 1. Rajada de 1 kHz por ~150 ms na Cena A
    let mut idx = 0usize;
    for _ in 0..28 {
        for i in 0..block {
            let t = (idx + i) as f32 / 48000.0;
            in_buf[i] = 0.5 * (2.0 * std::f32::consts::PI * 1000.0 * t).sin();
        }
        chain.process(&in_buf, &mut out_buf);
        idx += block;
    }

    // 2. Troca para Cena B
    let _ = pq.push(DspParamUpdate::BeginPresetCrossfade);
    // Rajada de 2 kHz na Cena B
    idx = 0;
    for _ in 0..28 {
        for i in 0..block {
            let t = (idx + i) as f32 / 48000.0;
            in_buf[i] = 0.5 * (2.0 * std::f32::consts::PI * 2000.0 * t).sin();
        }
        chain.process(&in_buf, &mut out_buf);
        idx += block;
    }

    // 3. Troca de volta para Cena A e entrada em silêncio por 3 s
    let _ = pq.push(DspParamUpdate::BeginPresetCrossfade);
    in_buf.fill(0.0);
    let mut tail_3s = Vec::with_capacity(48000 * 3);
    while tail_3s.len() < 48000 * 3 {
        chain.process(&in_buf, &mut out_buf);
        tail_3s.extend_from_slice(&out_buf);
    }

    // Pula os primeiros 30 ms do crossfade e mede energia em 1 kHz por 3 s
    let energy_1khz_db = goertzel_db(&tail_3s[1440..], 1000.0, 48000.0);
    assert!(
        energy_1khz_db < -80.0,
        "CA-2: eco antigo de 1 kHz voltou após troca A->B->A: {:.2} dBFS",
        energy_1khz_db
    );
}

#[test]
fn ca3_spillover_keeps_tail_above_minus_60_at_500ms_and_off_cuts_below_minus_80_in_30ms() {
    // Parte 1: Spillover LIGADO
    let mut chain = RealtimeDspChain::new(48000.0);
    let pq = chain.param_queue();
    let _ = pq.push(DspParamUpdate::AmpEnabled(false));
    let _ = pq.push(DspParamUpdate::CabEnabled(false));
    let _ = pq.push(DspParamUpdate::GateEnabled(false));
    let _ = pq.push(DspParamUpdate::DriveEnabled(false));
    let _ = pq.push(DspParamUpdate::Dly1Enabled(true));
    let _ = pq.push(DspParamUpdate::Dly1Spillover(true));
    let _ = pq.push(DspParamUpdate::Dly1TimeMs(220.0));
    let _ = pq.push(DspParamUpdate::Dly1Intensity(50.0));
    let _ = pq.push(DspParamUpdate::Dly1Mix(50.0));
    let _ = pq.push(DspParamUpdate::Rev1Enabled(true));
    let _ = pq.push(DspParamUpdate::Rev1Spillover(true));
    let _ = pq.push(DspParamUpdate::Rev1DecayS(2.5));
    let _ = pq.push(DspParamUpdate::Rev1Mix(50.0));

    let block = 256usize;
    let mut in_buf = vec![0.0f32; block];
    let mut out_buf = vec![0.0f32; block];

    // Rajada de -6 dBFS (0.5 pico) por 160 ms (30 * 256 = 7680 amostras = 128 ciclos exatos de 800 Hz)
    let mut idx = 0usize;
    for _ in 0..30 {
        for i in 0..block {
            let t = (idx + i) as f32 / 48000.0;
            in_buf[i] = 0.5 * (2.0 * std::f32::consts::PI * 800.0 * t).sin();
        }
        chain.process(&in_buf, &mut out_buf);
        idx += block;
    }

    // Troca de cena desligando delay e reverb (com spillover ligado) + silêncio
    let _ = pq.push(DspParamUpdate::BeginPresetCrossfade);
    let _ = pq.push(DspParamUpdate::Dly1Enabled(false));
    let _ = pq.push(DspParamUpdate::Rev1Enabled(false));
    in_buf.fill(0.0);

    let mut after_on = Vec::with_capacity(48000);
    while after_on.len() < 48000 {
        chain.process(&in_buf, &mut out_buf);
        after_on.extend_from_slice(&out_buf);
    }

    // Em 500 ms (amostras 23520..24480) cauda > -60 dBFS
    let rms_500ms = rms_of(&after_on[23520..24480]);
    let db_500ms = if rms_500ms > 1e-12 {
        20.0 * rms_500ms.log10()
    } else {
        -120.0
    };
    assert!(
        db_500ms > -60.0,
        "CA-3 (spillover ON): cauda em 500 ms caiu para {:.2} dBFS (esperado > -60 dBFS)",
        db_500ms
    );

    // Verifica ausência de degrau brusco nos primeiros 50 ms pós-troca
    let max_step = after_on[..2400]
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0f32, f32::max);
    assert!(
        max_step < 0.15,
        "CA-3 (spillover ON): degrau detectado na troca (max_step={})",
        max_step
    );

    // Parte 2: Spillover DESLIGADO
    let mut chain_off = RealtimeDspChain::new(48000.0);
    let pq_off = chain_off.param_queue();
    let _ = pq_off.push(DspParamUpdate::AmpEnabled(false));
    let _ = pq_off.push(DspParamUpdate::CabEnabled(false));
    let _ = pq_off.push(DspParamUpdate::GateEnabled(false));
    let _ = pq_off.push(DspParamUpdate::DriveEnabled(false));
    let _ = pq_off.push(DspParamUpdate::Dly1Enabled(true));
    let _ = pq_off.push(DspParamUpdate::Dly1Spillover(false));
    let _ = pq_off.push(DspParamUpdate::Dly1TimeMs(220.0));
    let _ = pq_off.push(DspParamUpdate::Dly1Intensity(50.0));
    let _ = pq_off.push(DspParamUpdate::Dly1Mix(50.0));
    let _ = pq_off.push(DspParamUpdate::Rev1Enabled(true));
    let _ = pq_off.push(DspParamUpdate::Rev1Spillover(false));
    let _ = pq_off.push(DspParamUpdate::Rev1DecayS(2.5));
    let _ = pq_off.push(DspParamUpdate::Rev1Mix(50.0));

    let mut idx = 0usize;
    for _ in 0..30 {
        for i in 0..block {
            let t = (idx + i) as f32 / 48000.0;
            in_buf[i] = 0.5 * (2.0 * std::f32::consts::PI * 800.0 * t).sin();
        }
        chain_off.process(&in_buf, &mut out_buf);
        idx += block;
    }

    let _ = pq_off.push(DspParamUpdate::BeginPresetCrossfade);
    let _ = pq_off.push(DspParamUpdate::Dly1Enabled(false));
    let _ = pq_off.push(DspParamUpdate::Rev1Enabled(false));
    in_buf.fill(0.0);

    let mut after_off = Vec::with_capacity(4800);
    while after_off.len() < 4800 {
        chain_off.process(&in_buf, &mut out_buf);
        after_off.extend_from_slice(&out_buf);
    }

    // Após 30 ms (amostra 1440..3840) cauda < -80 dBFS
    let rms_30ms = rms_of(&after_off[1440..3840]);
    let db_30ms = if rms_30ms > 1e-12 {
        20.0 * rms_30ms.log10()
    } else {
        -120.0
    };
    assert!(
        db_30ms < -80.0,
        "CA-3 (spillover OFF): cauda após 30 ms ficou em {:.2} dBFS (esperado < -80 dBFS)",
        db_30ms
    );
}

#[test]
fn ca8_invalid_scene_select_returns_400_and_leaves_state_unchanged() {
    let chain = RealtimeDspChain::new(48000.0);
    let param_queue = chain.param_queue();
    let dsp_arc = Arc::new(Mutex::new(chain));

    let mut st = AppState::default();
    st.grid.blocks.push(sonix_core::persistence::BlockDto {
        instance_id: "amp_1".to_string(),
        block_type: "amp".to_string(),
        row: 0,
        col: 2,
        bypass_by_scene: [false; 8],
        active_channel_by_scene: [0, 1, 0, 0, 0, 0, 0, 0],
        oversampling: "4x".to_string(),
    });
    let mut amp_ch = std::collections::HashMap::new();
    let mut ch_b = std::collections::HashMap::new();
    ch_b.insert("amp_1.model".to_string(), serde_json::json!("foo"));
    amp_ch.insert("B".to_string(), ch_b);
    st.channels.insert("amp_1".to_string(), amp_ch);

    let ctx = ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(st)),
        Arc::new(TelemetryBuffer::new(1024)),
        Some(dsp_arc),
        Some(param_queue),
        None,
        None,
    );

    let (code_before, _, state_before) = dispatch(&ctx, "/api/state", "GET", &[]);
    assert_eq!(code_before, 200);

    let (code, _, body) = dispatch(&ctx, "/api/scenes/select", "POST", br#"{"scene":"B"}"#);
    let body_str = String::from_utf8_lossy(&body);
    assert_eq!(
        code, 400,
        "CA-8: cena com amp_1.model='foo' deve responder 400, respondeu {}: {}",
        code, body_str
    );
    assert!(
        body_str.contains("amp_1.model"),
        "CA-8: resposta 400 deve citar a chave ofensiva 'amp_1.model': {}",
        body_str
    );

    let (code_after, _, state_after) = dispatch(&ctx, "/api/state", "GET", &[]);
    assert_eq!(code_after, 200);
    assert_eq!(
        state_before, state_after,
        "CA-8: GET /api/state mudou após erro 400 em /api/scenes/select"
    );
}

#[test]
fn ca9_state_reflects_every_accepted_param_and_aliases_and_preserves_on_save_load() {
    let chain = RealtimeDspChain::new(48000.0);
    let param_queue = chain.param_queue();
    let dsp_arc = Arc::new(Mutex::new(chain));

    let ctx = ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(1024)),
        Some(dsp_arc),
        Some(param_queue),
        None,
        None,
    );

    // Testa explicitamente pitch_1.* e os aliases citados no CA-9
    let explicit_cases: Vec<(&str, &str, serde_json::Value)> = vec![
        ("pitch_1.enabled", "pitch_1.enabled", serde_json::json!(true)),
        ("pitch_1.model", "pitch_1.model", serde_json::json!("drop_capo")),
        ("pitch_1.semitones", "pitch_1.semitones", serde_json::json!(-5.0)),
        ("pitch_1.key", "pitch_1.key", serde_json::json!("D")),
        ("pitch_1.scale", "pitch_1.scale", serde_json::json!("natural_minor")),
        ("pitch_1.mix", "pitch_1.mix", serde_json::json!(42.0)),
        ("pitch_1.mix_pct", "pitch_1.mix", serde_json::json!(65.0)),
        ("mod_2.mix_pct", "mod_2.mix", serde_json::json!(37.0)),
        ("rev_1.mix_pct", "rev_1.mix", serde_json::json!(44.0)),
        ("cab_1.model", "cab_1.speaker", serde_json::json!("v30_4x12")),
        ("cab_2.model", "cab_2.speaker", serde_json::json!("greenback_4x12")),
    ];

    for (post_key, canonical_key, val) in &explicit_cases {
        let payload = serde_json::json!({ (*post_key): val });
        let (code, _, resp) = dispatch(
            &ctx,
            "/api/params",
            "POST",
            &serde_json::to_vec(&payload).unwrap(),
        );
        assert_eq!(
            code,
            200,
            "CA-9: POST {} falhou: {}",
            post_key,
            String::from_utf8_lossy(&resp)
        );
        let (_, _, state_bytes) = dispatch(&ctx, "/api/state", "GET", &[]);
        let state: serde_json::Value = serde_json::from_slice(&state_bytes).unwrap();
        let actual = state
            .get(*canonical_key)
            .unwrap_or_else(|| panic!("CA-9: chave canônica '{}' ausente em GET /api/state", canonical_key));
        assert_eq!(
            actual, val,
            "CA-9: após POST {} = {}, GET /api/state['{}'] = {} (esperado {})",
            post_key, val, canonical_key, actual, val
        );
    }

    // Percorre todas as chaves escalares do GET /api/state (sem lista escrita à mão)
    let (_, _, state_bytes) = dispatch(&ctx, "/api/state", "GET", &[]);
    let state_obj = serde_json::from_slice::<serde_json::Value>(&state_bytes)
        .unwrap()
        .as_object()
        .unwrap()
        .clone();

    let mut tested_from_table = 0usize;
    for (k, v) in &state_obj {
        // Ignora metadados de preset/grafo que não são parâmetros de /api/params
        if !k.contains('.') || k.starts_with("preset.") || k == "looper.action" {
            continue;
        }
        let next_val: Option<serde_json::Value> = match v {
            serde_json::Value::Bool(b) => Some(serde_json::json!(!b)),
            serde_json::Value::Number(n) => {
                let f = n.as_f64().unwrap_or(0.0);
                // Escolhe um valor ligeiramente diferente dentro da faixa típica
                let candidate = if k == "tuner.ref_hz" {
                    if (f - 442.0).abs() > 0.1 { 442.0 } else { 440.0 }
                } else if k.contains("threshold_db") {
                    if (f - (-55.0)).abs() > 0.5 { -55.0 } else { -60.0 }
                } else if k.contains("distance_cm") {
                    if (f - 5.0).abs() > 0.5 { 5.0 } else { 8.0 }
                } else if k.contains("hicut_khz") {
                    if (f - 6.5).abs() > 0.5 { 6.5 } else { 7.5 }
                } else if k.contains("resonance_hz") {
                    if (f - 105.0).abs() > 0.5 { 105.0 } else { 95.0 }
                } else if k.contains("crossover_hz") {
                    if (f - 600.0).abs() > 0.5 { 600.0 } else { 500.0 }
                } else if k.contains("predelay_ms") {
                    if (f - 25.0).abs() > 0.5 { 25.0 } else { 15.0 }
                } else if k.contains("decay_s") {
                    if (f - 2.2).abs() > 0.1 { 2.2 } else { 1.8 }
                } else if k.contains("rate_hz") {
                    if (f - 1.5).abs() > 0.1 { 1.5 } else { 2.0 }
                } else if k.contains("semitones") {
                    if (f - (-2.0)).abs() > 0.1 { -2.0 } else { 2.0 }
                } else if k.contains("level_db") || k.contains("gain_db") || k.contains("volume_db") {
                    if (f - (-3.0)).abs() > 0.5 { -3.0 } else { 0.0 }
                } else if k.contains("release_ms") {
                    if (f - 120.0).abs() > 1.0 { 120.0 } else { 80.0 }
                } else if k.contains("attack_ms") || k.contains("hold_ms") {
                    if (f - 15.0).abs() > 0.5 { 15.0 } else { 10.0 }
                } else if k.contains("range_db") {
                    if (f - (-45.0)).abs() > 0.5 { -45.0 } else { -50.0 }
                } else if k.contains("freq_hz") {
                    if (f - 1200.0).abs() > 1.0 { 1200.0 } else { 1000.0 }
                } else if k.ends_with(".q") {
                    if (f - 1.4).abs() > 0.1 { 1.4 } else { 1.0 }
                } else if k.contains("mix") || k.contains("intensity") || k.contains("repeat_rate") {
                    if (f - 33.0).abs() > 0.5 { 33.0 } else { 50.0 }
                } else {
                    if (f - 4.0).abs() > 0.1 { 4.0 } else { 5.0 }
                };
                Some(serde_json::json!(candidate))
            }
            _ => None,
        };

        if let Some(new_v) = next_val {
            let payload = serde_json::json!({ (k.as_str()): new_v });
            let (code, _, resp_bytes) = dispatch(
                &ctx,
                "/api/params",
                "POST",
                &serde_json::to_vec(&payload).unwrap(),
            );
            let resp_json: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap_or_default();
            let was_applied = resp_json["applied"]
                .as_array()
                .map(|arr| arr.iter().any(|x| x.as_str() == Some(k.as_str())))
                .unwrap_or(false);
            if code == 200 && was_applied {
                let (_, _, after_bytes) = dispatch(&ctx, "/api/state", "GET", &[]);
                let after_obj: serde_json::Value = serde_json::from_slice(&after_bytes).unwrap();
                let actual = &after_obj[k.as_str()];
                if let (Some(a), Some(e)) = (actual.as_f64(), new_v.as_f64()) {
                    assert!(
                        (a - e).abs() < 1e-3,
                        "CA-9: chave '{}' não refletiu em GET /api/state (esperado {}, obtido {})",
                        k, e, a
                    );
                } else {
                    assert_eq!(
                        actual, &new_v,
                        "CA-9: chave '{}' não refletiu em GET /api/state",
                        k
                    );
                }
                tested_from_table += 1;
            }
        }
    }
    assert!(tested_from_table >= 25, "CA-9: poucas chaves testadas: {}", tested_from_table);

    // Save -> Load preserva pitch_1 e aliases
    let (save_code, _, save_body) = dispatch(
        &ctx,
        "/api/presets/save",
        "POST",
        br#"{"id":"ca9_test","name":"CA9 Test"}"#,
    );
    assert_eq!(save_code, 200, "CA-9 save falhou: {}", String::from_utf8_lossy(&save_body));

    // Altera valores antes de recarregar
    let _ = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"pitch_1.semitones":0,"cab_1.speaker":"twin_2x12_open"}"#,
    );
    let (load_code, _, load_body) = dispatch(
        &ctx,
        "/api/presets/load",
        "POST",
        br#"{"id":"ca9_test"}"#,
    );
    assert_eq!(load_code, 200, "CA-9 load falhou: {}", String::from_utf8_lossy(&load_body));

    let (_, _, reloaded_bytes) = dispatch(&ctx, "/api/state", "GET", &[]);
    let reloaded: serde_json::Value = serde_json::from_slice(&reloaded_bytes).unwrap();
    assert_eq!(reloaded["pitch_1.semitones"], serde_json::json!(-2.0));
    assert_eq!(reloaded["cab_1.speaker"], serde_json::json!("v30_4x12"));
}

#[test]
fn ca3_channel_switch_preserves_delay_and_reverb_spillover_tails() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);
    chain.set_gate_enabled(false);
    chain.set_amp_enabled(false);
    chain.set_cab_enabled(false);
    chain.set_dly_1_enabled(true);
    chain.set_dly_1_intensity(5.0);
    chain.set_dly_1_echo_volume(8.0);
    chain.set_dly_1_spillover(true);
    chain.set_rev_1_enabled(true);
    chain.set_rev_1_decay_s(2.5);
    chain.set_rev_1_mix(50.0);
    chain.set_rev_1_spillover(true);
    chain.set_active_channel(0);

    let burst: Vec<f32> = (0..256)
        .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 48000.0).sin())
        .collect();
    let silence = vec![0.0f32; 256];
    let mut out = vec![0.0f32; 256];

    for _ in 0..8 {
        chain.process(&burst, &mut out);
    }

    // Troca de canal com spillover ligado -> caudas de delay/reverb NÃO podem ser cortadas
    chain.set_active_channel(1);

    // 500 ms = 24000 amostras ≈ 94 blocos de 256
    for _ in 0..93 {
        chain.process(&silence, &mut out);
    }
    chain.process(&silence, &mut out);
    let rms = (out.iter().map(|x| x * x).sum::<f32>() / 256.0).sqrt();
    let db = 20.0 * rms.max(1e-12).log10();
    assert!(
        db > -60.0,
        "CA-3 (R2): troca de canal cortou cauda de delay/reverb com spillover ligado ({:.2} dBFS <= -60 dBFS)",
        db
    );
}

#[test]
fn ca2_clear_tail_on_scene_switch_is_o1_without_latency_spike() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 64);
    chain.set_gate_enabled(false);
    chain.set_amp_enabled(false);
    chain.set_cab_enabled(false);
    chain.set_dly_1_enabled(true);
    chain.set_dly_1_intensity(5.0);
    chain.set_dly_1_echo_volume(8.0);
    chain.set_dly_1_spillover(false);
    chain.set_rev_1_enabled(true);
    chain.set_rev_1_decay_s(2.5);
    chain.set_rev_1_mix(50.0);
    chain.set_rev_1_spillover(false);

    let burst: Vec<f32> = (0..64)
        .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 48000.0).sin())
        .collect();
    let silence = vec![0.0f32; 64];
    let mut out = vec![0.0f32; 64];

    for _ in 0..32 {
        chain.process(&burst, &mut out);
    }

    // Troca de canal com spillover desligado deve zerar cauda imediatamente
    chain.set_active_channel(1);
    for _ in 0..20 {
        chain.process(&silence, &mut out);
    }
    let tail_db = 20.0 * rms_of(&out).max(1e-12).log10();
    assert!(
        tail_db < -60.0,
        "CA-2 (R6): troca com spillover off deve silenciar cauda imediatamente ({tail_db:.2} dBFS)"
    );

    // O(1) clear_tail: 100 trocas de canal (spillover off) não podem fazer memset de 3 MB por troca
    let t0 = std::time::Instant::now();
    for i in 0..100 {
        chain.set_active_channel(i % 2);
    }
    let elapsed = t0.elapsed();
    assert!(
        elapsed < Duration::from_millis(5),
        "CA-2 (R6): 100 clear_tail() em troca de cena levaram {:?} (esperado O(1) < 5ms)",
        elapsed
    );
}
