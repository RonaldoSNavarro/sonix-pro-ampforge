//! Auditoria 2026-11-20 — AUD-20: Split balance no lugar, DynamicSplit parametrizado e crossover LR4.

use ampforge_dsp::dag::{SplitMode, SplitNode};
use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{LockFreeParamQueue, RealtimeDspChain, TelemetryBuffer};
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

fn make_ctx() -> (ApiContext, Arc<Mutex<RealtimeDspChain>>) {
    let mut chain = RealtimeDspChain::new(48_000.0);
    chain.prepare(48_000.0, 2048);
    let dsp = Arc::new(Mutex::new(chain));
    let pq = Arc::new(LockFreeParamQueue::new());
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

fn post_params(ctx: &ApiContext, body: &serde_json::Value) -> (u16, String) {
    let raw = body.to_string();
    let (status, _, resp) = dispatch(ctx, "/api/params", "POST", raw.as_bytes());
    (status, String::from_utf8_lossy(&resp).into_owned())
}

fn rms(buf: &[f32]) -> f32 {
    (buf.iter().map(|x| x * x).sum::<f32>() / buf.len().max(1) as f32).sqrt()
}

fn sine_wave(freq: f32, amp: f32, sr: f32, start_sample: usize, len: usize) -> Vec<f32> {
    (0..len)
        .map(|i| {
            let t = (start_sample + i) as f32 / sr;
            amp * (2.0 * PI * freq * t).sin()
        })
        .collect()
}

#[test]
fn aud_20_balance_sem_reset() {
    // 1. Verifica SplitNode::set_ratio no lugar, níveis A/B conforme razão +- 0,2 dB e zero reset brusco.
    let mut split = SplitNode::new(SplitMode::BalanceAb { ratio: 0.25 });
    split.set_sample_rate(48_000.0);

    let block = 512usize;
    let in1 = sine_wave(220.0, 0.5, 48_000.0, 0, block);
    let mut a1 = vec![0.0f32; block];
    let mut b1 = vec![0.0f32; block];
    split.process(&in1, &mut a1, &mut b1);

    let in2 = sine_wave(220.0, 0.5, 48_000.0, block, block);
    let mut a2 = vec![0.0f32; block];
    let mut b2 = vec![0.0f32; block];
    // Atualiza apenas a razão sem recriar o nó
    split.set_ratio(0.25);
    split.process(&in2, &mut a2, &mut b2);

    let expected_a = (1.0f32 - 0.25).sqrt();
    let expected_b = 0.25f32.sqrt();
    let rms_in = rms(&in2);
    let rms_a = rms(&a2);
    let rms_b = rms(&b2);
    let err_a_db = 20.0 * (rms_a / (rms_in * expected_a)).log10();
    let err_b_db = 20.0 * (rms_b / (rms_in * expected_b)).log10();
    assert!(
        err_a_db.abs() <= 0.2 && err_b_db.abs() <= 0.2,
        "AUD-20: níveis A/B divergentes da razão 0.25: err_a={err_a_db:.3} dB, err_b={err_b_db:.3} dB"
    );

    // Verifica continuidade de transição no limiar entre blocos (razao_transiente <= 2.0)
    let boundary_jump = (a2[0] - a1[block - 1]).abs();
    let max_step = a2.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0f32, f32::max).max(1e-6);
    let razao_transiente = boundary_jump / max_step;
    assert!(
        razao_transiente <= 2.0,
        "AUD-20: descontinuidade na atualização de ratio (razao_transiente={razao_transiente:.2})"
    );

    // 2. Verifica parâmetro split.balance pela API sem resetar o estado em RealtimeDspChain.
    let (ctx, dsp) = make_ctx();
    let (st, resp) = post_params(
        &ctx,
        &serde_json::json!({
            "gate.enabled": false,
            "amp_1.enabled": false,
            "cab_1.enabled": false,
            "amp_2.enabled": false,
            "cab_2.enabled": false,
            "comp_2.enabled": false,
            "mod_2.enabled": false,
            "eq_1.enabled": false,
            "dly_1.enabled": false,
            "rev_1.enabled": false,
            "split.enabled": true,
            "split.mode": "ab",
            "split.balance": 0.1,
            "merge.row_1_level_db": 0.0
        }),
    );
    assert_eq!(st, 200, "AUD-20: /api/params falhou para split.balance: {resp}");

    // Isola apenas linha A medindo com row_1_level_db = -60 e compara bal = 0.1 vs bal = 0.9
    let _ = post_params(&ctx, &serde_json::json!({ "merge.row_1_level_db": -60.0 }));
    let inp = sine_wave(330.0, 0.3, 48_000.0, 0, 1024);
    let mut out_01 = vec![0.0f32; 1024];
    {
        let mut ch = dsp.lock().unwrap();
        ch.drain_param_queue();
        ch.process(&inp, &mut out_01);
    }

    let (st2, resp2) = post_params(&ctx, &serde_json::json!({ "split.balance": 0.9 }));
    assert_eq!(st2, 200, "AUD-20: /api/params falhou em split.balance=0.9: {resp2}");
    let mut out_09 = vec![0.0f32; 1024];
    {
        let mut ch = dsp.lock().unwrap();
        ch.drain_param_queue();
        ch.process(&inp, &mut out_09);
    }

    // Esperado na linha A: sqrt(0.9) / sqrt(0.1) = 3.0 (+9.54 dB) +- 0.2 dB
    let ratio_db = 20.0 * (rms(&out_01[256..]) / rms(&out_09[256..]).max(1e-9)).log10();
    let expected_diff_db = 20.0 * (0.9f32.sqrt() / 0.1f32.sqrt()).log10();
    assert!(
        (ratio_db - expected_diff_db).abs() <= 0.2,
        "AUD-20: split.balance pela API não atuou na proporção esperada: medido {ratio_db:.3} dB, esperado {expected_diff_db:.3} dB"
    );
}

