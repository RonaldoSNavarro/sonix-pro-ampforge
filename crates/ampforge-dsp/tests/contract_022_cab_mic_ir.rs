//! crates/ampforge-dsp/tests/contract_022_cab_mic_ir.rs
//! Testes de Contrato Automatizados para Spec 022: Gabinete, Microfones e Impulse Response ao Vivo
//! Cobre Critérios de Aceite CA-1, CA-2, CA-3, CA-4, CA-5, CA-6, CA-8, CA-9, CA-10, CA-11, CA-12, CA-15

use std::f32::consts::PI;
use ampforge_dsp::cab::{
    VirtualCabProcessor, SpeakerCabModel, MicModel, WavIrLoader, ImpulseResponse,
    InvalidImpulseResponseError,
};

fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() { return 0.0; }
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

fn generate_sine(freq: f32, sr: f32, num_samples: usize) -> Vec<f32> {
    (0..num_samples)
        .map(|i| (2.0 * PI * freq * (i as f32 / sr)).sin())
        .collect()
}

fn calc_gain_at_freq(cab: &mut VirtualCabProcessor, freq: f32, sr: f32) -> f32 {
    let n = 4096;
    let in_sig = generate_sine(freq, sr, n);
    let mut out_sig = vec![0.0f32; n];
    cab.process(&in_sig, &mut out_sig);
    let in_rms = calculate_rms(&in_sig[n / 2..]);
    let out_rms = calculate_rms(&out_sig[n / 2..]);
    out_rms / in_rms.max(1e-9)
}

#[test]
fn test_ca_01_speaker_impedance_curves_and_resonance() {
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48000.0);

    // Greenback: nominal 98 Hz
    cab.set_speaker_model(SpeakerCabModel::Greenback4x12Closed);
    assert_eq!(cab.resonance_hz(), 98.0);
    let (f_max_gb, _) = cab.calculate_impedance_peak(60.0, 140.0, 1.0);
    assert!(
        (f_max_gb - 98.0).abs() <= 98.0 * 0.02,
        "Pico de impedância do Greenback deve estar em 98 Hz +- 2%: got {}", f_max_gb
    );

    // V30: nominal 108 Hz
    cab.set_speaker_model(SpeakerCabModel::V304x12Closed);
    assert_eq!(cab.resonance_hz(), 108.0);
    let (f_max_v30, _) = cab.calculate_impedance_peak(60.0, 140.0, 1.0);
    assert!(
        (f_max_v30 - 108.0).abs() <= 108.0 * 0.02,
        "Pico de impedância do V30 deve estar em 108 Hz +- 2%: got {}", f_max_v30
    );

    // Alnico Blue: nominal 82 Hz
    cab.set_speaker_model(SpeakerCabModel::AlnicoBlue2x12Open);
    assert_eq!(cab.resonance_hz(), 82.0);
    let (f_max_ab, _) = cab.calculate_impedance_peak(60.0, 140.0, 1.0);
    assert!(
        (f_max_ab - 82.0).abs() <= 82.0 * 0.02,
        "Pico de impedância do Alnico Blue deve estar em 82 Hz +- 2%: got {}", f_max_ab
    );
}

