//! Testes de contrato da Spec 038 — Gabinete: Convolução Particionada, Microfones e Ressonância

use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::cab::{ImpulseResponse, MicModel, VirtualCabProcessor, WavIrLoader};
use std::time::Instant;

fn make_wav_pcm16(sample_rate: u32, channels: u16, frames: usize, amp_fn: impl Fn(usize, u16) -> f32) -> Vec<u8> {
    let data_len = (frames * channels as usize * 2) as u32;
    let riff_len = 36 + data_len;
    let mut b = Vec::with_capacity(44 + data_len as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&riff_len.to_le_bytes());
    b.extend_from_slice(b"WAVE");
    b.extend_from_slice(b"fmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&channels.to_le_bytes());
    b.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * channels as u32 * 2;
    b.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = channels * 2;
    b.extend_from_slice(&block_align.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for i in 0..frames {
        for ch in 0..channels {
            let s = (amp_fn(i, ch).clamp(-1.0, 1.0) * 32767.0) as i16;
            b.extend_from_slice(&s.to_le_bytes());
        }
    }
    b
}

#[test]
fn ca1_48000_taps_96khz_ir_matches_f64_reference_under_minus_80db() {
    let sr = 96000.0f32;
    let ir_len = 48_000usize; // 500 ms @ 96 kHz
    let mut h = vec![0.0f32; ir_len];
    for (k, v) in h.iter_mut().enumerate() {
        let decay = (-6.0 * (k as f32 / ir_len as f32)).exp();
        *v = decay * (0.25 * (k as f32 * 0.13).sin() + 0.1 * (k as f32 * 0.47).cos());
    }
    h[0] = 0.5;
    h[40_000] = 0.15; // tap além de 32768!

    let ir = ImpulseResponse::from_samples(sr, h.clone());
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);
    cab.set_custom_ir(ir);
    cab.set_ir_enabled(true);
    cab.set_low_cut_hz(20.0);
    cab.set_hicut_khz(18.0);

    // Entrada multi-tom de 49152 amostras em blocos de 128
    let total = 49_152usize;
    let mut input = vec![0.0f32; total];
    for (i, s) in input.iter_mut().enumerate() {
        let t = i as f32 / sr;
        *s = 0.4 * (2.0 * std::f32::consts::PI * 220.0 * t).sin()
            + 0.25 * (2.0 * std::f32::consts::PI * 1350.0 * t).sin();
    }

    let mut out = vec![0.0f32; total];
    for idx in (0..total).step_by(128) {
        cab.process(&input[idx..idx + 128], &mut out[idx..idx + 128]);
    }

    // Referência offline f64 nos últimos 2048 pontos (onde taps > 32768 também contribuem)
    let mut max_err = 0.0f64;
    let mut max_ref = 0.0f64;
    for n in (total - 2048)..total {
        let mut ref_sum = 0.0f64;
        for k in 0..ir_len {
            if n >= k {
                ref_sum += (input[n - k] as f64) * (h[k] as f64);
            }
        }
        max_ref = max_ref.max(ref_sum.abs());
        max_err = max_err.max((out[n] as f64 - ref_sum).abs());
    }

    let err_db = 20.0 * (max_err / max_ref.max(1e-12)).log10();
    assert!(
        err_db <= -80.0,
        "CA-1: Erro vs referência f64 = {:.2} dB (esperado <= -80 dB)",
        err_db
    );
}

#[test]
fn ca2_zero_additional_latency_on_ir_convolution() {
    let sr = 48000.0f32;
    let mut h = vec![0.0f32; 4096];
    h[7] = 0.85; // primeiro tap não nulo no índice 7

    let ir = ImpulseResponse::from_samples(sr, h);
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);
    cab.set_custom_ir(ir);
    cab.set_ir_enabled(true);
    cab.set_low_cut_hz(20.0);
    cab.set_hicut_khz(18.0);

    let mut input = vec![0.0f32; 256];
    input[0] = 1.0;
    let mut output = vec![0.0f32; 256];
    cab.process(&input, &mut output);

    let first_nz = output.iter().position(|&v| v.abs() > 1e-4);
    assert_eq!(
        first_nz,
        Some(7),
        "CA-2: primeiro tap não nulo deveria aparecer no índice 7 (latência adicional = 0), obteve {:?}",
        first_nz
    );
}

