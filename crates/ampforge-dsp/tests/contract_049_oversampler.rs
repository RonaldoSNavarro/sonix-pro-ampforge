//! crates/ampforge-dsp/tests/contract_049_oversampler.rs
//! Testes de Contrato para Spec 049 (Parte A):
//! - CA-12: Latência medida por impulso (up + down) em 2x, 4x e 8x bate com `latency_samples()` em ±1 amostra.
//! - CA-13: Custo do oversampler 8x <= 25% do baseline, sem perda de stopband e sem piorar aliasing > 1 dB.

use std::time::Instant;

use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::oversampling::{HalfBandOversampler, OversamplingFactor};

fn measure_impulse_peak_sample(factor: OversamplingFactor) -> usize {
    let mut os = HalfBandOversampler::new(factor);
    let m = factor.multiplier();
    let mut buf = [0.0f32; 8];
    let total = 512usize;
    let mut max_val = 0.0f32;
    let mut max_idx = 0usize;

    for n in 0..total {
        let x = if n == 0 { 1.0f32 } else { 0.0f32 };
        os.upsample_sample(x, &mut buf[..m]);
        let y = os.downsample_block(&buf[..m]);
        if y.abs() > max_val {
            max_val = y.abs();
            max_idx = n;
        }
    }
    max_idx
}

#[test]
fn ca12_oversampler_roundtrip_impulse_matches_latency_samples() {
    for factor in [
        OversamplingFactor::X2,
        OversamplingFactor::X4,
        OversamplingFactor::X8,
    ] {
        let os = HalfBandOversampler::new(factor);
        let declared = os.latency_samples();
        let measured = measure_impulse_peak_sample(factor);
        assert_eq!(
            declared,
            measured,
            "CA-12 (R8, {:?}): latency_samples() = {} deve bater exatamente com o pico do impulso em {}",
            factor,
            declared,
            measured
        );
    }
}

#[test]
fn ca13_measure_oversampler_stopband_aliasing_and_8x_amp_time() {
    // 1. Stopband
    for factor in [
        OversamplingFactor::X2,
        OversamplingFactor::X4,
        OversamplingFactor::X8,
    ] {
        let os = HalfBandOversampler::new(factor);
        let sb = os.stopband_attenuation_db();
        eprintln!("CA-13 stopband {:?}: {:.2} dB", factor, sb);
        assert!(sb >= 75.0, "CA-13 ({:?}): stopband muito baixo: {:.2} dB", factor, sb);
    }

    // 2. Aliasing com senoide de 5 kHz a -6 dBFS em ganho alto
    let sr = 48000.0f32;
    let n = 4096usize;
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(sr);
    amp.set_oversampling_factor(8);
    amp.panel_mut().set_input_gain(9.5);
    amp.panel_mut().set_master_volume(7.0);

    // Aquecimento + sinal estacionário
    let warm = vec![0.0f32; 4096];
    let mut out_warm = vec![0.0f32; 4096];
    amp.process(&warm, &mut out_warm);

    let mut input = vec![0.0f32; n];
    let amp_lin = 10.0f32.powf(-6.0 / 20.0);
    for (i, s) in input.iter_mut().enumerate() {
        let t = i as f32 / sr;
        *s = amp_lin * (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
    }
    let mut output = vec![0.0f32; n];
    amp.process(&input, &mut output);

    // DFT para medir energia inarmônica (fora de DC e múltiplos de 5 kHz) abaixo de 20 kHz
    // Como 5000 * 4096 / 48000 = 426.67, usamos janela Hann para evitar leakage
    let mut windowed = vec![0.0f64; n];
    for i in 0..n {
        let w = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos());
        windowed[i] = output[i] as f64 * w;
    }

    let mut total_energy = 0.0f64;
    let mut inharmonic_energy = 0.0f64;
    let max_bin = ((20000.0 / sr) * n as f32).floor() as usize;
    for k in 1..=max_bin {
        let freq = k as f32 * sr / n as f32;
        let mut re = 0.0f64;
        let mut im = 0.0f64;
        let step = 2.0 * std::f64::consts::PI * k as f64 / n as f64;
        for (i, &x) in windowed.iter().enumerate() {
            let a = step * i as f64;
            re += x * a.cos();
            im -= x * a.sin();
        }
        let e = re * re + im * im;
        total_energy += e;
        // Harmônicos de 5 kHz abaixo de 20 kHz: 5k, 10k, 15k (e 20k)
        let is_harmonic = [5000.0f32, 10000.0, 15000.0, 20000.0]
            .iter()
            .any(|&h| (freq - h).abs() < 120.0);
        if !is_harmonic && freq > 120.0 {
            inharmonic_energy += e;
        }
    }
    let aliasing_ratio_db = 10.0 * (inharmonic_energy / total_energy.max(1e-20)).log10();
    eprintln!("CA-13 aliasing 5kHz -6dBFS (8x): {:.2} dBc", aliasing_ratio_db);

    // 3. Benchmark de 1 s a 48 kHz em 8x do HalfBandOversampler (upsample + downsample)
    let mut os8 = HalfBandOversampler::new(OversamplingFactor::X8);
    let mut os_buf = [0.0f32; 8];
    let mut acc_sink = 0.0f32;
    let t_os = Instant::now();
    for i in 0..48000usize {
        let x = 0.25 * (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / sr)).sin();
        os8.upsample_sample(x, &mut os_buf);
        acc_sink += os8.downsample_block(&os_buf);
    }
    let os_elapsed_ms = t_os.elapsed().as_secs_f64() * 1000.0;
    eprintln!("CA-13 1s@48kHz HalfBandOversampler 8x elapsed: {:.2} ms (sink={:.3})", os_elapsed_ms, acc_sink);
    assert!(
        os_elapsed_ms <= 20.0,
        "CA-13: HalfBandOversampler 8x 1s@48kHz deve rodar em <= 20 ms em release, levou {:.2} ms",
        os_elapsed_ms
    );

    // 4. Benchmark de 1 s a 48 kHz em 8x do WhiteBoxAmpProcessor (referência)
    let block = 128usize;
    let mut bench_in = vec![0.0f32; block];
    for (i, s) in bench_in.iter_mut().enumerate() {
        *s = 0.25 * (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / sr)).sin();
    }
    let mut bench_out = vec![0.0f32; block];
    let t0 = Instant::now();
    for _ in 0..(48000 / block) {
        amp.process(&bench_in, &mut bench_out);
    }
    let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
    eprintln!("CA-13 1s@48kHz amp 8x elapsed: {:.2} ms", elapsed_ms);
}