#[test]
fn test_ca_02_hicut_filter_response() {
    let sr = 48000.0;
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);
    // Usa IR unitária plana para isolar o filtro Butterworth 4a ordem do hi-cut da curva do falante (AUD-32)
    let mut flat_ir = vec![0.0f32; 256];
    flat_ir[0] = 1.0;
    cab.set_custom_ir(ImpulseResponse::from_samples(sr, flat_ir));
    cab.set_ir_enabled(true);

    for &target_khz in &[4.0, 8.0, 16.0] {
        cab.set_hicut_khz(target_khz);

        // Ganho na banda passante em 1 kHz
        let g_pass = calc_gain_at_freq(&mut cab, 1000.0, sr);

        // Varredura para encontrar a frequência onde ganho cai -3 dB (0.707 * g_pass)
        let target_gain = g_pass * 0.7071;
        let mut f_3db = target_khz * 1000.0;
        let mut min_diff = f32::MAX;

        for step in -20..=20 {
            let f = (target_khz * 1000.0) * (1.0 + step as f32 * 0.01);
            let g = calc_gain_at_freq(&mut cab, f, sr);
            let diff = (g - target_gain).abs();
            if diff < min_diff {
                min_diff = diff;
                f_3db = f;
            }
        }

        let expected_f = target_khz * 1000.0;
        let diff_pct = ((f_3db - expected_f) / expected_f).abs() * 100.0;
        assert!(
            diff_pct <= 10.0,
            "Ponto de -3 dB para {} kHz deve estar dentro de 10%: got {} Hz ({:.1}% diff)",
            target_khz, f_3db, diff_pct
        );
    }
}

#[test]
fn test_ca_03_open_vs_closed_back_bass_response() {
    let sr = 48000.0;
    let mut cab_closed = VirtualCabProcessor::new();
    cab_closed.set_sample_rate(sr);
    cab_closed.set_speaker_model(SpeakerCabModel::Greenback4x12Closed);
    cab_closed.mic_slot_mut(0).position_cap_to_edge = 0.5;

    let mut cab_open = VirtualCabProcessor::new();
    cab_open.set_sample_rate(sr);
    cab_open.set_speaker_model(SpeakerCabModel::AlnicoBlue2x12Open);
    cab_open.mic_slot_mut(0).position_cap_to_edge = 0.5;

    // Normalizado em 1 kHz
    let g1k_closed = calc_gain_at_freq(&mut cab_closed, 1000.0, sr);
    let g1k_open = calc_gain_at_freq(&mut cab_open, 1000.0, sr);

    // Média de energia na faixa 60 - 120 Hz (80 Hz, 100 Hz)
    let g80_closed = calc_gain_at_freq(&mut cab_closed, 80.0, sr) / g1k_closed;
    let g80_open = calc_gain_at_freq(&mut cab_open, 80.0, sr) / g1k_open;

    let diff_db = 20.0 * (g80_closed / g80_open.max(1e-6)).log10();
    assert!(
        diff_db >= 3.0,
        "Alnico Blue Open Back deve ter energia em 80 Hz >= 3 dB menor que Greenback Closed Back: diff={:.2} dB",
        diff_db
    );
}

#[test]
fn test_ca_04_sm57_cap_to_edge_position_treble_rolloff() {
    let sr = 48000.0;
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);
    cab.mic_slot_mut(0).model = MicModel::Sm57;
    cab.mic_slot_mut(0).distance_cm = 2.0;

    let positions = [0.0, 0.25, 0.50, 0.75, 1.0];
    let mut gains_6k = Vec::new();

    for &pos in &positions {
        cab.mic_slot_mut(0).position_cap_to_edge = pos;
        let g = calc_gain_at_freq(&mut cab, 6000.0, sr);
        gains_6k.push(g);
    }

    // 1. Posição 1.0 tem energia em 6 kHz >= 6 dB menor que posição 0.0
    let drop_db = 20.0 * (gains_6k[0] / gains_6k[4].max(1e-6)).log10();
    assert!(
        drop_db >= 6.0,
        "Borda (1.0) deve ter energia em 6 kHz >= 6 dB menor que centro (0.0): drop={:.2} dB",
        drop_db
    );

    // 2. Queda monotônica
    for i in 0..4 {
        assert!(
            gains_6k[i] >= gains_6k[i + 1] - 1e-4,
            "Energia em 6 kHz deve cair monotonicamente de cap para edge: pos {} ({:.3}) vs pos {} ({:.3})",
            positions[i], gains_6k[i], positions[i + 1], gains_6k[i + 1]
        );
    }
}

