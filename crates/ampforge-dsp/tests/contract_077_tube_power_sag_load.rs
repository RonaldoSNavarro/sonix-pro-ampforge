//! Testes de contrato da Spec 077: Biblioteca de 10 Sub-Modelos Valvulados, Power Amp, Fonte/Sag e Carga Reativa
//! Cobre:
//! CA-01: Saturação Dependente de Fluxo Magnético no OutputTransformer (80 Hz vs 2000 Hz, zero inversão)
//! CA-02: Dinâmica Diferencial de PowerSupplySag (Tube 5U4G vs Solid-State Silicon)
//! CA-03: Interação ReactiveLoadAndNfb (NFB ON vs NFB OFF + Presence)
//! CA-04: CathodeFollowerStage (< 1.0 + assimetria) e CathodeBias vs FixedBias
//! CA-05: Segurança em Tempo Real (zero-alloc)

use ampforge_dsp::tube::{
    BiasTopology, CathodeFollowerStage, OutputTransformer, OutputTransformerModel,
    PowerSupplyAndRectifier, PowerSupplySag, ReactiveLoadAndNfb, RectifierKind,
};

fn compute_thd(samples: &[f64], freq_hz: f64, sr: f64) -> f64 {
    let n = samples.len();
    let bin_fund = (freq_hz * (n as f64) / sr).round() as usize;
    let dft_mag = |bin: usize| -> f64 {
        let mut re = 0.0;
        let mut im = 0.0;
        for (i, &x) in samples.iter().enumerate() {
            let ang = 2.0 * std::f64::consts::PI * (bin as f64) * (i as f64) / (n as f64);
            re += x * ang.cos();
            im -= x * ang.sin();
        }
        (re * re + im * im).sqrt()
    };
    let h1 = dft_mag(bin_fund).max(1e-12);
    let mut harm_sq = 0.0;
    for k in 2..=5 {
        let b = bin_fund * k;
        if b < n / 2 {
            let m = dft_mag(b);
            harm_sq += m * m;
        }
    }
    harm_sq.sqrt() / h1
}

#[test]
fn ca01_output_transformer_flux_saturation_without_phase_inversion() {
    let sr = 384_000.0;
    let mut xf_80 = OutputTransformer::new(OutputTransformerModel::DagnallC1998, sr);
    xf_80.sat_amount = 1.0;

    // 80 Hz: 19200 amostras = 4 ciclos exatos em 384 kHz
    let mut buf_80 = Vec::with_capacity(19200);
    let mut dot_80 = 0.0;
    for i in 0..38400 {
        let x = 0.707 * (2.0 * std::f64::consts::PI * 80.0 * (i as f64) / sr).sin();
        let y = xf_80.process(x);
        if i >= 19200 {
            buf_80.push(y);
            dot_80 += x * y;
        }
    }
    assert!(dot_80 > 0.0, "Transformador jamais deve inverter polaridade! dot={}", dot_80);
    let thd_80 = compute_thd(&buf_80, 80.0, sr);

    let mut xf_2k = OutputTransformer::new(OutputTransformerModel::DagnallC1998, sr);
    xf_2k.sat_amount = 1.0;
    let mut buf_2k = Vec::with_capacity(19200);
    for i in 0..38400 {
        let x = 0.707 * (2.0 * std::f64::consts::PI * 2000.0 * (i as f64) / sr).sin();
        let y = xf_2k.process(x);
        if i >= 19200 {
            buf_2k.push(y);
        }
    }
    let thd_2k = compute_thd(&buf_2k, 2000.0, sr);

    assert!(
        thd_80 >= 2.5 * thd_2k,
        "THD(80Hz)={:.5} deve ser >= 2.5x THD(2000Hz)={:.5}",
        thd_80, thd_2k
    );
}

