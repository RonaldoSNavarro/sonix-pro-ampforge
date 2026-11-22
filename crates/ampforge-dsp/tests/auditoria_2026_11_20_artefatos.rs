//! Testes de contrato para remoção de artefatos temporais e espúrios (Auditoria 2026-11-20).
//! Cobre Fase 3 (`AUD-10`..`AUD-15`).

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::f32::consts::PI;
use std::hint::black_box;
use std::path::Path;

use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::regua::razao_transiente;

const SR: f32 = 48_000.0;
const TOTAL_AMOSTRAS: usize = 48_000;
const EVENTO_AMOSTRA: usize = 24_064; // Múltiplo de 128 (188 * 128) e de 32 (752 * 32)

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

struct CountingGuard;
impl Drop for CountingGuard {
    fn drop(&mut self) {
        COUNTING_ACTIVE.with(|a| a.set(false));
    }
}

fn measure_allocs<F: FnOnce()>(f: F) -> (usize, usize) {
    THREAD_ALLOCS.with(|c| c.set(0));
    THREAD_DEALLOCS.with(|c| c.set(0));
    COUNTING_ACTIVE.with(|a| a.set(true));
    let _guard = CountingGuard;
    f();
    drop(_guard);
    (
        THREAD_ALLOCS.with(|c| c.get()),
        THREAD_DEALLOCS.with(|c| c.get()),
    )
}

fn verify_allocator_self_check() {
    let (allocs, deallocs) = measure_allocs(|| {
        let mut v = Vec::<u8>::with_capacity(16);
        v.push(42);
        black_box(&v);
    });
    assert!(
        allocs >= 1 && deallocs >= 1,
        "CountingAllocator self-check falhou: allocs={allocs}, deallocs={deallocs}"
    );
}

fn render_model_switch(a: AmpModelId, b: AmpModelId, block_size: usize) -> (Vec<f32>, f32) {
    let mut amp = WhiteBoxAmpProcessor::new(a);
    amp.set_sample_rate(SR);
    amp.panel_mut().set_input_gain(4.5);
    amp.panel_mut().set_master_volume(6.0);
    amp.panel_mut().set_volume_2(5.0);
    amp.panel_mut().set_bass(5.0);
    amp.panel_mut().set_middle(5.0);
    amp.panel_mut().set_treble(5.0);
    amp.panel_mut().set_presence(5.0);
    amp.set_standby(false);

    let mut input = vec![0.0_f32; TOTAL_AMOSTRAS];
    for (i, sample) in input.iter_mut().enumerate() {
        let t = i as f32 / SR;
        *sample = 0.2512 * (2.0 * PI * 220.0 * t).sin(); // -12 dBFS
    }
    let mut output = vec![0.0_f32; TOTAL_AMOSTRAS];

    let mut pos = 0;
    let mut switched = false;
    while pos < TOTAL_AMOSTRAS {
        if !switched && pos >= EVENTO_AMOSTRA {
            amp.set_target_model(b);
            switched = true;
        }
        let end = (pos + block_size).min(TOTAL_AMOSTRAS);
        amp.process(&input[pos..end], &mut output[pos..end]);
        pos = end;
    }

    let ratio = razao_transiente(&output, EVENTO_AMOSTRA, SR);
    (output, ratio)
}

#[test]
fn aud_10_troca_de_modelo_sem_clique() {
    let mut falhas = Vec::new();
    let mut max_ratio = 0.0_f32;
    for &a in &AmpModelId::ALL {
        for &b in &AmpModelId::ALL {
            if a == b {
                continue;
            }
            let (_out, ratio) = render_model_switch(a, b, 128);
            max_ratio = max_ratio.max(ratio);
            if ratio > 2.0 {
                falhas.push(format!("{a:?} -> {b:?}: razao_transiente={ratio:.3}"));
            }
        }
    }

    assert!(
        falhas.is_empty(),
        "AUD-10 falhou em bloco 128 (max={max_ratio:.3} > 2.0):\n{}",
        falhas.join("\n")
    );

    let amp_src = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/amp.rs"),
    )
    .expect("ler crates/ampforge-dsp/src/amp.rs");

    for forbidden in [
        "target_model_id",
        "target_voicing_",
        "target_mod_settings",
        "model_crossfade",
    ] {
        assert!(
            !amp_src.contains(forbidden),
            "AUD-10 lint: `{forbidden}` ainda existe em crates/ampforge-dsp/src/amp.rs"
        );
    }
}