#[test]
fn aud_20_dinamico_obedece_parametros() {
    let (ctx, dsp) = make_ctx();
    let (st, resp) = post_params(
        &ctx,
        &serde_json::json!({
            "gate.enabled": false,
            "amp_1.enabled": false,
            "cab_1.enabled": false,
            "amp_2.enabled": false,
            "cab_2.enabled": false,
            "comp_2.enabled": false,
            "mod_2.enabled": false,
            "eq_1.enabled": false,
            "dly_1.enabled": false,
            "rev_1.enabled": false,
            "split.enabled": true,
            "split.mode": "dynamic_split",
            "split.dyn_threshold_db": -6.0,
            "split.dyn_attack_ms": 5.0,
            "split.dyn_release_ms": 80.0,
            "merge.row_1_level_db": -60.0
        }),
    );
    assert_eq!(st, 200, "AUD-20: parâmetros dynamic_split rejeitados: {resp}");

    // Sinal AC 440 Hz a -20 dBFS (amp = 0.1) para atravessar o DC blocker de entrada
    let sig = sine_wave(440.0, 0.1, 48_000.0, 0, 2048);
    let mut out_thresh_high = vec![0.0f32; 2048];
    {
        let mut ch = dsp.lock().unwrap();
        ch.drain_param_queue();
        ch.process(&sig, &mut out_thresh_high);
    }

    // Threshold -40 dB (< -20 dBFS): sinal vai para ramo B (atenuado pelo merge -60 dB)
    let (st_t, resp_t) = post_params(&ctx, &serde_json::json!({ "split.dyn_threshold_db": -40.0 }));
    assert_eq!(st_t, 200, "AUD-20: split.dyn_threshold_db rejeitado: {resp_t}");
    let mut out_thresh_low = vec![0.0f32; 2048];
    {
        let mut ch = dsp.lock().unwrap();
        ch.drain_param_queue();
        ch.process(&sig, &mut out_thresh_low);
    }

    let drop_db = 20.0 * (rms(&out_thresh_high[1024..]) / rms(&out_thresh_low[1024..]).max(1e-9)).log10();
    assert!(
        drop_db > 10.0,
        "AUD-20: split.dyn_threshold_db não mudou roteamento dinâmico (diferença={drop_db:.2} dB)"
    );

    // Attack 1 ms vs 50 ms no transiente
    let run_attack = |attack_ms: f32| -> f32 {
        let _ = post_params(
            &ctx,
            &serde_json::json!({
                "split.dyn_threshold_db": -12.0,
                "split.dyn_attack_ms": attack_ms,
                "split.dyn_release_ms": 200.0
            }),
        );
        let mut ch = dsp.lock().unwrap();
        ch.drain_param_queue();
        let silence = vec![0.0f32; 8192];
        let mut tmp = vec![0.0f32; 8192];
        ch.process(&silence, &mut tmp);
        let burst = sine_wave(440.0, 0.5, 48_000.0, 0, 720); // 15 ms
        let mut out_burst = vec![0.0f32; 720];
        ch.process(&burst, &mut out_burst);
        rms(&out_burst[240..])
    };
    let fast_att_a = run_attack(1.0);
    let slow_att_a = run_attack(50.0);
    assert!(
        slow_att_a > fast_att_a * 1.5,
        "AUD-20: dyn_attack_ms=1ms deve desviar para ramo B mais rápido que 50ms (fast_A={fast_att_a:.4}, slow_A={slow_att_a:.4})"
    );

    // Release 10 ms vs 500 ms após transiente
    let run_release = |release_ms: f32| -> f32 {
        let _ = post_params(
            &ctx,
            &serde_json::json!({
                "split.dyn_threshold_db": -25.0,
                "split.dyn_attack_ms": 1.0,
                "split.dyn_release_ms": release_ms
            }),
        );
        let mut ch = dsp.lock().unwrap();
        ch.drain_param_queue();
        let loud = sine_wave(440.0, 0.8, 48_000.0, 0, 4096);
        let mut tmp = vec![0.0f32; 4096];
        ch.process(&loud, &mut tmp);
        let tail = sine_wave(440.0, 0.01, 48_000.0, 4096, 2400); // 50 ms em sinal baixo
        let mut out_tail = vec![0.0f32; 2400];
        ch.process(&tail, &mut out_tail);
        rms(&out_tail[1440..])
    };
    let fast_rel_a = run_release(10.0);
    let slow_rel_a = run_release(500.0);
    assert!(
        fast_rel_a > slow_rel_a * 1.5,
        "AUD-20: dyn_release_ms=10ms deve retornar ao ramo A mais rápido que 500ms (fast_A={fast_rel_a:.5}, slow_A={slow_rel_a:.5})"
    );
}

