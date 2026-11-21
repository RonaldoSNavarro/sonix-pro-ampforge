use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::f32::consts::PI;
use std::fs;
use std::path::Path;

use sonix_plugin::{PluginFormat, PluginParamValues, SonixPluginParams, SonixPluginProcessor};

struct CountingAllocator;
thread_local! {
    static TRACK_ALLOC: Cell<bool> = const { Cell::new(false) };
    static ALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
    static DEALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        TRACK_ALLOC.with(|t| {
            if t.get() {
                ALLOC_COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        TRACK_ALLOC.with(|t| {
            if t.get() {
                DEALLOC_COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.dealloc(ptr, layout) }
    }
}

fn measure_allocs<F: FnOnce()>(f: F) -> (usize, usize) {
    ALLOC_COUNT.with(|c| c.set(0));
    DEALLOC_COUNT.with(|c| c.set(0));
    TRACK_ALLOC.with(|t| t.set(true));
    f();
    TRACK_ALLOC.with(|t| t.set(false));
    (
        ALLOC_COUNT.with(|c| c.get()),
        DEALLOC_COUNT.with(|c| c.get()),
    )
}

fn rms(buf: &[f32]) -> f32 {
    if buf.is_empty() {
        return 0.0;
    }
    (buf.iter().map(|&x| x * x).sum::<f32>() / buf.len() as f32).sqrt()
}

#[test]
fn aud_16_process_block_nao_aloca_com_midi() {
    let (a_chk, _) = measure_allocs(|| {
        let v = Vec::<u8>::with_capacity(64);
        std::hint::black_box(v);
    });
    assert!(a_chk >= 1, "CountingAllocator self-check falhou");

    let mut proc = SonixPluginProcessor::new(PluginFormat::Clap);
    proc.prepare(48_000.0, 256);
    proc.compile_default_modifiers();
    let nih_params = SonixPluginParams::default();
    let params = PluginParamValues::from(&nih_params);

    let in_l = [0.15_f32; 256];
    let in_r = [0.15_f32; 256];
    let mut out_l = [0.0_f32; 256];
    let mut out_r = [0.0_f32; 256];
    let mut midi_ccs = [(1u8, 0.5_f32); 64];
    for (i, slot) in midi_ccs.iter_mut().enumerate() {
        *slot = ((i % 16) as u8 + 1, (i as f32) / 64.0);
    }

    for _ in 0..8 {
        proc.process_block(
            &in_l,
            Some(&in_r),
            &mut out_l,
            &mut out_r,
            &params,
            &midi_ccs,
            None,
        );
    }

    let (allocs, deallocs) = measure_allocs(|| {
        for _ in 0..32 {
            proc.process_block(
                &in_l,
                Some(&in_r),
                &mut out_l,
                &mut out_r,
                &params,
                &midi_ccs,
                None,
            );
        }
    });
    assert_eq!(allocs, 0, "AUD-16: process_block alocou {allocs} vez(es) com MIDI");
    assert_eq!(deallocs, 0, "AUD-16: process_block desalocou {deallocs} vez(es) com MIDI");
}

#[test]
fn aud_16_todo_parametro_age() {
    let sr = 48_000.0;
    let block = 256usize;
    let total_blocks = 80usize;

    let run_case = |configure: &dyn Fn(&mut PluginParamValues),
                    is_gate: bool,
                    transport_bpm: Option<f64>|
     -> Vec<f32> {
        let mut proc = SonixPluginProcessor::new(PluginFormat::Vst3);
        proc.prepare(sr, block);
        let mut params = PluginParamValues::from(&SonixPluginParams::default());
        configure(&mut params);

        let mut input = vec![0.0_f32; block];
        let mut all_out = Vec::with_capacity(block * total_blocks);
        for b in 0..total_blocks {
            for i in 0..block {
                let n = b * block + i;
                let t = n as f32 / sr;
                let amp = if is_gate {
                    0.02
                } else if n < 4096 {
                    0.30
                } else {
                    0.0
                };
                input[i] = amp
                    * (0.5 * (2.0 * PI * 220.0 * t).sin() + 0.5 * (2.0 * PI * 3500.0 * t).sin());
            }
            let mut out_l = vec![0.0_f32; block];
            let mut out_r = vec![0.0_f32; block];
            proc.process_block(
                &input,
                Some(&input),
                &mut out_l,
                &mut out_r,
                &params,
                &[],
                transport_bpm,
            );
            all_out.extend_from_slice(&out_l);
        }
        all_out
    };

    type Case = (&'static str, bool, bool, fn(&mut PluginParamValues), fn(&mut PluginParamValues));
    let cases: &[Case] = &[
        ("amp_1.gain", false, false, |p| p.amp_gain = 0.2, |p| p.amp_gain = 9.8),
        ("amp_1.bass", false, false, |p| p.amp_bass = 0.0, |p| p.amp_bass = 10.0),
        ("amp_1.middle", false, false, |p| p.amp_middle = 0.0, |p| p.amp_middle = 10.0),
        ("amp_1.treble", false, false, |p| p.amp_treble = 0.0, |p| p.amp_treble = 10.0),
        ("amp_1.master", false, false, |p| p.amp_master = 0.5, |p| p.amp_master = 9.5),
        ("amp_1.presence", false, false, |p| p.amp_presence = 0.0, |p| p.amp_presence = 10.0),
        (
            "gate.threshold_db",
            false,
            true,
            |p| p.gate_threshold_db = -90.0,
            |p| p.gate_threshold_db = -10.0,
        ),
        ("dly_1.intensity", false, false, |p| p.dly_intensity = 0.0, |p| p.dly_intensity = 9.0),
        ("rev_1.mix", false, false, |p| p.rev_mix = 0.0, |p| p.rev_mix = 1.0),
        ("cab_1.enabled", true, false, |p| p.cab_enabled = false, |p| p.cab_enabled = true),
        (
            "host_tempo_sync",
            true,
            false,
            |p| {
                p.dly_intensity = 8.0;
                p.host_tempo_sync = false;
            },
            |p| {
                p.dly_intensity = 8.0;
                p.host_tempo_sync = true;
            },
        ),
    ];

    for &(id, is_discrete, is_gate, set_min, set_max) in cases {
        let bpm = if id == "host_tempo_sync" {
            Some(175.0)
        } else {
            None
        };
        let out_min = run_case(&set_min, is_gate, bpm);
        let out_max = run_case(&set_max, is_gate, bpm);

        let diff_rms = rms(&out_min
            .iter()
            .zip(out_max.iter())
            .map(|(a, b)| a - b)
            .collect::<Vec<_>>());
        if is_discrete {
            assert!(
                diff_rms > 1e-5,
                "AUD-16: parâmetro discreto {id} não alterou saída (diff_rms={diff_rms})"
            );
        } else {
            let r_min = rms(&out_min).max(1e-9);
            let r_max = rms(&out_max).max(1e-9);
            let delta_db = (20.0 * (r_max / r_min).log10()).abs();
            let diff_db_vs_sig = 20.0 * (diff_rms / r_min.max(r_max)).max(1e-9).log10();
            assert!(
                delta_db >= 0.5 || diff_db_vs_sig >= -20.0,
                "AUD-16: parâmetro {id} sem efeito >= 0.5 dB (delta_db={delta_db:.3}, diff_vs_sig={diff_db_vs_sig:.2} dB)"
            );
        }
    }
}

#[test]
fn aud_16_entrada_direita_e_usada() {
    let mut proc = SonixPluginProcessor::new(PluginFormat::Vst3);
    proc.prepare(48_000.0, 256);
    proc.set_stereo_input(true);
    let params = PluginParamValues::from(&SonixPluginParams::default());

    let in_l = [0.0_f32; 256];
    let in_r: Vec<f32> = (0..256)
        .map(|i| 0.4 * (2.0 * PI * 330.0 * (i as f32) / 48_000.0).sin())
        .collect();
    let mut out_l = [0.0_f32; 256];
    let mut out_r = [0.0_f32; 256];

    let mut max_rms = 0.0_f32;
    for _ in 0..8 {
        proc.process_block(
            &in_l,
            Some(&in_r),
            &mut out_l,
            &mut out_r,
            &params,
            &[],
            None,
        );
        max_rms = max_rms.max(rms(&out_l));
    }
    assert!(
        max_rms > 1e-3,
        "AUD-16: entrada R ignorada no layout 2->2 (max_rms={max_rms})"
    );
}

#[test]
fn aud_16_latencia_reportada() {
    let mut proc = SonixPluginProcessor::new(PluginFormat::Clap);
    proc.prepare(48_000.0, 256);
    let reported = proc.latency_samples();

    let mut in_l = [0.0_f32; 256];
    in_l[0] = 0.01;
    let mut out_l = [0.0_f32; 256];
    let mut out_r = [0.0_f32; 256];
    proc.process(&in_l, &in_l, &mut out_l, &mut out_r);
    let measured = ampforge_dsp::regua::atraso_por_impulso(&out_l) as u32;

    assert!(
        (reported as isize - measured as isize).abs() <= 1,
        "AUD-16: latência reportada={reported} diverge da medida={measured}"
    );
}

#[test]
fn aud_16_lint_process_chama_process_block() {
    let src = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"))
        .expect("ler crates/sonix-plugin/src/lib.rs");
    let impl_pos = src
        .find("impl Plugin for SonixProPlugin")
        .expect("impl Plugin for SonixProPlugin ausente");
    let after_impl = &src[impl_pos..];
    let proc_pos = after_impl
        .find("fn process(")
        .expect("fn process ausente em impl Plugin for SonixProPlugin");
    let body_start = after_impl[proc_pos..].find("-> ProcessStatus").unwrap() + proc_pos;
    let next_impl = after_impl[body_start..]
        .find("\nimpl ")
        .unwrap_or(after_impl.len() - body_start);
    let process_body = &after_impl[body_start..body_start + next_impl];

    assert!(
        process_body.contains("self.processor.process_block"),
        "AUD-16 lint: fn process deve delegar a self.processor.process_block"
    );
    assert!(
        !process_body.contains(".chain.set_"),
        "AUD-16 lint: fn process não deve chamar setters diretos da chain"
    );
    assert!(
        !src.contains("uses_embedded_webview_ipc"),
        "AUD-16 lint: uses_embedded_webview_ipc deve ser removido"
    );
}