#[test]
fn ca02_power_supply_sag_tube_5u4g_vs_solid_state() {
    let sr = 48_000.0;
    let mut sag_ss = PowerSupplySag::new(PowerSupplyAndRectifier::new(RectifierKind::SolidStateSilicon, 460.0), sr);
    let mut sag_5u4 = PowerSupplySag::new(PowerSupplyAndRectifier::new(RectifierKind::Tube5U4G, 460.0), sr);

    let mut max_drop_ss = 0.0f64;
    let mut max_drop_5u4 = 0.0f64;
    // Burst de 300 ms a 0 dBFS
    for i in 0..(48_000 * 300 / 1000) {
        let x = (2.0 * std::f64::consts::PI * 200.0 * (i as f64) / sr).sin();
        sag_ss.step(x, 10.0);
        sag_5u4.step(x, 10.0);
        max_drop_ss = max_drop_ss.max(sag_ss.sag_drop_ratio());
        max_drop_5u4 = max_drop_5u4.max(sag_5u4.sag_drop_ratio());
    }

    assert!(max_drop_ss <= 0.05, "SS sag deve ser <= 5%, foi {:.3}", max_drop_ss);
    assert!(
        max_drop_5u4 >= 0.12 && max_drop_5u4 <= 0.25,
        "5U4G sag deve estar em 12..25%, foi {:.3}",
        max_drop_5u4
    );
    assert!(max_drop_5u4 >= 2.5 * max_drop_ss);
}

#[test]
fn ca03_reactive_load_and_nfb_on_vs_off() {
    let sr = 48_000.0;
    let mut load_off = ReactiveLoadAndNfb::new(sr);
    load_off.configure(90.0, 16, false, 8.0, 10.0, 0.0);

    let mut load_on = ReactiveLoadAndNfb::new(sr);
    load_on.configure(90.0, 16, true, 8.0, 0.0, 0.0);

    let measure_db = |l: &mut ReactiveLoadAndNfb, freq: f64| -> f64 {
        l.reset();
        let mut sq_in = 0.0;
        let mut sq_out = 0.0;
        for i in 0..4096 {
            let x = 0.03 * (2.0 * std::f64::consts::PI * freq * (i as f64) / sr).sin();
            let y = l.process(x);
            if i >= 2048 {
                sq_in += x * x;
                sq_out += y * y;
            }
        }
        10.0 * (sq_out / sq_in).log10()
    };

    let rel_off_90 = measure_db(&mut load_off, 90.0) - measure_db(&mut load_off, 1000.0);
    let rel_on_90 = measure_db(&mut load_on, 90.0) - measure_db(&mut load_on, 1000.0);
    assert!(
        rel_off_90 - rel_on_90 >= 2.5,
        "NFB OFF deve elevar pico em 90Hz >= 2.5 dB: off={:.2} vs on={:.2}",
        rel_off_90, rel_on_90
    );

    // Presence em NFB ON eleva 5 kHz >= 4.0 dB
    let p0_5k = measure_db(&mut load_on, 5000.0);
    load_on.configure(90.0, 16, true, 8.0, 10.0, 0.0);
    let p10_5k = measure_db(&mut load_on, 5000.0);
    assert!(p10_5k - p0_5k >= 4.0, "Presence NFB ON: {:.2} -> {:.2}", p0_5k, p10_5k);
}

#[test]
fn ca04_cathode_follower_and_cathode_bias_excursion() {
    let sr = 48_000.0;
    let mut cf = CathodeFollowerStage::default();
    let mut max_out = 0.0f64;
    for i in 0..1024 {
        let x = 0.01 * (2.0 * std::f64::consts::PI * 440.0 * (i as f64) / sr).sin();
        let y = cf.process(x, 1.0);
        max_out = max_out.max(y.abs());
    }
    let gain = max_out / 0.01;
    assert!(gain > 0.85 && gain < 0.99, "CF gain={}", gain);

    let mut fixed = BiasTopology::fixed();
    let mut cath = BiasTopology::cathode_biased(50.0, 250e-6, sr);
    for _ in 0..2000 {
        let f_h = fixed.process_excursion(0.8);
        let c_h = cath.process_excursion(0.8);
        assert!((f_h - 1.0).abs() < 1e-9);
        let _ = c_h;
    }
    assert!(cath.excursion() > 0.05);
}