#[test]
fn aud_10_troca_com_bloco_32() {
    let mut falhas = Vec::new();
    let mut max_ratio = 0.0_f32;
    for &a in &AmpModelId::ALL {
        for &b in &AmpModelId::ALL {
            if a == b {
                continue;
            }
            let (_out, ratio) = render_model_switch(a, b, 32);
            max_ratio = max_ratio.max(ratio);
            if ratio > 2.0 {
                falhas.push(format!("{a:?} -> {b:?}: razao_transiente={ratio:.3}"));
            }
        }
    }

    assert!(
        falhas.is_empty(),
        "AUD-10 falhou em bloco 32 (max={max_ratio:.3} > 2.0):\n{}",
        falhas.join("\n")
    );
}

#[test]
fn aud_10_zero_alloc() {
    verify_allocator_self_check();

    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp.set_sample_rate(SR);
    amp.set_standby(false);

    let inp128 = [0.2_f32; 128];
    let mut out128 = [0.0_f32; 128];
    let inp32 = [0.2_f32; 32];
    let mut out32 = [0.0_f32; 32];

    for _ in 0..32 {
        amp.process(&inp128, &mut out128);
    }

    let (allocs, deallocs) = measure_allocs(|| {
        amp.set_target_model(AmpModelId::BritJcm800_2203);
        for _ in 0..24 {
            amp.process(&inp128, &mut out128);
        }
        amp.set_target_model(AmpModelId::CaliDualRectoRevF);
        for _ in 0..48 {
            amp.process(&inp32, &mut out32);
        }
    });

    assert_eq!(allocs, 0, "AUD-10: troca de modelo alocou {allocs} vez(es)");
    assert_eq!(
        deallocs, 0,
        "AUD-10: troca de modelo desalocou {deallocs} vez(es)"
    );
}

use ampforge_dsp::cab::VirtualCabProcessor;
use ampforge_dsp::fx::eq::{EqMode, EqProcessor};
use ampforge_dsp::fx::wah::{WahMode, WahProcessor};
use ampforge_dsp::regua::ganho_db;

fn gen_sine(freq_hz: f32) -> Vec<f32> {
    (0..TOTAL_AMOSTRAS)
        .map(|i| {
            let t = i as f32 / SR;
            0.2512 * (2.0 * PI * freq_hz * t).sin() // -12 dBFS
        })
        .collect()
}

fn run_amp_step_configured<F: Fn(&mut WhiteBoxAmpProcessor, bool), S: Fn(&mut WhiteBoxAmpProcessor)>(
    model: AmpModelId,
    freq_hz: f32,
    init: S,
    apply: F,
    forward: bool,
) -> (Vec<f32>, Vec<f32>) {
    let mut amp = WhiteBoxAmpProcessor::new(model);
    amp.set_sample_rate(SR);
    amp.panel_mut().set_input_gain(3.0);
    amp.panel_mut().set_master_volume(6.0);
    amp.panel_mut().set_volume_2(5.0);
    amp.panel_mut().set_bass(5.0);
    amp.panel_mut().set_middle(5.0);
    amp.panel_mut().set_treble(5.0);
    amp.panel_mut().set_presence(5.0);
    amp.panel_mut().set_cut(2.0);
    amp.set_standby(false);
    init(&mut amp);
    apply(&mut amp, !forward);

    let input = gen_sine(freq_hz);
    let mut output = vec![0.0_f32; TOTAL_AMOSTRAS];
    let block = 128usize;
    let mut pos = 0;
    let mut stepped = false;
    while pos < TOTAL_AMOSTRAS {
        if !stepped && pos >= EVENTO_AMOSTRA {
            apply(&mut amp, forward);
            stepped = true;
        }
        let end = (pos + block).min(TOTAL_AMOSTRAS);
        amp.process(&input[pos..end], &mut output[pos..end]);
        pos = end;
    }
    (input, output)
}

fn run_amp_step<F: Fn(&mut WhiteBoxAmpProcessor, bool)>(
    model: AmpModelId,
    freq_hz: f32,
    apply: F,
    forward: bool,
) -> (Vec<f32>, Vec<f32>) {
    run_amp_step_configured(model, freq_hz, |_| {}, apply, forward)
}

fn run_cab_hicut_step(freq_hz: f32, forward: bool) -> (Vec<f32>, Vec<f32>) {
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(SR);
    cab.set_hicut_khz(if forward { 3.5 } else { 18.0 });

    let input = gen_sine(freq_hz);
    let mut output = vec![0.0_f32; TOTAL_AMOSTRAS];
    let block = 128usize;
    let mut pos = 0;
    let mut stepped = false;
    while pos < TOTAL_AMOSTRAS {
        if !stepped && pos >= EVENTO_AMOSTRA {
            cab.set_hicut_khz(if forward { 18.0 } else { 3.5 });
            stepped = true;
        }
        let end = (pos + block).min(TOTAL_AMOSTRAS);
        cab.process(&input[pos..end], &mut output[pos..end]);
        pos = end;
    }
    (input, output)
}

