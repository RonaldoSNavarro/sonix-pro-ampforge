//! crates/ampforge-dsp/tests/contract_031_authentic_amps.rs
//! Testes de Contrato TDD para Spec 031: Amplificadores Fiéis aos Esquemas Originais
//! Cobre CA-1..CA-7, CA-9, CA-10

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::f32::consts::PI;

use ampforge_dsp::amp::{
    AmpChannelMode, AmpInputJack, AmpModelId, WhiteBoxAmpProcessor,
};
use ampforge_dsp::modshop::{PowerTube, PreampTube, RectifierType, ToneStackTopology};
use ampforge_dsp::tonestack::{analog_tonestack_magnitude_db, PassiveToneStack};

// ─── Zero-Alloc Guard para CA-9 ─────────────────────────────────────────────────────────────
struct CountingAllocator;
thread_local! {
    static COUNT_ALLOCS: Cell<bool> = const { Cell::new(false) };
    static ALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        COUNT_ALLOCS.with(|c| {
            if c.get() {
                ALLOC_COUNT.with(|a| a.set(a.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt()
}

fn goertzel_mag(samples: &[f32], sample_rate: f64, freq_hz: f64) -> f64 {
    let n = samples.len();
    let omega = 2.0 * std::f64::consts::PI * freq_hz / sample_rate;
    let coeff = 2.0 * omega.cos();
    let mut s_prev = 0.0f64;
    let mut s_prev2 = 0.0f64;
    for (i, &x) in samples.iter().enumerate() {
        let hann = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos());
        let s = (x as f64) * hann + coeff * s_prev - s_prev2;
        s_prev2 = s_prev;
        s_prev = s;
    }
    let power = s_prev2 * s_prev2 + s_prev * s_prev - coeff * s_prev * s_prev2;
    (power.max(0.0).sqrt()) / (n as f64 * 0.25)
}

fn measure_filter_db(
    ts: &mut PassiveToneStack,
    topo: ToneStackTopology,
    b: f32,
    m: f32,
    t: f32,
    sr: f64,
    freq: f64,
) -> f64 {
    ts.update(topo, b, m, t, sr);
    ts.reset();
    let total = 4096usize;
    let settle = 2048usize;
    let mut out = vec![0.0f32; total];
    for (i, y) in out.iter_mut().enumerate() {
        let x = (2.0 * std::f64::consts::PI * freq * i as f64 / sr).sin();
        *y = ts.process(x) as f32;
    }
    let steady = &out[settle..];
    let mag = goertzel_mag(steady, sr, freq);
    20.0 * mag.max(1e-9).log10()
}

#[test]
fn test_ca_01_passive_tonestack_matches_analog_curves_and_interacts() {
    let sr = 96000.0; // oversampled internal rate (e.g. 48k * 2 or 48k * 4 = 192k)
    let mut ts = PassiveToneStack::new();
    let freqs = [60.0, 120.0, 350.0, 650.0, 1200.0, 3000.0, 6000.0, 9000.0];

    for &(topo, b, m, t) in &[
        (ToneStackTopology::MarshallCathodeFollower, 5.0, 5.0, 5.0),
        (ToneStackTopology::MarshallCathodeFollower, 8.0, 2.0, 8.0),
        (ToneStackTopology::FenderTmbMidScoop, 6.0, 4.0, 7.0),
        (ToneStackTopology::VoxTopBoost, 7.0, 5.0, 7.0),
        (ToneStackTopology::MesaMarkPreGain, 7.0, 3.0, 7.5),
    ] {
        for &f in &freqs {
            let ref_db = analog_tonestack_magnitude_db(topo, b, m, t, f);
            let dig_db = measure_filter_db(&mut ts, topo, b, m, t, sr, f);
            let err = (dig_db - ref_db).abs();
            assert!(
                err <= 1.0,
                "ToneStack {:?} (B={}, M={}, T={}) @ {} Hz: digital={:.2} dB vs analog={:.2} dB (erro={:.2} dB > 1.0 dB)",
                topo, b, m, t, f, dig_db, ref_db, err
            );
        }
    }

    // Interação real Bass/Middle/Treble: alterar Middle em Marshall desloca a resposta em 400 Hz (+2,35 dB conforme Yeh & Smith)
    let gain_treb_mid0 = measure_filter_db(&mut ts, ToneStackTopology::MarshallCathodeFollower, 5.0, 0.0, 5.0, sr, 400.0);
    let gain_treb_mid10 = measure_filter_db(&mut ts, ToneStackTopology::MarshallCathodeFollower, 5.0, 10.0, 5.0, sr, 400.0);
    assert!(
        (gain_treb_mid10 - gain_treb_mid0).abs() >= 1.5,
        "Malha passiva FMV deve exibir interação forte entre Middle e graves/médio-graves (diff={:.2} dB)",
        (gain_treb_mid10 - gain_treb_mid0).abs()
    );
}

#[test]
fn test_ca_02_plexi_1959_volume_2_and_jumpered_inputs() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritPlexi1959);
    amp.set_sample_rate(48000.0);
    amp.set_input_jack(AmpInputJack::Jumpered);
    amp.panel_mut().set_input_gain(2.0);
    amp.panel_mut().set_volume_2(0.0);

    let n = 4096;
    let input: Vec<f32> = (0..n)
        .map(|i| 0.05 * (2.0 * PI * 140.0 * (i as f32 / 48000.0)).sin())
        .collect();
    let mut out_v2_0 = vec![0.0f32; n];
    amp.process(&input, &mut out_v2_0);
    let rms_0 = rms(&out_v2_0[1024..]);

    amp.panel_mut().set_volume_2(10.0);
    let mut out_v2_10 = vec![0.0f32; n];
    amp.process(&input, &mut out_v2_10);
    let rms_10 = rms(&out_v2_10[1024..]);

    let diff_db = 20.0 * (rms_10 / rms_0.max(1e-7)).log10();
    assert!(
        diff_db >= 3.0,
        "Plexi 1959 jumpered: volume_2 0 -> 10 com gain=2 deve elevar RMS >= 3 dB (foi {:.2} dB)",
        diff_db
    );
}

#[test]
fn test_ca_03_ac30_tone_cut_and_zero_nfb() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::UkAc30TopBoost);
    amp.set_sample_rate(48000.0);
    assert!(
        amp.mod_settings().nfb <= 0.01,
        "AC30 Top Boost deve iniciar com NFB global = 0 (foi {})",
        amp.mod_settings().nfb
    );

    let n = 4096;
    let input: Vec<f32> = (0..n)
        .map(|i| 0.04 * (2.0 * PI * 3200.0 * (i as f32 / 48000.0)).sin())
        .collect();

    amp.panel_mut().set_cut(0.0);
    let mut out_cut0 = vec![0.0f32; n];
    amp.process(&input, &mut out_cut0);
    let rms_cut0 = rms(&out_cut0[1024..]);

    amp.panel_mut().set_cut(10.0);
    let mut out_cut10 = vec![0.0f32; n];
    amp.process(&input, &mut out_cut10);
    let rms_cut10 = rms(&out_cut10[1024..]);

    let atten_db = 20.0 * (rms_cut0 / rms_cut10.max(1e-7)).log10();
    assert!(
        atten_db >= 6.0,
        "AC30 Tone Cut 0 -> 10 deve atenuar >2 kHz em >= 6 dB (foi {:.2} dB)",
        atten_db
    );
}