#[test]
fn ca3_partitioned_convolution_is_under_15_percent_of_direct_fir() {
    let sr = 48000.0f32;
    let ir_len = 24_000usize;
    let mut h = vec![0.0f32; ir_len];
    for (k, v) in h.iter_mut().enumerate() {
        *v = 0.01 * (-5.0 * (k as f32 / ir_len as f32)).exp();
    }

    let ir = ImpulseResponse::from_samples(sr, h.clone());
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);
    cab.set_custom_ir(ir);
    cab.set_ir_enabled(true);

    let input = vec![0.25f32; 128];
    let mut output = vec![0.0f32; 128];

    // Warmup
    for _ in 0..8 {
        cab.process(&input, &mut output);
    }

    let iters = 200usize;
    let t0 = Instant::now();
    for _ in 0..iters {
        cab.process(&input, &mut output);
    }
    let part_ns = t0.elapsed().as_nanos() as f64 / iters as f64;

    // Referência FIR direto 24k taps por bloco de 128 (mesmas condições)
    let mut circ = vec![0.0f32; 32768];
    let mut pos = 0usize;
    let t1 = Instant::now();
    for _ in 0..iters {
        for i in 0..128 {
            circ[pos] = input[i];
            let mut sum = 0.0f32;
            for k in 0..ir_len {
                let r_idx = (pos + 32768 - k) & 32767;
                sum += circ[r_idx] * h[k];
            }
            output[i] = sum;
            pos = (pos + 1) & 32767;
        }
    }
    let fir_ns = t1.elapsed().as_nanos() as f64 / iters as f64;
    let ratio_pct = (part_ns / fir_ns.max(1.0)) * 100.0;
    eprintln!(
        "CA-3 BENCH: partitioned={:.0} ns/blk, direct_fir={:.0} ns/blk, ratio={:.2}%",
        part_ns, fir_ns, ratio_pct
    );
    assert!(
        ratio_pct <= 15.0,
        "CA-3: convolução particionada deve custar <= 15% do FIR direto (foi {:.2}%)",
        ratio_pct
    );
}

#[test]
fn ca4_mic_models_have_distinct_frequency_responses() {
    let sr = 48000.0f32;
    let measure_band_db = |model: MicModel, f_lo: f32, f_hi: f32| -> f32 {
        let mut cab = VirtualCabProcessor::new();
        cab.set_sample_rate(sr);
        cab.set_ir_enabled(false);
        cab.mic_slot_mut(0).model = model;
        cab.mic_slot_mut(0).position_cap_to_edge = 0.2;
        cab.mic_slot_mut(0).distance_cm = 2.0;

        // Varredura com soma de senoides na banda
        let n = 4096usize;
        let mut inp = vec![0.0f32; n];
        let freqs = [f_lo, (f_lo + f_hi) * 0.5, f_hi];
        for (i, s) in inp.iter_mut().enumerate() {
            let t = i as f32 / sr;
            let mut v = 0.0;
            for &f in &freqs {
                v += (2.0 * std::f32::consts::PI * f * t).sin();
            }
            *s = v * 0.1;
        }
        let mut out = vec![0.0f32; n];
        cab.process(&inp, &mut out);
        let rms: f32 = (out[1024..].iter().map(|&x| x * x).sum::<f32>() / (n - 1024) as f32).sqrt();
        20.0 * rms.max(1e-12).log10()
    };

    let sm57_hi = measure_band_db(MicModel::Sm57, 6000.0, 10000.0);
    let r121_hi = measure_band_db(MicModel::R121, 6000.0, 10000.0);
    assert!(
        sm57_hi - r121_hi >= 3.0,
        "CA-4: R121 (ribbon) em 6-10 kHz deve ser >= 3 dB mais escuro que SM57 (sm57={:.2} dB, r121={:.2} dB)",
        sm57_hi,
        r121_hi
    );

    let sm57_lowmid = measure_band_db(MicModel::Sm57, 280.0, 355.0);
    let md421_lowmid = measure_band_db(MicModel::Md421, 280.0, 355.0);
    let sm57_mid = measure_band_db(MicModel::Sm57, 2500.0, 3550.0);
    let md421_mid = measure_band_db(MicModel::Md421, 2500.0, 3550.0);
    let max_diff = (sm57_lowmid - md421_lowmid)
        .abs()
        .max((sm57_mid - md421_mid).abs());
    assert!(
        max_diff >= 1.0,
        "CA-4: MD421 deve diferir do SM57 em >= 1 dB (max_diff={:.2} dB)",
        max_diff
    );
}