fn run_eq_step<F: Fn(&mut EqProcessor, bool)>(
    mode: EqMode,
    freq_hz: f32,
    apply: F,
    forward: bool,
) -> (Vec<f32>, Vec<f32>) {
    let mut eq = EqProcessor::new(mode);
    eq.set_sample_rate(SR);
    apply(&mut eq, !forward);

    let input = gen_sine(freq_hz);
    let mut output = vec![0.0_f32; TOTAL_AMOSTRAS];
    let block = 128usize;
    let mut pos = 0;
    let mut stepped = false;
    while pos < TOTAL_AMOSTRAS {
        if !stepped && pos >= EVENTO_AMOSTRA {
            apply(&mut eq, forward);
            stepped = true;
        }
        let end = (pos + block).min(TOTAL_AMOSTRAS);
        eq.process(&input[pos..end], &mut output[pos..end]);
        pos = end;
    }
    (input, output)
}

fn run_wah_step<F: Fn(&mut WahProcessor, bool)>(
    freq_hz: f32,
    apply: F,
    forward: bool,
) -> (Vec<f32>, Vec<f32>) {
    let mut wah = WahProcessor::new();
    wah.set_sample_rate(SR);
    wah.set_mode(WahMode::CryBabyGcb95);
    wah.set_q(4.0);
    wah.set_position(0.5);
    wah.set_bypassed(false);
    apply(&mut wah, !forward);

    let input = gen_sine(freq_hz);
    let mut output = vec![0.0_f32; TOTAL_AMOSTRAS];
    let block = 128usize;
    let mut pos = 0;
    let mut stepped = false;
    while pos < TOTAL_AMOSTRAS {
        if !stepped && pos >= EVENTO_AMOSTRA {
            apply(&mut wah, forward);
            stepped = true;
        }
        let end = (pos + block).min(TOTAL_AMOSTRAS);
        wah.process(&input[pos..end], &mut output[pos..end]);
        pos = end;
    }
    (input, output)
}

fn collect_aud_11_step_cases() -> Vec<(&'static str, bool, f32, Vec<f32>, Vec<f32>)> {
    let mut cases = Vec::new();
    let specs: &[(&str, &[f32], AmpModelId, fn(&mut WhiteBoxAmpProcessor, bool))] = &[
        ("amp.bass", &[1000.0, 100.0], AmpModelId::BritJcm800_2203, |a, hi| {
            a.panel_mut().set_bass(if hi { 10.0 } else { 0.0 });
        }),
        ("amp.middle", &[1000.0], AmpModelId::BritJcm800_2203, |a, hi| {
            a.panel_mut().set_middle(if hi { 10.0 } else { 0.0 });
        }),
        ("amp.treble", &[1000.0], AmpModelId::BritJcm800_2203, |a, hi| {
            a.panel_mut().set_treble(if hi { 10.0 } else { 0.0 });
        }),
        ("amp.presence", &[1000.0, 4000.0], AmpModelId::BritJcm800_2203, |a, hi| {
            a.panel_mut().set_presence(if hi { 10.0 } else { 0.0 });
        }),
        ("amp.master", &[1000.0], AmpModelId::BritJcm800_2203, |a, hi| {
            a.panel_mut().set_master_volume(if hi { 9.5 } else { 0.5 });
        }),
        ("amp.volume_2", &[1000.0], AmpModelId::UkAc30TopBoost, |a, hi| {
            a.panel_mut().set_volume_2(if hi { 9.5 } else { 0.5 });
        }),
        ("amp.cut", &[1000.0, 4000.0], AmpModelId::UkAc30TopBoost, |a, hi| {
            a.panel_mut().set_cut(if hi { 10.0 } else { 0.0 });
        }),
        (
            "amp.speaker_resonance_hz",
            &[1000.0, 100.0],
            AmpModelId::BritJcm800_2203,
            |a, hi| {
                a.set_speaker_resonance_hz(if hi { 135.0 } else { 55.0 });
            },
        ),
    ];

    for &(nome, freqs, model, func) in specs {
        for &freq in freqs {
            for forward in [true, false] {
                let (inp, out) = run_amp_step(model, freq, func, forward);
                cases.push((nome, forward, freq, inp, out));
            }
        }
    }

    for freq in [1000.0, 4500.0] {
        for forward in [true, false] {
            let (inp, out) = run_cab_hicut_step(freq, forward);
            cases.push(("cab.hicut_khz", forward, freq, inp, out));
        }
    }

    for forward in [true, false] {
        let (inp, out) = run_eq_step(
            EqMode::Graphic10BandPedal,
            1000.0,
            |e, hi| e.set_graphic_band_db(6, if hi { 12.0 } else { -12.0 }),
            forward,
        );
        cases.push(("eq.graphic_band_gain", forward, 1000.0, inp, out));

        let (inp, out) = run_eq_step(
            EqMode::Parametric8Band,
            1000.0,
            |e, hi| e.set_parametric_band(3, 1000.0, if hi { 12.0 } else { -12.0 }, 1.5),
            forward,
        );
        cases.push(("eq.param_gain", forward, 1000.0, inp, out));

        let (inp, out) = run_eq_step(
            EqMode::Parametric8Band,
            1000.0,
            |e, hi| e.set_parametric_band(3, if hi { 1000.0 } else { 300.0 }, 9.0, 2.0),
            forward,
        );
        cases.push(("eq.param_freq", forward, 1000.0, inp, out));

        for freq in [1000.0, 1400.0] {
            let (inp, out) = run_eq_step(
                EqMode::Parametric8Band,
                freq,
                |e, hi| e.set_parametric_band(3, 1000.0, 9.0, if hi { 6.0 } else { 0.7 }),
                forward,
            );
            cases.push(("eq.param_q", forward, freq, inp, out));
        }

        let (inp, out) = run_eq_step(
            EqMode::Graphic10BandPedal,
            1000.0,
            |e, hi| e.set_input_gain_db(if hi { 12.0 } else { -12.0 }),
            forward,
        );
        cases.push(("eq.input_gain_db", forward, 1000.0, inp, out));

        let (inp, out) = run_eq_step(
            EqMode::Graphic10BandPedal,
            1000.0,
            |e, hi| e.set_output_level_db(if hi { 12.0 } else { -12.0 }),
            forward,
        );
        cases.push(("eq.output_level_db", forward, 1000.0, inp, out));
    }

    for forward in [true, false] {
        for freq in [350.0, 2200.0] {
            let (inp, out) = run_wah_step(
                freq,
                |w, hi| w.set_position(if hi { 1.0 } else { 0.0 }),
                forward,
            );
            cases.push(("wah.position", forward, freq, inp, out));
        }
        for freq in [1000.0, 900.0] {
            let (inp, out) = run_wah_step(
                freq,
                |w, hi| w.set_q(if hi { 10.0 } else { 1.0 }),
                forward,
            );
            cases.push(("wah.q", forward, freq, inp, out));
        }
    }

    cases
}