#[test]
fn test_ca_04_jcm800_cold_clipper_even_harmonics_and_low_input_pad() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);
    amp.panel_mut().set_input_gain(8.0);
    amp.panel_mut().set_master_volume(5.0);

    let n = 4096;
    let freq = 440.0;
    let input: Vec<f32> = (0..n)
        .map(|i| 0.08 * (2.0 * PI * freq * (i as f32 / 48000.0)).sin())
        .collect();
    let mut out = vec![0.0f32; n];
    amp.process(&input, &mut out);

    let f1 = goertzel_mag(&out[1024..], 48000.0, freq as f64);
    let f2 = goertzel_mag(&out[1024..], 48000.0, (freq * 2.0) as f64);
    let h2_dbc = 20.0 * (f2 / f1.max(1e-9)).log10();
    assert!(
        h2_dbc > -28.0,
        "JCM800 com gain=8 deve gerar harmônico par 2f0 expressivo pelo cold clipper V1b (2f0={:.2} dBc)",
        h2_dbc
    );

    // Input LOW vs HIGH em regime linear (1 kHz, -80 dBFS): LOW pula V1a + divisor 68k/68k
    let n_lin = 16_384;
    let mut amp_clean = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_clean.set_sample_rate(48000.0);
    amp_clean.panel_mut().set_input_gain(5.0);
    let small_in: Vec<f32> = (0..n_lin)
        .map(|i| 0.0001 * (2.0 * PI * 1000.0 * (i as f32 / 48000.0)).sin())
        .collect();
    amp_clean.set_input_jack(AmpInputJack::High);
    let mut out_hi = vec![0.0f32; n_lin];
    amp_clean.process(&small_in, &mut out_hi);
    let mag_hi = goertzel_mag(&out_hi[8192..], 48000.0, 1000.0).max(1e-12);

    amp_clean.set_input_jack(AmpInputJack::Low);
    let mut out_lo = vec![0.0f32; n_lin];
    amp_clean.process(&small_in, &mut out_lo);
    let mag_lo = goertzel_mag(&out_lo[8192..], 48000.0, 1000.0).max(1e-12);

    let diff_db = 20.0 * (mag_hi / mag_lo).log10();
    let expected_diff_db = 20.0 * amp_clean.ganho_estagio1_pequeno_sinal().log10() + 6.0206;
    assert!(
        (diff_db - expected_diff_db).abs() <= 1.0,
        "JCM800 HIGH - LOW @ 1 kHz (-80 dBFS) deve igualar 20*log10(ganho_estagio1)+6,02 dB ({:.2} dB vs {:.2} dB)",
        diff_db,
        expected_diff_db
    );
}

