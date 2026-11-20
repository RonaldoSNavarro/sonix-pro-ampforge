//! Spec 052 — Testes de contrato para Wah clássico com indutor ressonante (`WahProcessor`),
//! segurança tempo-real zero-alloc e efeito mensurável dos modelos de modulação e pitch (CA-3).

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};

use ampforge_dsp::fx::modulation::{ModModel, ModulationProcessor};
use ampforge_dsp::fx::pitch::{PitchModel, PitchProcessor};
use ampforge_dsp::fx::wah::{WahMode, WahProcessor};

struct CountingAllocator;

thread_local! {
    static COUNTING_ENABLED: Cell<bool> = const { Cell::new(false) };
}

static RT_ALLOCS: AtomicUsize = AtomicUsize::new(0);
static RT_DEALLOCS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING_ENABLED.with(|c| c.get()) {
            RT_ALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if COUNTING_ENABLED.with(|c| c.get()) {
            RT_DEALLOCS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

struct CountingGuard;
impl Drop for CountingGuard {
    fn drop(&mut self) {
        COUNTING_ENABLED.with(|c| c.set(false));
    }
}

fn with_counting_alloc<F: FnOnce()>(f: F) -> (usize, usize) {
    RT_ALLOCS.store(0, Ordering::SeqCst);
    RT_DEALLOCS.store(0, Ordering::SeqCst);
    COUNTING_ENABLED.with(|c| c.set(true));
    let _guard = CountingGuard;
    f();
    drop(_guard);
    (RT_ALLOCS.load(Ordering::SeqCst), RT_DEALLOCS.load(Ordering::SeqCst))
}

fn goertzel_mag(samples: &[f32], freq_hz: f32, sample_rate: f32) -> f32 {
    let n = samples.len();
    let omega = 2.0 * PI * freq_hz / sample_rate;
    let coeff = 2.0 * omega.cos();
    let mut s_prev = 0.0f64;
    let mut s_prev2 = 0.0f64;
    let coeff64 = coeff as f64;
    for &x in samples {
        let s = (x as f64) + coeff64 * s_prev - s_prev2;
        s_prev2 = s_prev;
        s_prev = s;
    }
    let power = s_prev2 * s_prev2 + s_prev * s_prev - coeff64 * s_prev * s_prev2;
    (power.max(0.0).sqrt() / (n as f64 * 0.5)) as f32
}

fn measure_wah_peak_hz(pos: f32, q: f32, mode: WahMode) -> (f32, f32) {
    let sr = 48_000.0;
    let mut wah = WahProcessor::new();
    wah.set_sample_rate(sr);
    wah.set_mode(mode);
    wah.set_q(q);
    wah.set_position(pos);
    wah.set_bypassed(false);

    // Warm up smoother to target position with silence
    let sil = [0.0f32; 4096];
    let mut tmp = [0.0f32; 4096];
    wah.process(&sil, &mut tmp);

    // Measure impulse response of stationary filter
    let mut imp = [0.0f32; 4096];
    imp[0] = 1.0;
    let mut out = [0.0f32; 4096];
    wah.process(&imp, &mut out);

    let mut best_freq = 350.0f32;
    let mut best_mag = -1.0f32;
    for step in 0..=260 {
        let f = 200.0 + step as f32 * 10.0;
        let mag = goertzel_mag(&out, f, sr);
        if mag > best_mag {
            best_mag = mag;
            best_freq = f;
        }
    }
    (best_freq, best_mag)
}

#[test]
fn test_ca_03_wah_processor_sweeps_resonant_peak_between_350_and_2200_hz() {
    let (f_heel, mag_heel) = measure_wah_peak_hz(0.0, 5.0, WahMode::CryBabyGcb95);
    let (f_mid, _) = measure_wah_peak_hz(0.5, 5.0, WahMode::CryBabyGcb95);
    let (f_toe, mag_toe) = measure_wah_peak_hz(1.0, 5.0, WahMode::CryBabyGcb95);

    assert!(
        (300.0..=420.0).contains(&f_heel),
        "CA-3: Calcanhar do Wah (position=0.0) deve ressoar perto de 350 Hz, obteve {f_heel:.1} Hz"
    );
    assert!(
        (760.0..=1000.0).contains(&f_mid),
        "CA-3: Meio do curso (position=0.5) deve ressoar perto de ~877 Hz, obteve {f_mid:.1} Hz"
    );
    assert!(
        (1950.0..=2450.0).contains(&f_toe),
        "CA-3: Ponta do Wah (position=1.0) deve ressoar perto de 2200 Hz, obteve {f_toe:.1} Hz"
    );
    assert!(mag_heel > 0.0 && mag_toe > 0.0);

    // Fator Q aumenta proeminência do pico
    let (_, mag_q_lo) = measure_wah_peak_hz(0.5, 1.0, WahMode::CryBabyGcb95);
    let (_, mag_q_hi) = measure_wah_peak_hz(0.5, 9.0, WahMode::CryBabyGcb95);
    assert!(
        mag_q_hi > mag_q_lo * 1.4,
        "CA-3: Wah Q=9.0 deve aumentar pico de ressonância frente a Q=1.0 (lo={mag_q_lo:.4}, hi={mag_q_hi:.4})"
    );

    // Compara modos CryBabyGcb95 vs ClydeMcCoy vs AutoEnvelope
    let sr = 48_000.0f32;
    let mut input = vec![0.0f32; 4096];
    for (i, s) in input.iter_mut().enumerate() {
        let t = i as f32 / sr;
        *s = 0.25 * (2.0 * PI * 440.0 * t).sin() + 0.15 * (2.0 * PI * 1320.0 * t).sin();
    }
    let run_mode = |mode: WahMode, pos: f32| -> Vec<f32> {
        let mut w = WahProcessor::new();
        w.set_sample_rate(sr);
        w.set_mode(mode);
        w.set_position(pos);
        w.set_q(6.0);
        let mut out = vec![0.0f32; input.len()];
        w.process(&input, &mut out);
        out
    };
    let out_gcb = run_mode(WahMode::CryBabyGcb95, 0.35);
    let out_mccoy = run_mode(WahMode::ClydeMcCoy, 0.35);
    let out_auto = run_mode(WahMode::AutoEnvelope, 0.0);
    let diff_mccoy: f32 = out_gcb.iter().zip(&out_mccoy).map(|(a, b)| (a - b).abs()).sum();
    let diff_auto: f32 = out_gcb.iter().zip(&out_auto).map(|(a, b)| (a - b).abs()).sum();
    assert!(diff_mccoy > 0.5, "CA-3: ClydeMcCoy deve diferir de CryBabyGcb95 (diff={diff_mccoy})");
    assert!(diff_auto > 0.5, "CA-3: AutoEnvelope deve modular com envoltória (diff={diff_auto})");

    // Bypass preserva entrada exata
    let mut w_byp = WahProcessor::new();
    w_byp.set_sample_rate(sr);
    w_byp.set_bypassed(true);
    let mut out_byp = vec![0.0f32; input.len()];
    w_byp.process(&input, &mut out_byp);
    let max_err = input
        .iter()
        .zip(&out_byp)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(max_err < 1e-6, "CA-3: Wah em bypass deve ser bit-transparente (max_err={max_err})");
}

#[test]
fn test_ca_03_wah_processor_zero_alloc_in_realtime_callback() {
    let mut wah = WahProcessor::new();
    wah.set_sample_rate(48_000.0);
    wah.set_mode(WahMode::CryBabyGcb95);
    wah.set_position(0.6);
    wah.set_q(5.0);

    // Verifica que o CountingAllocator detecta alocação real (não tautológico)
    let (injected, _) = with_counting_alloc(|| {
        let v: Vec<f32> = Vec::with_capacity(16);
        std::hint::black_box(&v);
    });
    assert!(injected >= 1, "CountingAllocator precisa detectar alocação de teste");

    let input = [0.08f32; 256];
    let mut output = [0.0f32; 256];
    let (allocs, deallocs) = with_counting_alloc(|| {
        for _ in 0..200 {
            wah.process(&input, &mut output);
        }
    });
    assert_eq!(allocs, 0, "CA-3: WahProcessor::process alocou {allocs} vez(es)");
    assert_eq!(deallocs, 0, "CA-3: WahProcessor::process desalocou {deallocs} vez(es)");
}

#[test]
fn test_ca_03_all_modulation_and_pitch_models_have_measurable_dsp_effect() {
    let sr = 48_000.0f32;
    let n = 8192usize;
    let input: Vec<f32> = (0..n)
        .map(|i| 0.35 * (2.0 * PI * 220.0 * i as f32 / sr).sin())
        .collect();

    for model in [
        ModModel::AnalogBbdChorus,
        ModModel::StereoTriChorus,
        ModModel::ThroughZeroFlanger,
        ModModel::Orange90Phaser,
        ModModel::OpticalBiasTremolo,
        ModModel::PhotocellUnivibe,
        ModModel::RotarySpeaker,
    ] {
        let mut p = ModulationProcessor::new(model);
        p.set_sample_rate(sr);
        p.set_rate_hz(2.5);
        p.set_depth(8.5);
        p.set_mix_pct(80.0);
        let mut out = vec![0.0f32; n];
        p.process(&input, &mut out);
        let delta: f32 = out[2048..]
            .iter()
            .zip(&input[2048..])
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(
            delta > 1.0,
            "CA-3: modelo de modulação {:?} deve alterar o sinal de entrada (delta={delta})",
            model
        );
    }

    for (model, st) in [
        (PitchModel::PedalWhammy, 12.0f32),
        (PitchModel::PolyphonicDropCapo, -2.0),
        (PitchModel::IntelligentDualHarmonizer, 0.0),
        (PitchModel::MicroPitchDetune, 0.25),
    ] {
        let mut p = PitchProcessor::new(model);
        p.prepare(sr);
        p.set_semitones(st);
        p.set_mix_pct(100.0);
        let mut out = vec![0.0f32; n];
        p.process(&input, &mut out);
        let delta: f32 = out[2048..]
            .iter()
            .zip(&input[2048..])
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(
            delta > 1.0,
            "CA-3: modelo de pitch {:?} deve alterar o sinal de entrada (delta={delta})",
            model
        );
    }
}