fn collect_aud_11_ramp_cases() -> Vec<(&'static str, bool, f32, Vec<f32>, Vec<f32>)> {
    let mut cases = Vec::new();
    let amp_specs: &[(&str, f32, AmpModelId, fn(&mut WhiteBoxAmpProcessor), fn(&mut WhiteBoxAmpProcessor, bool))] = &[
        ("amp.bass", 1000.0, AmpModelId::BritJcm800_2203, |_| {}, |a, hi| {
            a.panel_mut().set_bass(if hi { 10.0 } else { 0.0 });
        }),
        ("amp.middle", 1000.0, AmpModelId::BritJcm800_2203, |_| {}, |a, hi| {
            a.panel_mut().set_middle(if hi { 10.0 } else { 0.0 });
        }),
        ("amp.treble", 1000.0, AmpModelId::BritJcm800_2203, |_| {}, |a, hi| {
            a.panel_mut().set_treble(if hi { 10.0 } else { 0.0 });
        }),
        ("amp.presence", 4000.0, AmpModelId::BritJcm800_2203, |_| {}, |a, hi| {
            a.panel_mut().set_presence(if hi { 10.0 } else { 0.0 });
        }),
        ("amp.master", 1000.0, AmpModelId::BritJcm800_2203, |a| {
            a.set_sag_amount(0.0);
        }, |a, hi| {
            a.panel_mut().set_master_volume(if hi { 9.5 } else { 0.5 });
        }),
        ("amp.volume_2", 1000.0, AmpModelId::UkAc30TopBoost, |_| {}, |a, hi| {
            a.panel_mut().set_volume_2(if hi { 9.5 } else { 0.5 });
        }),
        ("amp.cut", 4000.0, AmpModelId::UkAc30TopBoost, |a| {
            // Desabilita sag da retificadora GZ34 (tau_rel = 80 ms) para isolar a rampa de 20 ms do controle Cut
            a.set_sag_amount(0.0);
        }, |a, hi| {
            a.panel_mut().set_cut(if hi { 10.0 } else { 0.0 });
        }),
        (
            "amp.speaker_resonance_hz",
            200.0,
            AmpModelId::BritJcm800_2203,
            |a| {
                a.set_sag_amount(0.0);
                a.set_transformer_saturation(0.0);
            },
            |a, hi| {
                a.set_speaker_resonance_hz(if hi { 135.0 } else { 55.0 });
            },
        ),
    ];

    for &(nome, freq, model, init, func) in amp_specs {
        for forward in [true, false] {
            let (inp, out) = run_amp_step_configured(model, freq, init, func, forward);
            cases.push((nome, forward, freq, inp, out));
        }
    }

    for forward in [true, false] {
        let (inp, out) = run_cab_hicut_step(4500.0, forward);
        cases.push(("cab.hicut_khz", forward, 4500.0, inp, out));

        let (inp, out) = run_eq_step(
            EqMode::Graphic10BandPedal,
            1000.0,
            |e, hi| e.set_graphic_band_db(6, if hi { 12.0 } else { -12.0 }),
            forward,
        );
        cases.push(("eq.graphic_band_gain", forward, 1000.0, inp, out));

        let (inp, out) = run_eq_step(
            EqMode::Parametric8Band,
            1000.0,
            |e, hi| e.set_parametric_band(3, 1000.0, if hi { 12.0 } else { -12.0 }, 1.5),
            forward,
        );
        cases.push(("eq.param_gain", forward, 1000.0, inp, out));

        let (inp, out) = run_eq_step(
            EqMode::Parametric8Band,
            1000.0,
            |e, hi| e.set_parametric_band(3, if hi { 1000.0 } else { 300.0 }, 9.0, 2.0),
            forward,
        );
        cases.push(("eq.param_freq", forward, 1000.0, inp, out));

        let (inp, out) = run_eq_step(
            EqMode::Parametric8Band,
            1400.0,
            |e, hi| e.set_parametric_band(3, 1000.0, 9.0, if hi { 6.0 } else { 0.7 }),
            forward,
        );
        cases.push(("eq.param_q", forward, 1400.0, inp, out));

        let (inp, out) = run_eq_step(
            EqMode::Graphic10BandPedal,
            1000.0,
            |e, hi| e.set_input_gain_db(if hi { 12.0 } else { -12.0 }),
            forward,
        );
        cases.push(("eq.input_gain_db", forward, 1000.0, inp, out));

        let (inp, out) = run_eq_step(
            EqMode::Graphic10BandPedal,
            1000.0,
            |e, hi| e.set_output_level_db(if hi { 12.0 } else { -12.0 }),
            forward,
        );
        cases.push(("eq.output_level_db", forward, 1000.0, inp, out));

        let (inp, out) = run_wah_step(
            2200.0,
            |w, hi| w.set_position(if hi { 1.0 } else { 0.0 }),
            forward,
        );
        cases.push(("wah.position", forward, 2200.0, inp, out));

        let (inp, out) = run_wah_step(
            900.0,
            |w, hi| w.set_q(if hi { 10.0 } else { 1.0 }),
            forward,
        );
        cases.push(("wah.q", forward, 900.0, inp, out));
    }

    cases
}