#[test]
fn test_ca_05_dual_rect_channel_modes_and_spongy_variac() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::CaliDualRectoRevF);
    amp.set_sample_rate(48000.0);
    amp.panel_mut().set_input_gain(6.5);
    amp.panel_mut().set_master_volume(5.0);

    let n = 4096;
    let in_sig: Vec<f32> = (0..n)
        .map(|i| 0.08 * (2.0 * PI * 1000.0 * (i as f32 / 48000.0)).sin())
        .collect();

    amp.set_channel_mode(AmpChannelMode::Clean);
    let mut out_clean = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_clean);
    let thd_clean = amp.compute_thd(&out_clean[2048..]);

    amp.set_channel_mode(AmpChannelMode::Modern);
    let mut out_mod = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_mod);
    let thd_mod = amp.compute_thd(&out_mod[2048..]);

    assert!(
        thd_mod > 3.0 * thd_clean.max(0.005),
        "Dual Rectifier Modern deve ter THD > 3x Clean (clean={:.4}, modern={:.4})",
        thd_clean, thd_mod
    );

    // Bold vs Spongy variac + rectifier
    amp.set_channel_mode(AmpChannelMode::Vintage);
    amp.set_rectifier_type(RectifierType::Dual5U4Gb);
    amp.set_spongy_power(false);
    let mut out_bold = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_bold);
    let rms_bold = rms(&out_bold[2048..]);

    amp.set_spongy_power(true);
    let mut out_spongy = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_spongy);
    let rms_spongy = rms(&out_spongy[2048..]);

    assert!(
        rms_spongy < rms_bold * 0.94,
        "Spongy deve reduzir tensão B+ e saída antes do sag em relação a Bold (bold={:.4}, spongy={:.4})",
        rms_bold, rms_spongy
    );
}