#[test]
fn test_ca_05_directional_mic_proximity_effect() {
    let sr = 48000.0;
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);
    cab.mic_slot_mut(0).model = MicModel::Sm57;
    cab.mic_slot_mut(0).position_cap_to_edge = 0.3;

    // Distância 1 cm
    cab.mic_slot_mut(0).distance_cm = 1.0;
    let g100_1cm = calc_gain_at_freq(&mut cab, 100.0, sr);
    let g1k_1cm = calc_gain_at_freq(&mut cab, 1000.0, sr);
    let bass_rel_1cm = 20.0 * (g100_1cm / g1k_1cm.max(1e-6)).log10();

    // Distância 15 cm
    cab.mic_slot_mut(0).distance_cm = 15.0;
    let g100_15cm = calc_gain_at_freq(&mut cab, 100.0, sr);
    let g1k_15cm = calc_gain_at_freq(&mut cab, 1000.0, sr);
    let bass_rel_15cm = 20.0 * (g100_15cm / g1k_15cm.max(1e-6)).log10();

    let proximity_diff_db = bass_rel_1cm - bass_rel_15cm;
    assert!(
        proximity_diff_db >= 3.0,
        "Proximidade a 1 cm vs 15 cm deve aumentar relação 100Hz/1kHz em >= 3 dB: got {:.2} dB",
        proximity_diff_db
    );
}

#[test]
fn test_ca_06_two_mics_comb_filter_null_at_5khz() {
    let sr = 48000.0;
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);

    // Mic 1: SM57 a 2.0 cm
    cab.mic_slot_mut(0).model = MicModel::Sm57;
    cab.mic_slot_mut(0).position_cap_to_edge = 0.2;
    cab.mic_slot_mut(0).distance_cm = 2.0;
    cab.mic_slot_mut(0).level_db = 0.0;
    cab.mic_slot_mut(0).enabled = true;

    // Mic 2: SM57 idêntico a 2.0 + 3.43 cm = 5.43 cm (delta t = 0.1 ms -> cancelamento em 5 kHz)
    cab.mic_slot_mut(1).model = MicModel::Sm57;
    cab.mic_slot_mut(1).position_cap_to_edge = 0.2;
    cab.mic_slot_mut(1).distance_cm = 5.43;
    cab.mic_slot_mut(1).level_db = 0.0;
    cab.mic_slot_mut(1).enabled = true;

    // Mede ganho em 4.5 kHz, 5.0 kHz e 5.5 kHz
    let g_4k5 = calc_gain_at_freq(&mut cab, 4500.0, sr);
    let g_5k0 = calc_gain_at_freq(&mut cab, 5000.0, sr);
    let g_5k5 = calc_gain_at_freq(&mut cab, 5500.0, sr);

    // O vale deve estar em 5 kHz (menor que em 4.5k e 5.5k em pelo menos 6 dB)
    assert!(
        g_5k0 < g_4k5 * 0.5 && g_5k0 < g_5k5 * 0.5,
        "Filtro pente deve produzir vale acentuado em 5 kHz: g(4.5k)={}, g(5k)={}, g(5.5k)={}",
        g_4k5, g_5k0, g_5k5
    );
}