#[test]
fn aud_11_degrau_sem_zipper() {
    let cases = collect_aud_11_step_cases();
    let mut falhas = Vec::new();
    let mut max_ratio = 0.0_f32;

    for (nome, forward, freq, _inp, out) in &cases {
        let ratio = razao_transiente(out, EVENTO_AMOSTRA, SR);
        max_ratio = max_ratio.max(ratio);
        if ratio > 2.0 {
            let dir = if *forward { "min->max" } else { "max->min" };
            falhas.push(format!("{nome} ({dir}, {freq} Hz): razao_transiente={ratio:.3}"));
        }
    }

    assert!(
        falhas.is_empty(),
        "AUD-11 zipper detectado (max={max_ratio:.3} > 2.0):\n{}",
        falhas.join("\n")
    );
}

#[test]
fn aud_11_rampa_termina_em_25ms() {
    let cases = collect_aud_11_ramp_cases();
    let s_ini = EVENTO_AMOSTRA - 1_440; // 30 ms antes do degrau (480 amostras = 10 ms)
    let s_25ms = EVENTO_AMOSTRA + (SR * 0.025) as usize; // 25 ms após degrau
    let s_fim = 45_600; // 950 ms (regime final garantido)
    let win = (SR * 0.010) as usize; // 10 ms = 480 amostras

    let mut falhas = Vec::new();
    for (nome, forward, freq, inp, out) in &cases {
        let g_ini = ganho_db(&inp[s_ini..s_ini + win], &out[s_ini..s_ini + win], SR, *freq);
        let g_25 = ganho_db(&inp[s_25ms..s_25ms + win], &out[s_25ms..s_25ms + win], SR, *freq);
        let g_fim = ganho_db(&inp[s_fim..s_fim + win], &out[s_fim..s_fim + win], SR, *freq);

        let dir = if *forward { "min->max" } else { "max->min" };
        if (g_fim - g_ini).abs() < 0.25 {
            falhas.push(format!(
                "{nome} ({dir}, {freq} Hz): variacao insuficiente |g_fim-g_ini|={:.3} dB",
                (g_fim - g_ini).abs()
            ));
        }
        let delta = (g_25 - g_fim).abs();
        if delta >= 0.1 {
            falhas.push(format!(
                "{nome} ({dir}, {freq} Hz): nao assentou em 25 ms (|g_25-g_fim|={delta:.3} dB)"
            ));
        }
    }

    assert!(
        falhas.is_empty(),
        "AUD-11 rampa excedeu 25 ms ou teste tautológico:\n{}",
        falhas.join("\n")
    );
}