#[test]
fn aud_20_crossover_lr4() {
    let sr = 48_000.0f32;
    let fc = 500.0f32;
    let mut split = SplitNode::new(SplitMode::Crossover { freq_hz: fc });
    split.set_sample_rate(sr);

    // 1. Soma |LP + HP| = 0 dB +- 0.1 dB de 20 Hz a 20 kHz
    let freqs = [20.0, 50.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 5000.0, 12000.0, 20000.0];
    let n = 8192usize;
    for &f in &freqs {
        let sig = sine_wave(f, 0.5, sr, 0, n);
        let mut out_lp = vec![0.0f32; n];
        let mut out_hp = vec![0.0f32; n];
        // Aquece filtros para f específico antes da medição estacionária
        split.process(&sig, &mut out_lp, &mut out_hp);
        split.process(&sig, &mut out_lp, &mut out_hp);

        let sum: Vec<f32> = out_lp.iter().zip(out_hp.iter()).map(|(a, b)| a + b).collect();
        let db = 20.0 * (rms(&sum[n / 2..]) / rms(&sig[n / 2..]).max(1e-9)).log10();
        assert!(
            db.abs() <= 0.1,
            "AUD-20 LR4: |LP + HP| em f={f} Hz deve ser 0 dB +- 0.1 dB, medido {db:.4} dB"
        );
    }

    // 2. Inclinação de -24 dB/oitava +- 1 em 4 * fc (comparado à banda passante e entre 2*fc e 4*fc)
    let mut measure_lp_db = |f: f32| -> f32 {
        let sig = sine_wave(f, 0.5, sr, 0, 16384);
        let mut out_lp = vec![0.0f32; 16384];
        let mut out_hp = vec![0.0f32; 16384];
        split.process(&sig, &mut out_lp, &mut out_hp);
        split.process(&sig, &mut out_lp, &mut out_hp);
        20.0 * (rms(&out_lp[8192..]) / rms(&sig[8192..]).max(1e-9)).log10()
    };

    let db_4fc = measure_lp_db(4.0 * fc);
    // Para LR4 teórico: em 4*fc (2 oitavas acima de fc), ganho = 1/(1+4^4) = -48.20 dB => -24.1 dB/oitava
    let slope_per_oct = db_4fc / 2.0;
    assert!(
        (slope_per_oct - (-24.0)).abs() <= 1.0,
        "AUD-20 LR4: inclinação média até 4*fc ({} Hz) deve ser -24 dB/oitava +- 1 dB, medido {slope_per_oct:.2} dB/oct (total {db_4fc:.2} dB)",
        4.0 * fc
    );
}