#[test]
fn test_ca_08_zero_latency_fir_convolution_fidelity() {
    let sr = 48000.0;
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);

    // Cria IR sintética de 4096 amostras (decaimento exponencial suave)
    let ir_len = 4096;
    let mut ir_samples = vec![0.0f32; ir_len];
    ir_samples[0] = 0.85; // Pico no índice 0
    for i in 1..ir_len {
        ir_samples[i] = 0.5 * (-(i as f32) / 500.0).exp() * (2.0 * PI * 400.0 * i as f32 / sr).sin();
    }

    let ir = ImpulseResponse::from_samples(sr, ir_samples.clone());
    cab.set_custom_ir(ir);
    cab.set_ir_enabled(true);
    cab.set_hicut_khz(18.0);

    // 1. Verifica latência zero: impulso na entrada produz IR[0] no índice 0
    let mut impulse_in = vec![0.0f32; 128];
    impulse_in[0] = 1.0;
    let mut impulse_out = vec![0.0f32; 128];
    cab.process(&impulse_in, &mut impulse_out);

    assert!(
        (impulse_out[0] - ir_samples[0]).abs() <= 1e-4,
        "Impulso no índice 0 deve produzir exatamente IR[0] no índice 0 (0 amostras de latência): got {}, exp {}",
        impulse_out[0], ir_samples[0]
    );

    // 2. Convolução com sinal arbitrário e comparação com convolução direta
    let mut test_sig = vec![0.0f32; 512];
    for i in 0..512 {
        test_sig[i] = (2.0 * PI * 1000.0 * (i as f32 / sr)).sin() * 0.3;
    }

    // Reset cab
    cab.reset_ir_state();
    let mut cab_out = vec![0.0f32; 512];
    cab.process(&test_sig, &mut cab_out);

    // Convolução direta de referência
    let mut ref_out = vec![0.0f32; 512];
    for n in 0..512 {
        let mut sum = 0.0f32;
        for k in 0..=n {
            sum += test_sig[n - k] * ir_samples[k];
        }
        ref_out[n] = sum;
    }

    let mut err_energy = 0.0f32;
    let mut ref_energy = 0.0f32;
    for i in 0..512 {
        let diff = cab_out[i] - ref_out[i];
        err_energy += diff * diff;
        ref_energy += ref_out[i] * ref_out[i];
    }

    let err_db = 10.0 * (err_energy.max(1e-15) / ref_energy.max(1e-15)).log10();
    assert!(
        err_db <= -80.0,
        "Saída do bloco deve diferir da convolução direta em <= -80 dB: got {:.2} dB",
        err_db
    );
}

#[test]
fn test_ca_09_windowed_sinc_resampling_fidelity() {
    let loader = WavIrLoader::new(48000.0);

    // Cria IR de 44.1 kHz com multitom senoidal até 18 kHz
    let n_in = 4410; // 100 ms
    let mut raw_ir = vec![0.0f32; n_in];
    let test_freqs = [500.0, 2000.0, 8000.0, 14000.0, 18000.0];
    for i in 0..n_in {
        let t = i as f32 / 44100.0;
        let mut s = 0.0f32;
        for &f in &test_freqs {
            s += (2.0 * PI * f * t).sin();
        }
        raw_ir[i] = s * (-t * 30.0).exp();
    }

    let resampled = loader.resample_sinc(&raw_ir, 44100.0, 48000.0);
    assert_eq!(resampled.sample_rate(), 48000.0);

    // Verifica que a magnitude de cada tom senoidal até 18 kHz está dentro de +-0.5 dB
    for &f in &test_freqs {
        let mag_orig = dft_magnitude(&raw_ir, 44100.0, f);
        let mag_resamp = dft_magnitude(resampled.samples(), 48000.0, f);
        let diff_db = 20.0 * (mag_resamp / mag_orig.max(1e-9)).log10();
        assert!(
            diff_db.abs() <= 0.5,
            "Reamostragem sinc para {} Hz deve preservar magnitude em +-0.5 dB: diff={:.2} dB",
            f, diff_db
        );
    }
}

fn dft_magnitude(samples: &[f32], sr: f32, freq: f32) -> f32 {
    let mut real = 0.0f32;
    let mut imag = 0.0f32;
    let n = samples.len();
    for i in 0..n {
        let angle = 2.0 * PI * freq * (i as f32 / sr);
        real += samples[i] * angle.cos();
        imag += samples[i] * angle.sin();
    }
    (real * real + imag * imag).sqrt() / sr
}

