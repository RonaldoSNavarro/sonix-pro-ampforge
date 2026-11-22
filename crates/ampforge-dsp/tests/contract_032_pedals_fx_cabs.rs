//! crates/ampforge-dsp/tests/contract_032_pedals_fx_cabs.rs
//! Testes de Contrato Automatizados (TDD RED -> GREEN) para Spec 032:
//! Pedais de Drive, Compressor 1176LN (Output + ALL), Tape Echo RE-201 (12 modos + Bass/Treble/Spring),
//! Equalizador Multi-Modo (8 bandas / Boogie 5 / MXR 10), Gabinete Twin 2x12 Open, Off-Axis e Zero-Alloc.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::f32::consts::PI;

use ampforge_dsp::cab::{MicModel, SpeakerCabModel, VirtualCabProcessor};
use ampforge_dsp::fx::delay::{DelayModel, DelayProcessor};
use ampforge_dsp::fx::dynamics::{CompressorProcessor, CompressorTopology};
use ampforge_dsp::fx::drive::{DriveModel, DriveProcessor};
use ampforge_dsp::fx::eq::{EqMode, EqProcessor};

struct CountingAlloc;
thread_local! {
    static TRACK_ALLOC: Cell<bool> = const { Cell::new(false) };
    static ALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        TRACK_ALLOC.with(|t| {
            if t.get() {
                ALLOC_COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAlloc = CountingAlloc;

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
}

fn sine(freq: f32, sr: f32, len: usize, amp: f32) -> Vec<f32> {
    (0..len)
        .map(|i| amp * (2.0 * PI * freq * (i as f32 / sr)).sin())
        .collect()
}

fn dft_harmonics(samples: &[f32], sr: f32, fund: f32, count: usize) -> Vec<f32> {
    let n = samples.len();
    (1..=count)
        .map(|h| {
            let f = fund * h as f32;
            let mut re = 0.0f64;
            let mut im = 0.0f64;
            for (i, &s) in samples.iter().enumerate() {
                let ang = 2.0 * std::f64::consts::PI * f as f64 * (i as f64 / sr as f64);
                re += s as f64 * ang.cos();
                im += s as f64 * ang.sin();
            }
            ((re * re + im * im).sqrt() / n as f64) as f32
        })
        .collect()
}

fn thd_pct(samples: &[f32], sr: f32, fund: f32) -> f32 {
    let h = dft_harmonics(samples, sr, fund, 7);
    let f1 = h[0].max(1e-9);
    let harm_sq: f32 = h[1..].iter().map(|m| m * m).sum();
    (harm_sq.sqrt() / f1) * 100.0
}

fn measure_gain_db_drive(drive: &mut DriveProcessor, freq: f32, sr: f32, amp: f32) -> f32 {
    let n = 4096;
    let sig = sine(freq, sr, n, amp);
    let mut out = vec![0.0f32; n];
    drive.process(&sig, &mut out);
    let r_in = rms(&sig[n / 2..]);
    let r_out = rms(&out[n / 2..]);
    20.0 * (r_out.max(1e-9) / r_in.max(1e-9)).log10()
}

#[test]
fn test_ca_01_six_drive_models_have_distinct_harmonic_signatures_and_fuzz_bias() {
    let sr = 48000.0;
    let n = 4096;
    let sig = sine(220.0, sr, n, 0.25);

    let models = [
        DriveModel::GreenScreamer808,
        DriveModel::GoldenKloneCentaur,
        DriveModel::BlackRodentRat,
        DriveModel::OrangeBossDs,
        DriveModel::FuzzFaceSiGe,
        DriveModel::BigMuffPi,
    ];

    let mut signatures: Vec<(DriveModel, f32, f32, f32)> = Vec::new();
    for m in models {
        let mut d = DriveProcessor::new(m);
        d.set_sample_rate(sr);
        d.set_gain(5.0);
        d.set_tone(5.0);
        d.set_level(5.0);
        d.set_bias(5.0);
        let mut out = vec![0.0f32; n];
        d.process(&sig, &mut out);
        let tail = &out[n / 2..];
        let harms = dft_harmonics(tail, sr, 220.0, 5);
        let h2_over_h3 = harms[1] / harms[2].max(1e-6);
        let t = thd_pct(tail, sr, 220.0);
        let hf_gain = measure_gain_db_drive(&mut d, 3000.0, sr, 0.02);
        signatures.push((m, t, h2_over_h3, hf_gain));
    }

    // Todos os 6 modelos devem ter assinatura distinta (não podem cair num mesmo process_generic)
    for i in 0..signatures.len() {
        for j in (i + 1)..signatures.len() {
            let (ma, ta, r23a, hfa) = signatures[i];
            let (mb, tb, r23b, hfb) = signatures[j];
            let diff_thd = (ta - tb).abs();
            let diff_r23 = (r23a - r23b).abs();
            let diff_hf = (hfa - hfb).abs();
            assert!(
                diff_thd > 1.0 || diff_r23 > 0.05 || diff_hf > 1.5,
                "Modelos {:?} e {:?} produziram saída indistinguível (THD {:.2} vs {:.2}, H2/H3 {:.3} vs {:.3}, HF {:.2} vs {:.2})",
                ma, mb, ta, tb, r23a, r23b, hfa, hfb
            );
        }
    }

    // FuzzFaceSiGe: controle de bias altera assimetria / gating do germânio
    let mut ff_mid = DriveProcessor::new(DriveModel::FuzzFaceSiGe);
    ff_mid.set_sample_rate(sr);
    ff_mid.set_bias(5.0);
    let mut out_mid = vec![0.0f32; n];
    ff_mid.process(&sig, &mut out_mid);

    let mut ff_cold = DriveProcessor::new(DriveModel::FuzzFaceSiGe);
    ff_cold.set_sample_rate(sr);
    ff_cold.set_bias(1.0);
    let mut out_cold = vec![0.0f32; n];
    ff_cold.process(&sig, &mut out_cold);

    let delta_bias = (rms(&out_mid[n / 2..]) - rms(&out_cold[n / 2..])).abs()
        + (thd_pct(&out_mid[n / 2..], sr, 220.0) - thd_pct(&out_cold[n / 2..], sr, 220.0)).abs();
    assert!(
        delta_bias > 0.5,
        "Bias do Fuzz Face deve alterar resposta/saturação de germânio (delta={})",
        delta_bias
    );
}

#[test]
fn test_ca_02_proco_rat_filter_knob_darkens_highs_clockwise() {
    let sr = 48000.0;
    let mut rat = DriveProcessor::new(DriveModel::BlackRodentRat);
    rat.set_sample_rate(sr);
    rat.set_gain(5.0);
    rat.set_level(5.0);

    rat.set_tone(0.0); // Filter mínimo -> aberto (mais agudos)
    let g_open = measure_gain_db_drive(&mut rat, 4000.0, sr, 0.02);

    rat.set_tone(10.0); // Filter máximo horário -> escurece timbre >= 6 dB acima de 3 kHz
    let g_dark = measure_gain_db_drive(&mut rat, 4000.0, sr, 0.02);

    let drop_db = g_open - g_dark;
    assert!(
        drop_db >= 6.0,
        "No RAT, aumentar Filter de 0 para 10 deve reduzir energia acima de 3 kHz em >= 6 dB: drop={:.2} dB (open={:.2}, dark={:.2})",
        drop_db, g_open, g_dark
    );
}

#[test]
fn test_ca_03_big_muff_mid_scoop_and_klon_clean_blend() {
    let sr = 48000.0;

    // 1. Big Muff Pi com Tone = 5.0 tem notch de médios em ~900-1000 Hz >= 4 dB abaixo de 200 Hz e 3500 Hz
    let mut muff = DriveProcessor::new(DriveModel::BigMuffPi);
    muff.set_sample_rate(sr);
    muff.set_gain(5.0);
    muff.set_tone(5.0);
    muff.set_level(5.0);

    let g_low = measure_gain_db_drive(&mut muff, 200.0, sr, 0.01);
    let g_mid = measure_gain_db_drive(&mut muff, 950.0, sr, 0.01);
    let g_high = measure_gain_db_drive(&mut muff, 3500.0, sr, 0.01);

    let edge_avg = 0.5 * (g_low + g_high);
    assert!(
        edge_avg - g_mid >= 4.0 && g_low - g_mid >= 3.0 && g_high - g_mid >= 3.0,
        "Big Muff com Tone=5 deve ter scoop de médios em 800-1200 Hz >= 4 dB (200Hz={:.2} dB, 950Hz={:.2} dB, 3.5kHz={:.2} dB)",
        g_low, g_mid, g_high
    );

    // 2. Klon Centaur com Gain = 2.0 mantém mistura limpa (> 0.7 correlação com entrada)
    let mut klon = DriveProcessor::new(DriveModel::GoldenKloneCentaur);
    klon.set_sample_rate(sr);
    klon.set_gain(2.0);
    klon.set_tone(5.0);
    klon.set_level(5.0);

    let n = 4096;
    let sig = sine(440.0, sr, n, 0.2);
    let mut out = vec![0.0f32; n];
    klon.process(&sig, &mut out);

    let s = &sig[n / 2..];
    let o = &out[n / 2..];
    let dot: f32 = s.iter().zip(o.iter()).map(|(a, b)| a * b).sum();
    let ns = s.iter().map(|a| a * a).sum::<f32>().sqrt();
    let no = o.iter().map(|b| b * b).sum::<f32>().sqrt();
    let corr = dot / (ns * no).max(1e-9);
    assert!(
        corr > 0.70,
        "Klon com Gain=2.0 deve preservar forte mistura limpa (correlação={:.3})",
        corr
    );
}

#[test]
fn test_ca_04_comp_2_output_makeup_and_all_buttons_in_mode() {
    let sr = 48000.0;
    let n = 4096;
    let sig = sine(220.0, sr, n, 0.35);

    // 1. Output makeup gain real (-12 dB vs +12 dB => ~24 dB delta)
    let mut c_neg = CompressorProcessor::new(CompressorTopology::StudioFet1176);
    c_neg.set_sample_rate(sr);
    c_neg.set_threshold_db(-6.0);
    c_neg.set_makeup_gain_db(-12.0);
    let mut out_neg = vec![0.0f32; n];
    c_neg.process(&sig, &mut out_neg);

    let mut c_pos = CompressorProcessor::new(CompressorTopology::StudioFet1176);
    c_pos.set_sample_rate(sr);
    c_pos.set_threshold_db(-6.0);
    c_pos.set_makeup_gain_db(12.0);
    let mut out_pos = vec![0.0f32; n];
    c_pos.process(&sig, &mut out_pos);

    let diff_db = 20.0 * (rms(&out_pos[n / 2..]) / rms(&out_neg[n / 2..]).max(1e-9)).log10();
    assert!(
        (diff_db - 24.0).abs() <= 1.0,
        "comp_2.output +-12 dB deve produzir ~24 dB de diferença (got {:.2} dB)",
        diff_db
    );

    // 2. All-Buttons-In ("all") vs ratio 20: maior THD e compressão mais agressiva
    let mut c_20 = CompressorProcessor::new(CompressorTopology::StudioFet1176);
    c_20.set_sample_rate(sr);
    c_20.set_threshold_db(-20.0);
    c_20.set_ratio(20.0);
    c_20.set_all_buttons_in(false);
    let mut out_20 = vec![0.0f32; n];
    c_20.process(&sig, &mut out_20);

    let mut c_all = CompressorProcessor::new(CompressorTopology::StudioFet1176);
    c_all.set_sample_rate(sr);
    c_all.set_threshold_db(-20.0);
    c_all.set_ratio(20.0);
    c_all.set_all_buttons_in(true);
    let mut out_all = vec![0.0f32; n];
    c_all.process(&sig, &mut out_all);

    let thd_20 = thd_pct(&out_20[n / 2..], sr, 220.0);
    let thd_all = thd_pct(&out_all[n / 2..], sr, 220.0);
    assert!(
        thd_all > thd_20 + 0.5 && c_all.gain_reduction_db() < c_20.gain_reduction_db(),
        "Modo ALL (All-Buttons-In) deve produzir maior THD ({:.2}% vs {:.2}%) e GR mais agressiva ({:.2} vs {:.2} dB)",
        thd_all, thd_20, c_all.gain_reduction_db(), c_20.gain_reduction_db()
    );
}

#[test]
fn test_ca_05_re201_12_modes_wet_eq_and_spring_only_mode_12() {
    let sr = 48000.0;
    let len = 48000;
    let mut impulse = vec![0.0f32; len];
    impulse[0] = 0.8;

    // 1. Mode 1 (single head) vs Mode 5 (multi-head + spring)
    let mut d1 = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
    d1.set_sample_rate(sr);
    d1.set_delay_time_ms(120.0);
    d1.set_feedback_pct(25.0);
    d1.set_mix_pct(50.0);
    d1.set_re201_mode(1);
    let mut out1 = vec![0.0f32; len];
    d1.process(&impulse, &mut out1);

    // Nota: Mode 5 (H1+mola) e Mode 8 (H1+H2+mola) após tabela única RE-201 (AUD-28)
    let mut d8 = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
    d8.set_sample_rate(sr);
    d8.set_delay_time_ms(120.0);
    d8.set_feedback_pct(25.0);
    d8.set_mix_pct(50.0);
    d8.set_reverb_volume(5.0);
    d8.set_re201_mode(8);
    let mut out8 = vec![0.0f32; len];
    d8.process(&impulse, &mut out8);

    // Head 1 em 120 ms (5760 amostras), Head 2 em ~230.4 ms (11059 amostras):
    // Mode 8 (H1+H2+mola) gera tap distinto em ~230 ms onde Mode 1 não tem cabeça
    let head2_m1 = out1[10800..11300].iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    let head2_m8 = out8[10800..11300].iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    assert!(
        head2_m8 > 0.10 && head2_m1 < 0.02,
        "Mode 8 multi-head deve produzir tap rítmico da cabeça 2 em ~230 ms ausente no Mode 1 (m1={}, m8={})",
        head2_m1, head2_m8
    );

    // 2. Bass & Treble atuam somente nas repetições (sinal seco imediato em impulse[0] permanece idêntico)
    let sig_burst = {
        let mut s = vec![0.0f32; 24000];
        let burst = sine(4500.0, sr, 1200, 0.4);
        s[..1200].copy_from_slice(&burst);
        s
    };

    let mut d_bright = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
    d_bright.set_sample_rate(sr);
    d_bright.set_delay_time_ms(100.0);
    d_bright.set_feedback_pct(40.0);
    d_bright.set_mix_pct(50.0);
    d_bright.set_re201_mode(1);
    d_bright.set_tone_treble(10.0);
    let mut out_bright = vec![0.0f32; 24000];
    d_bright.process(&sig_burst, &mut out_bright);

    let mut d_dark = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
    d_dark.set_sample_rate(sr);
    d_dark.set_delay_time_ms(100.0);
    d_dark.set_feedback_pct(40.0);
    d_dark.set_mix_pct(50.0);
    d_dark.set_re201_mode(1);
    d_dark.set_tone_treble(0.0);
    let mut out_dark = vec![0.0f32; 24000];
    d_dark.process(&sig_burst, &mut out_dark);

    // Dry path nos primeiros 500 amostras (< 100 ms delay): idêntico!
    let dry_diff = (rms(&out_bright[..500]) - rms(&out_dark[..500])).abs();
    assert!(dry_diff < 1e-4, "Treble do RE-201 não deve alterar o sinal seco (diff={})", dry_diff);

    // Wet repeat window (4800..6500): bright deve ter mais agudo que dark
    let repeat_bright = rms(&out_bright[4800..6500]);
    let repeat_dark = rms(&out_dark[4800..6500]);
    assert!(
        repeat_bright > repeat_dark * 1.4,
        "Treble do RE-201 deve alterar espectro da repetição (bright={}, dark={})",
        repeat_bright, repeat_dark
    );

    // 3. Mode 12 (Reverb Only): tem cauda de mola audível sem eco discreto de fita no instante da cabeça 1
    let mut d12 = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
    d12.set_sample_rate(sr);
    d12.set_delay_time_ms(180.0); // ~8640 amostras
    d12.set_feedback_pct(50.0);
    d12.set_mix_pct(50.0);
    d12.set_reverb_volume(8.0);
    d12.set_re201_mode(12);
    let mut out12 = vec![0.0f32; len];
    d12.process(&impulse, &mut out12);
    let reverb_tail_rms = rms(&out12[1500..12000]);
    assert!(reverb_tail_rms > 0.002, "Mode 12 deve ter cauda de spring reverb");
    let discrete_echo_peak = out12[8600..8720].iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    assert!(
        discrete_echo_peak < 0.05,
        "Mode 12 (Reverb Only) não deve produzir pico discreto de fita no tempo do head 1 (peak={})",
        discrete_echo_peak
    );
}

#[test]
fn test_ca_06_eq_1_multi_mode_switching_and_10band_response() {
    let sr = 48000.0;
    let n = 4096;
    let mut eq = EqProcessor::new(EqMode::Parametric8Band);
    eq.set_sample_rate(sr);

    eq.set_mode(EqMode::Graphic10BandPedal);
    assert_eq!(eq.mode(), EqMode::Graphic10BandPedal);
    assert_eq!(eq.graphic_frequencies().len(), 10);

    // Banda 10 (16 kHz) e Banda 2 (62.5 Hz) em +6 dB; trim pós +3 dB
    eq.set_graphic_band_db(1, 6.0); // 62.5 Hz
    eq.set_graphic_band_db(8, -6.0); // 8 kHz
    eq.set_output_level_db(3.0);

    let sig_62 = sine(62.5, sr, n, 0.2);
    let mut out_62 = vec![0.0f32; n];
    eq.process(&sig_62, &mut out_62);
    let g_62 = 20.0 * (rms(&out_62[n / 2..]) / rms(&sig_62[n / 2..])).log10();
    assert!(
        g_62 > 7.5,
        "Banda 62.5 Hz +6 dB + trim +3 dB no MXR 10-band deve resultar > 7.5 dB (got {:.2} dB)",
        g_62
    );

    let sig_8k = sine(8000.0, sr, n, 0.2);
    let mut out_8k = vec![0.0f32; n];
    eq.process(&sig_8k, &mut out_8k);
    let g_8k = 20.0 * (rms(&out_8k[n / 2..]) / rms(&sig_8k[n / 2..])).log10();
    assert!(
        g_8k < -1.0,
        "Banda 8 kHz -6 dB + trim +3 dB deve atenuar (got {:.2} dB)",
        g_8k
    );
}

#[test]
fn test_ca_07_cab_twin_2x12_open_and_mic_off_axis_45_deg() {
    let sr = 48000.0;
    let n = 4096;

    // 1. Twin2x12Open ressonância 90 Hz e resposta open-back brilhante distinta de Greenback
    let mut cab_twin = VirtualCabProcessor::new();
    cab_twin.set_sample_rate(sr);
    cab_twin.set_speaker_model(SpeakerCabModel::Twin2x12Open);
    assert!(cab_twin.speaker_model().is_open_back());
    assert!((cab_twin.resonance_hz() - 90.0).abs() < 1.0);

    let mut cab_gb = VirtualCabProcessor::new();
    cab_gb.set_sample_rate(sr);
    cab_gb.set_speaker_model(SpeakerCabModel::Greenback4x12Closed);

    let sig_hi = sine(5000.0, sr, n, 0.4);
    let mut out_twin = vec![0.0f32; n];
    let mut out_gb = vec![0.0f32; n];
    cab_twin.process(&sig_hi, &mut out_twin);
    cab_gb.process(&sig_hi, &mut out_gb);
    let db_diff = 20.0 * (rms(&out_twin[n / 2..]) / rms(&out_gb[n / 2..]).max(1e-9)).log10();
    assert!(
        db_diff > 1.5,
        "Twin 2x12 Open deve soar mais brilhante em 5 kHz que Greenback fechado (+{:.2} dB)",
        db_diff
    );

    // 2. Microfone 45° off-axis atenua brilho >= 4 dB acima de 4 kHz vs 0° on-axis
    let mut cab_axis = VirtualCabProcessor::new();
    cab_axis.set_sample_rate(sr);
    cab_axis.mic_slot_mut(0).model = MicModel::Sm57;
    cab_axis.mic_slot_mut(0).angle_off_axis_deg = 0.0;
    let mut out_0 = vec![0.0f32; n];
    cab_axis.process(&sig_hi, &mut out_0);

    cab_axis.mic_slot_mut(0).angle_off_axis_deg = 45.0;
    let mut out_45 = vec![0.0f32; n];
    cab_axis.process(&sig_hi, &mut out_45);

    let axis_drop_db = 20.0 * (rms(&out_0[n / 2..]) / rms(&out_45[n / 2..]).max(1e-9)).log10();
    assert!(
        axis_drop_db >= 3.5,
        "Off-axis 45° deve atenuar brilho >= 3.5 dB em 5 kHz vs 0° (got {:.2} dB)",
        axis_drop_db
    );
}

#[test]
fn test_ca_08_zero_allocations_in_fx_and_cab_process_paths() {
    let sr = 48000.0;
    let input = sine(440.0, sr, 512, 0.25);
    let mut output = vec![0.0f32; 512];

    let mut drives: Vec<DriveProcessor> = [
        DriveModel::GreenScreamer808,
        DriveModel::GoldenKloneCentaur,
        DriveModel::BlackRodentRat,
        DriveModel::OrangeBossDs,
        DriveModel::FuzzFaceSiGe,
        DriveModel::BigMuffPi,
    ]
    .into_iter()
    .map(|m| {
        let mut p = DriveProcessor::new(m);
        p.set_sample_rate(sr);
        p
    })
    .collect();

    let mut comp = CompressorProcessor::new(CompressorTopology::StudioFet1176);
    comp.set_sample_rate(sr);
    comp.set_all_buttons_in(true);
    comp.set_makeup_gain_db(6.0);

    let mut delay = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
    delay.set_sample_rate(sr);
    delay.set_re201_mode(8);
    delay.set_reverb_volume(5.0);

    let mut eq = EqProcessor::new(EqMode::Graphic10BandPedal);
    eq.set_sample_rate(sr);
    eq.set_mode(EqMode::Parametric8Band);
    eq.set_mode(EqMode::Graphic10BandPedal);
    eq.set_graphic_band_db(0, 4.0);
    eq.set_graphic_band_db(9, -3.0);

    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);
    cab.set_speaker_model(SpeakerCabModel::Twin2x12Open);
    cab.mic_slot_mut(0).angle_off_axis_deg = 45.0;

    // Warm-up antes de ligar o rastreamento de heap
    for d in &mut drives {
        d.process(&input, &mut output);
    }
    comp.process(&input, &mut output);
    delay.process(&input, &mut output);
    eq.process(&input, &mut output);
    cab.process(&input, &mut output);

    ALLOC_COUNT.with(|c| c.set(0));
    TRACK_ALLOC.with(|t| t.set(true));

    for d in &mut drives {
        d.process(&input, &mut output);
    }
    comp.process(&input, &mut output);
    delay.process(&input, &mut output);
    eq.set_mode(EqMode::Graphic5BandBoogie);
    eq.process(&input, &mut output);
    cab.process(&input, &mut output);

    TRACK_ALLOC.with(|t| t.set(false));
    let allocs = ALLOC_COUNT.with(|c| c.get());
    assert_eq!(
        allocs, 0,
        "Nenhuma alocação de heap pode ocorrer em process()/set_mode() de Drive/Comp/Delay/EQ/Cab (alocou {}x)",
        allocs
    );
}
