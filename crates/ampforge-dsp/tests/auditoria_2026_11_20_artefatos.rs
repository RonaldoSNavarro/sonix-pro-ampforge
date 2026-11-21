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