#[test]
fn test_ca_06_twin_reverb_channels_spring_reverb_and_bias_tremolo() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp.set_sample_rate(48000.0);
    amp.panel_mut().set_input_gain(4.0);
    amp.panel_mut().set_reverb(7.5);
    amp.panel_mut().set_vibrato_speed(6.0);
    amp.panel_mut().set_vibrato_depth(8.5);

    // Pulso curto seguido de silêncio longo para medir cauda de Reverb no canal Vibrato vs Normal
    let burst_len = 2048;
    let tail_len = 16384;
    let mut sig = vec![0.0f32; burst_len + tail_len];
    for (i, s) in sig[..burst_len].iter_mut().enumerate() {
        *s = 0.25 * (2.0 * PI * 330.0 * (i as f32 / 48000.0)).sin();
    }

    amp.set_input_jack(AmpInputJack::Normal);
    let mut out_norm = vec![0.0f32; sig.len()];
    amp.process(&sig, &mut out_norm);
    let tail_rms_norm = rms(&out_norm[burst_len + 4096..]);

    let mut amp_vib = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp_vib.set_sample_rate(48000.0);
    amp_vib.panel_mut().set_input_gain(4.0);
    amp_vib.panel_mut().set_reverb(7.5);
    amp_vib.panel_mut().set_vibrato_speed(6.0);
    amp_vib.panel_mut().set_vibrato_depth(0.0);
    amp_vib.set_input_jack(AmpInputJack::Vibrato);
    let mut out_vib = vec![0.0f32; sig.len()];
    amp_vib.process(&sig, &mut out_vib);
    let tail_rms_vib = rms(&out_vib[burst_len + 4096..]);

    assert!(
        tail_rms_vib > tail_rms_norm * 5.0 && tail_rms_vib > 0.002,
        "Canal Vibrato do Twin Reverb com reverb=7.5 deve gerar cauda real vs Normal seco (norm={:.6}, vib={:.6})",
        tail_rms_norm, tail_rms_vib
    );

    // Modulação de amplitude por LFO (Vibrato/Tremolo) no canal Vibrato
    amp_vib.panel_mut().set_reverb(0.0);
    amp_vib.panel_mut().set_vibrato_speed(6.0);
    amp_vib.panel_mut().set_vibrato_depth(9.0);
    let cont_n = 24000;
    let cont_in: Vec<f32> = (0..cont_n)
        .map(|i| 0.1 * (2.0 * PI * 440.0 * (i as f32 / 48000.0)).sin())
        .collect();
    let mut cont_out = vec![0.0f32; cont_n];
    amp_vib.process(&cont_in, &mut cont_out);

    let mut min_env = f32::MAX;
    let mut max_env = 0.0f32;
    for chunk in cont_out[4096..].chunks(512) {
        let e = rms(chunk);
        min_env = min_env.min(e);
        max_env = max_env.max(e);
    }
    assert!(
        max_env > min_env * 1.45,
        "Bias tremolo no canal Vibrato deve modular envelope (min={:.5}, max={:.5})",
        min_env, max_env
    );
}