#[test]
fn ca5_cab_resonance_hz_shifts_amp_cab_low_frequency_peak() {
    let sr = 48000.0f32;
    let find_peak_hz = |res_hz: f32| -> f32 {
        let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
        amp.set_sample_rate(sr);
        amp.panel_mut().set_input_gain(2.0);
        amp.panel_mut().set_master_volume(3.0);
        amp.mod_settings_mut().nfb = 1.0; // realça pico de ressonância
        amp.set_speaker_resonance_hz(res_hz);

        let mut cab = VirtualCabProcessor::new();
        cab.set_sample_rate(sr);
        cab.set_low_cut_hz(20.0);
        cab.set_resonance_hz(res_hz);

        let mut best_hz = 0.0f32;
        let mut best_rms = 0.0f32;
        for f in (55..=160).step_by(5) {
            let freq = f as f32;
            let n = 4096usize;
            let mut inp = vec![0.0f32; n];
            for (i, s) in inp.iter_mut().enumerate() {
                *s = 0.05 * (2.0 * std::f32::consts::PI * freq * (i as f32 / sr)).sin();
            }
            let mut amp_out = vec![0.0f32; n];
            let mut cab_out = vec![0.0f32; n];
            amp.process(&inp, &mut amp_out);
            cab.process(&amp_out, &mut cab_out);
            let rms = (cab_out[2048..].iter().map(|&x| x * x).sum::<f32>() / 2048.0).sqrt();
            if rms > best_rms {
                best_rms = rms;
                best_hz = freq;
            }
        }
        best_hz
    };

    let peak_70 = find_peak_hz(70.0);
    let peak_130 = find_peak_hz(130.0);
    assert!(
        peak_130 - peak_70 >= 30.0,
        "CA-5: mover cab_1.resonance_hz 70->130 Hz deve subir o pico em >= 30 Hz (peak_70={}, peak_130={})",
        peak_70,
        peak_130
    );
}

#[test]
fn ca8_and_ca9_resample_48k_to_44k1_and_reject_truncated_wav() {
    // CA-8: 48 kHz (4800 amostras = 100 ms) carregado em motor 44.1 kHz -> ~4410 amostras
    let wav_48k = make_wav_pcm16(48000, 1, 4800, |i, _| if i == 0 { 0.8 } else { 0.0 });
    let loader = WavIrLoader::new(44100.0);
    let ir = loader.load_from_bytes(&wav_48k).expect("WAV 48k valido falhou");
    let expected = (4800.0f32 * 44100.0 / 48000.0).round() as isize;
    assert!(
        (ir.samples().len() as isize - expected).abs() <= 1,
        "CA-8: esperado {} +-1 amostras, obteve {}",
        expected,
        ir.samples().len()
    );

    // CA-9: WAV truncado rejeitado com erro
    let truncated = &wav_48k[..30];
    assert!(loader.load_from_bytes(truncated).is_err());
}