#[test]
fn aud_11_zero_alloc() {
    verify_allocator_self_check();

    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(SR);
    amp.set_standby(false);

    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(SR);

    let mut eq_g = EqProcessor::new(EqMode::Graphic10BandPedal);
    eq_g.set_sample_rate(SR);

    let mut eq_p = EqProcessor::new(EqMode::Parametric8Band);
    eq_p.set_sample_rate(SR);

    let mut wah = WahProcessor::new();
    wah.set_sample_rate(SR);
    wah.set_mode(WahMode::CryBabyGcb95);
    wah.set_bypassed(false);

    let inp = [0.2_f32; 128];
    let mut out = [0.0_f32; 128];

    // Warm-up antes de medir
    for _ in 0..8 {
        amp.process(&inp, &mut out);
        cab.process(&inp, &mut out);
        eq_g.process(&inp, &mut out);
        eq_p.process(&inp, &mut out);
        wah.process(&inp, &mut out);
    }

    let (allocs, deallocs) = measure_allocs(|| {
        amp.panel_mut().set_bass(1.0);
        amp.panel_mut().set_middle(9.0);
        amp.panel_mut().set_treble(8.0);
        amp.panel_mut().set_presence(9.0);
        amp.panel_mut().set_master_volume(8.0);
        amp.panel_mut().set_volume_2(7.0);
        amp.panel_mut().set_cut(6.0);
        amp.set_speaker_resonance_hz(130.0);

        cab.set_hicut_khz(4.0);
        eq_g.set_graphic_band_db(6, 10.0);
        eq_g.set_input_gain_db(6.0);
        eq_g.set_output_level_db(-6.0);
        eq_p.set_parametric_band(3, 1200.0, 8.0, 4.0);
        wah.set_position(1.0);
        wah.set_q(8.0);

        for _ in 0..20 {
            amp.process(&inp, &mut out);
            cab.process(&inp, &mut out);
            eq_g.process(&inp, &mut out);
            eq_p.process(&inp, &mut out);
            wah.process(&inp, &mut out);
        }
    });

    assert_eq!(allocs, 0, "AUD-11: alocação durante rampa/process ({allocs})");
    assert_eq!(
        deallocs, 0,
        "AUD-11: desalocação durante rampa/process ({deallocs})"
    );
}

use ampforge_dsp::fx::pitch::{PitchModel, PitchProcessor};

#[test]
fn aud_12_seco_sem_atraso() {
    let mut p = PitchProcessor::new(PitchModel::PedalWhammy);
    p.prepare(SR);
    p.set_mix_pct(50.0);
    p.set_semitones(0.0);

    let mut inp = vec![0.0_f32; 2048];
    inp[0] = 1.0;
    let mut out = vec![0.0_f32; 2048];
    p.process(&inp, &mut out);

    assert_eq!(
        p.algorithmic_latency_samples(),
        0,
        "AUD-12: PitchProcessor algorithmic_latency_samples deve ser 0 (caminho seco sem atraso)"
    );
    assert!(
        out[0].abs() >= 0.45,
        "AUD-12: com mix=50% e impulso unitário em idx 0, primeiro pico seco deve sair em out[0] (obteve out[0]={:.5}, pico_idx={})",
        out[0],
        out.iter()
            .enumerate()
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .map(|(i, _)| i)
            .unwrap_or(0)
    );
}

#[test]
fn aud_12_latencia_wet() {
    for sr in [44_100.0_f32, 48_000.0, 96_000.0] {
        let mut p = PitchProcessor::new(PitchModel::PedalWhammy);
        p.prepare(sr);
        p.set_mix_pct(100.0);
        p.set_semitones(0.0);

        let n = 4096usize;
        let mut inp = vec![0.0_f32; n];
        inp[0] = 1.0;
        let mut out = vec![0.0_f32; n];
        p.process(&inp, &mut out);

        let peak_idx = out
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .map(|(i, _)| i)
            .unwrap_or(0);
        let expected_samples = (512.0 * (sr / 48_000.0)).round() as isize;
        let diff = (peak_idx as isize - expected_samples).abs();
        assert!(
            diff <= 2,
            "AUD-12 (sr={sr}): pico wet deve ficar em {expected_samples} +/- 2 amostras (obteve {peak_idx})"
        );

        let expected_ms = (512.0 * (sr / 48_000.0)) / sr * 1000.0;
        assert!(
            (p.wet_latency_ms() - expected_ms).abs() < 0.1,
            "AUD-12 (sr={sr}): wet_latency_ms={:.3} ms divergiu do esperado {expected_ms:.3} ms",
            p.wet_latency_ms()
        );
    }
}