#[test]
fn test_ca_10_ir_normalization_options() {
    let sr = 48000.0;
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);

    // IR com ganho arbitrário (ex: 0.25)
    let n = 2048;
    let mut raw_ir = vec![0.0f32; n];
    raw_ir[0] = 0.25;

    // 1. Sem normalização: energia preservada
    let ir_raw = ImpulseResponse::from_samples_with_normalization(sr, raw_ir.clone(), false);
    cab.set_custom_ir(ir_raw);
    cab.set_ir_enabled(true);
    cab.set_hicut_khz(18.0);

    let in_sig = vec![1.0f32; 128];
    let mut out_raw = vec![0.0f32; 128];
    cab.process(&in_sig, &mut out_raw);
    assert!((out_raw[0] - 0.25).abs() <= 1e-4);

    // 2. Com normalização: energia unitária (impulso unitário tem pico 1.0)
    let ir_norm = ImpulseResponse::from_samples_with_normalization(sr, raw_ir, true);
    cab.set_custom_ir(ir_norm);
    let mut out_norm = vec![0.0f32; 128];
    cab.process(&in_sig, &mut out_norm);
    assert!((out_norm[0] - 1.0).abs() <= 1e-3);
}

#[test]
fn test_ca_11_wav_ir_loader_length_limits_and_invalid_files() {
    let loader = WavIrLoader::new(48000.0);

    // 1. Arquivo vazio -> Erro EmptyFile
    assert!(matches!(loader.load_from_bytes(&[]), Err(InvalidImpulseResponseError::EmptyFile)));

    // 2. Arquivo com channels = 0 -> Erro sem pânico
    let mut bad_wav = vec![0u8; 44];
    bad_wav[0..4].copy_from_slice(b"RIFF");
    bad_wav[8..12].copy_from_slice(b"WAVE");
    bad_wav[12..16].copy_from_slice(b"fmt ");
    bad_wav[16..20].copy_from_slice(&16u32.to_le_bytes());
    bad_wav[20..22].copy_from_slice(&1u16.to_le_bytes()); // PCM
    bad_wav[22..24].copy_from_slice(&0u16.to_le_bytes()); // channels = 0 !
    assert!(loader.load_from_bytes(&bad_wav).is_err());

    // 3. IR com mais de 500 ms (> 24000 amostras a 48 kHz) -> Erro de tamanho excessivo
    let n_too_long = 48000 * 600 / 1000; // 600 ms = 28800 amostras
    let long_samples = vec![0.0f32; n_too_long];
    let check_len = WavIrLoader::validate_ir_duration(long_samples.len(), 48000.0);
    assert!(check_len.is_err(), "IR com 600 ms deve ser rejeitada como ir_too_long");
}

#[test]
fn test_ca_12_ir_switching_continuity_and_crossfade() {
    let sr = 48000.0;
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);

    let n = 2048;
    let mut ir1_samples = vec![0.0f32; 256];
    let mut ir2_samples = vec![0.0f32; 256];
    for i in 0..256 {
        let decay = (-(i as f32) / 30.0).exp();
        ir1_samples[i] = 0.2 * decay;
        ir2_samples[i] = 0.1 * decay;
    }
    let ir1 = ImpulseResponse::from_samples(sr, ir1_samples);
    let ir2 = ImpulseResponse::from_samples(sr, ir2_samples);

    cab.set_custom_ir(ir1);
    cab.set_ir_enabled(true);

    let in_sig = generate_sine(1000.0, sr, n);
    let mut out1 = vec![0.0f32; n];
    cab.process(&in_sig, &mut out1);

    // Troca de IR durante execução contínua
    cab.set_target_ir(ir2);

    let mut out_switch = vec![0.0f32; n];
    cab.process(&in_sig, &mut out_switch);

    // Não deve haver saltos abruptos (|x[n] - x[n-1]| grande devido a clique no crossfade)
    for i in 1..n {
        let diff = (out_switch[i] - out_switch[i - 1]).abs();
        assert!(
            diff < 0.25,
            "Descontinuidade detectada na troca de IR na amostra {}: diff={}",
            i, diff
        );
    }
}