#[test]
fn test_ca_07_real_selectable_oversampling_2x_4x_8x() {
    let n = 4096;
    let sr = 48000.0f32;
    // Seno 5 kHz em alto ganho gera harmônicos/aliasing acima de Nyquist (24 kHz)
    let input: Vec<f32> = (0..n)
        .map(|i| 0.25 * (2.0 * PI * 5000.0 * (i as f32 / sr)).sin())
        .collect();

    let measure_non_harmonic_alias = |factor: usize| -> (u32, f64) {
        let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::CaliDualRectoRevF);
        amp.set_sample_rate(sr);
        amp.set_channel_mode(AmpChannelMode::Modern);
        amp.panel_mut().set_input_gain(10.0);
        amp.set_oversampling_factor(factor);
        let lat = amp.algorithmic_latency_samples();
        let mut out = vec![0.0f32; n];
        amp.process(&input, &mut out);
        // Em fs=48k com f0=5k, harmônicas verdadeiras são 5k, 10k, 15k, 20k.
        // 5*5k = 25k dobra em 23k; 6*5k = 30k dobra em 18k; 7*5k = 35k dobra em 13k.
        let alias_13k = goertzel_mag(&out[1024..], 48000.0, 13000.0);
        let alias_18k = goertzel_mag(&out[1024..], 48000.0, 18000.0);
        let f1 = goertzel_mag(&out[1024..], 48000.0, 5000.0).max(1e-9);
        let alias_dbc = 20.0 * ((alias_13k + alias_18k) * 0.5 / f1).max(1e-9).log10();
        (lat, alias_dbc)
    };

    let (lat2, alias2) = measure_non_harmonic_alias(2);
    let (lat4, alias4) = measure_non_harmonic_alias(4);
    let (lat8, alias8) = measure_non_harmonic_alias(8);

    assert!(
        lat2 > 0 && lat4 > 0 && lat8 > 0,
        "latências medidas devem ser positivas (lat2={lat2}, lat4={lat4}, lat8={lat8})"
    );
    assert!(
        alias8 < alias2 - 6.0,
        "Oversampling 8x deve reduzir aliasing significativamente vs 2x (alias2={:.2} dBc, alias4={:.2} dBc, alias8={:.2} dBc)",
        alias2, alias4, alias8
    );
}

#[test]
fn test_ca_09_zero_heap_allocations_during_process_and_switch() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);
    let input = [0.05f32; 256];
    let mut output = [0.0f32; 256];

    // Warm-up
    amp.process(&input, &mut output);

    ALLOC_COUNT.with(|a| a.set(0));
    COUNT_ALLOCS.with(|c| c.set(true));

    for i in 0..20 {
        if i == 5 {
            amp.set_target_model(AmpModelId::UkAc30TopBoost);
        }
        if i == 10 {
            amp.set_preamp_tube_type(PreampTube::Ef86);
            amp.set_power_tube_type(PowerTube::El84);
            amp.set_tonestack_topology(ToneStackTopology::VoxTopBoost);
        }
        amp.process(&input, &mut output);
    }

    COUNT_ALLOCS.with(|c| c.set(false));
    let allocs = ALLOC_COUNT.with(|a| a.get());
    assert_eq!(
        allocs, 0,
        "Zero alocações de heap permitidas durante process() e troca de modelo (houve {})",
        allocs
    );
}

#[test]
fn test_ca_10_safety_power_standby_and_no_nan_clip() {
    let models = [
        AmpModelId::BritPlexi1959,
        AmpModelId::BritJcm800_2203,
        AmpModelId::UsTwinBlackface65,
        AmpModelId::CaliDualRectoRevF,
        AmpModelId::UkAc30TopBoost,
    ];
    let mut in_buf = vec![0.0f32; 512];
    for (i, s) in in_buf.iter_mut().enumerate() {
        *s = 0.8 * (2.0 * PI * 300.0 * (i as f32 / 48000.0)).sin();
    }
    let mut out_buf = vec![0.0f32; 512];

    for m in models {
        let mut amp = WhiteBoxAmpProcessor::new(m);
        amp.set_sample_rate(48000.0);
        amp.panel_mut().set_input_gain(10.0);
        amp.panel_mut().set_master_volume(10.0);
        amp.panel_mut().set_output_level_db(12.0);
        amp.process(&in_buf, &mut out_buf);
        for &y in &out_buf {
            assert!(y.is_finite() && y.abs() <= 1.0, "Saída deve permanecer finita e <= 1.0");
        }

        // Standby corta B+
        amp.set_standby(true);
        amp.process(&in_buf, &mut out_buf);
        assert!(rms(&out_buf[256..]) < 1e-5, "Standby deve silenciar saída");

        // Power Off corta rede
        amp.set_standby(false);
        amp.set_power(false);
        amp.process(&in_buf, &mut out_buf);
        assert!(rms(&out_buf[256..]) < 1e-5, "Power off deve silenciar saída");
    }
}