#[test]
fn aud_12_custo_distribuido() {
    let mut p = PitchProcessor::new(PitchModel::IntelligentDualHarmonizer);
    p.prepare(SR);
    p.set_mix_pct(50.0);

    let total = (SR * 5.0) as usize;
    let bloco = 64usize;
    let mut inp = vec![0.0_f32; bloco];
    let mut out = vec![0.0_f32; bloco];

    let mut ops_por_bloco = Vec::with_capacity(total / bloco + 1);
    let mut pos = 0usize;
    while pos < total {
        let len = bloco.min(total - pos);
        for i in 0..len {
            let t = (pos + i) as f32 / SR;
            inp[i] = 0.25 * (2.0 * PI * 220.0 * t).sin();
        }
        p.process(&inp[..len], &mut out[..len]);
        ops_por_bloco.push(p.ops_ultima_chamada as f64);
        pos += len;
    }

    let soma: f64 = ops_por_bloco.iter().sum();
    let media = soma / (ops_por_bloco.len() as f64);
    let max_ops = ops_por_bloco.iter().copied().fold(0.0_f64, f64::max);
    let razao = if media > 0.0 { max_ops / media } else { f64::INFINITY };
    assert!(
        media > 0.0 && razao <= 1.25,
        "AUD-12: pico de autocorrelação por bloco alto demais: max={max_ops:.1}, media={media:.2}, max/media={razao:.3} (> 1.25)"
    );
}

#[test]
fn aud_12_zero_alloc() {
    verify_allocator_self_check();

    let mut p = PitchProcessor::new(PitchModel::IntelligentDualHarmonizer);
    p.prepare(SR);
    p.set_mix_pct(50.0);

    let inp: Vec<f32> = (0..64)
        .map(|i| 0.25 * (2.0 * PI * 220.0 * (i as f32) / SR).sin())
        .collect();
    let mut out = [0.0_f32; 64];
    for _ in 0..8 {
        p.process(&inp, &mut out);
    }

    let (allocs, deallocs) = measure_allocs(|| {
        for _ in 0..40 {
            p.process(&inp, &mut out);
        }
    });

    assert_eq!(allocs, 0, "AUD-12: Pitch alocou {allocs} vez(es)");
    assert_eq!(deallocs, 0, "AUD-12: Pitch desalocou {deallocs} vez(es)");
}


#[test]
fn aud_14_banda_passante() {
    for &sr in &[44100.0_f32, 48000.0, 88200.0, 96000.0] {
        for &os in &[2usize, 4, 8] {
            let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
            amp.set_sample_rate(sr);
            amp.set_oversampling_factor(os);

            let pair_16k =
                amp.measure_ai_filter_magnitude_db(16_000.0) + amp.measure_aa_filter_magnitude_db(16_000.0);
            let pair_18k =
                amp.measure_ai_filter_magnitude_db(18_000.0) + amp.measure_aa_filter_magnitude_db(18_000.0);

            assert!(
                pair_16k >= -0.5 && pair_16k <= 0.5,
                "AUD-14 passband 16kHz fora de [-0.5, +0.5] dB em sr={sr} os={os}x: {pair_16k:.3} dB"
            );
            assert!(
                pair_18k >= -1.5 && pair_18k <= 1.5,
                "AUD-14 passband 18kHz fora de [-1.5, +1.5] dB em sr={sr} os={os}x: {pair_18k:.3} dB"
            );
        }
    }
}

#[test]
fn aud_14_banda_de_rejeicao() {
    for &sr in &[44100.0_f32, 48000.0, 88200.0, 96000.0] {
        for &os in &[2usize, 4, 8] {
            let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
            amp.set_sample_rate(sr);
            amp.set_oversampling_factor(os);

            for &frac in &[0.60_f64, 0.75, 0.90] {
                let f = (sr as f64) * frac;
                let aa_db = amp.measure_aa_filter_magnitude_db(f);
                let ai_db = amp.measure_ai_filter_magnitude_db(f);
                assert!(
                    aa_db <= -60.0,
                    "AUD-14 stopband AA em sr={sr} os={os}x f={f:.1}Hz ({frac}*fs): {aa_db:.2} dB (> -60 dB)"
                );
                assert!(
                    ai_db <= -60.0,
                    "AUD-14 stopband AI em sr={sr} os={os}x f={f:.1}Hz ({frac}*fs): {ai_db:.2} dB (> -60 dB)"
                );
            }
        }
    }
}

#[test]
fn aud_15_saida_igual_a_convolucao_direta() {
    use ampforge_dsp::cab::{ImpulseResponse, VirtualCabProcessor};

    let ir_len = 24_000usize;
    let mut ir_samples = vec![0.0_f32; ir_len];
    for (i, s) in ir_samples.iter_mut().enumerate() {
        let t = i as f32 / 48_000.0;
        *s = (-t * 14.0).exp() * (2.0 * PI * 430.0 * t).sin() * 0.15;
    }
    ir_samples[0] = 0.5;
    ir_samples[128] = -0.22;
    ir_samples[513] = 0.11;

    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48_000.0);
    cab.set_custom_ir(ImpulseResponse::from_samples(48_000.0, ir_samples.clone()));
    cab.set_ir_enabled(true);
    cab.set_hicut_khz(18.0);

    let sig_len = 4096usize;
    let mut input = vec![0.0_f32; sig_len];
    for (i, s) in input.iter_mut().enumerate() {
        let t = i as f32 / 48_000.0;
        *s = 0.4 * (2.0 * PI * 190.0 * t).sin() + 0.25 * (2.0 * PI * 1150.0 * t).cos();
    }

    let mut out = vec![0.0_f32; sig_len];
    let mut pos = 0usize;
    while pos < sig_len {
        let end = (pos + 32).min(sig_len);
        cab.process(&input[pos..end], &mut out[pos..end]);
        pos = end;
    }

    let mut max_err = 0.0_f32;
    for n in 0..sig_len {
        let mut ref_y = 0.0_f64;
        let k_max = n.min(ir_len - 1);
        for k in 0..=k_max {
            ref_y += (input[n - k] as f64) * (ir_samples[k] as f64);
        }
        let err = (out[n] - ref_y as f32).abs();
        if err > max_err {
            max_err = err;
        }
    }
    assert!(
        max_err < 1e-4,
        "AUD-15: erro máximo vs convolução direta = {max_err:.6} (>= 1e-4)"
    );
}

#[test]
fn aud_15_custo_distribuido() {
    use ampforge_dsp::cab::{ImpulseResponse, VirtualCabProcessor};

    let ir_len = 48_000usize; // 500 ms @ 96 kHz
    let mut ir_samples = vec![0.0_f32; ir_len];
    for (i, s) in ir_samples.iter_mut().enumerate() {
        let t = i as f32 / 96_000.0;
        *s = (-t * 10.0).exp() * (2.0 * PI * 310.0 * t).sin() * 0.05;
    }

    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(96_000.0);
    cab.set_custom_ir(ImpulseResponse::from_samples(96_000.0, ir_samples));
    cab.set_ir_enabled(true);

    let inp = [0.1_f32; 32];
    let mut out = [0.0_f32; 32];
    // Aquecimento para preencher FDL (48000 amostras)
    for _ in 0..(48_000 / 32 + 4) {
        cab.process(&inp, &mut out);
    }

    let mut macs = Vec::with_capacity(64);
    for _ in 0..64 {
        cab.process(&inp, &mut out);
        macs.push(cab.ultima_chamada_macs() as f64);
    }
    let media = macs.iter().sum::<f64>() / (macs.len() as f64);
    let max_macs = macs.iter().copied().fold(0.0_f64, f64::max);
    let razao = if media > 0.0 { max_macs / media } else { f64::INFINITY };
    assert!(
        media > 0.0 && razao <= 1.25,
        "AUD-15: pico de MACs por bloco de 32 alto demais: max={max_macs:.1}, media={media:.2}, max/media={razao:.3} (> 1.25)"
    );
}

#[test]
fn aud_15_zero_alloc() {
    use ampforge_dsp::cab::{ImpulseResponse, VirtualCabProcessor};

    verify_allocator_self_check();

    let ir_samples = vec![0.01_f32; 48_000];
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(96_000.0);
    cab.set_custom_ir(ImpulseResponse::from_samples(96_000.0, ir_samples));
    cab.set_ir_enabled(true);

    let inp = [0.1_f32; 32];
    let mut out = [0.0_f32; 32];
    for _ in 0..16 {
        cab.process(&inp, &mut out);
    }

    let (allocs, deallocs) = measure_allocs(|| {
        for _ in 0..32 {
            cab.process(&inp, &mut out);
        }
    });
    assert_eq!(allocs, 0, "AUD-15: Cab IR alocou {allocs} vez(es)");
    assert_eq!(deallocs, 0, "AUD-15: Cab IR desalocou {deallocs} vez(es)");
}
