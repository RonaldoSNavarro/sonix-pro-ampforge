//! Testes de contrato na cadeia em tempo real (`RealtimeDspChain`) para remoção de artefatos.
//! Auditoria 2026-11-20 (`AUD-10` AMP 2).

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::f32::consts::PI;
use std::hint::black_box;
use std::sync::{Arc, Mutex};

use ampforge_dsp::amp::AmpModelId;
use ampforge_dsp::regua::razao_transiente;
use serde_json::json;
use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{DspParamUpdate, RealtimeDspChain, TelemetryBuffer};

const SR: f32 = 48_000.0;
const TOTAL_AMOSTRAS: usize = 48_000;
const EVENTO_AMOSTRA: usize = 24_064;
const BLOCK_SIZE: usize = 128;

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
        v.push(7);
        black_box(&v);
    });
    assert!(
        allocs >= 1 && deallocs >= 1,
        "CountingAllocator self-check falhou: allocs={allocs}, deallocs={deallocs}"
    );
}

#[test]
fn aud_10_amp2_troca_de_modelo_sem_clique() {
    verify_allocator_self_check();

    let pares = [
        (AmpModelId::UsTwinBlackface65, AmpModelId::BritJcm800_2203),
        (AmpModelId::BritJcm800_2203, AmpModelId::UsTwinBlackface65),
        (AmpModelId::BritJcm800_2203, AmpModelId::UkAc30TopBoost),
        (AmpModelId::UkAc30TopBoost, AmpModelId::CaliDualRectoRevF),
        (AmpModelId::CaliDualRectoRevF, AmpModelId::BritPlexi1959),
        (AmpModelId::BritPlexi1959, AmpModelId::UsTwinBlackface65),
        (AmpModelId::UkAc30TopBoost, AmpModelId::UsTwinBlackface65),
        (AmpModelId::BritPlexi1959, AmpModelId::CaliDualRectoRevF),
    ];

    let mut input = vec![0.0_f32; TOTAL_AMOSTRAS];
    for (i, sample) in input.iter_mut().enumerate() {
        let t = i as f32 / SR;
        *sample = 0.2512 * (2.0 * PI * 220.0 * t).sin();
    }

    let mut falhas = Vec::new();
    let mut max_ratio = 0.0_f32;

    for (a, b) in pares {
        let mut chain = RealtimeDspChain::new(SR);
        chain.prepare(SR, 2048);
        let dsp = Arc::new(Mutex::new(chain));
        let pq = dsp.lock().unwrap().param_queue();
        let ctx = ApiContext::new(
            Arc::new(Mutex::new(None)),
            Arc::new(Mutex::new(AppState::default())),
            Arc::new(TelemetryBuffer::new(128)),
            Some(dsp.clone()),
            Some(pq.clone()),
            None,
            None,
        );

        let setup = json!({
            "gate.enabled": false,
            "amp_1.enabled": true,
            "amp_1.standby": true,
            "cab_1.enabled": false,
            "split.enabled": true,
            "split.mode": "ab",
            "amp_2.enabled": true,
            "amp_2.model": "twin_ab763",
            "cab_2.enabled": false,
            "merge.row_1_level_db": 0.0,
            "amp_2.gain": 4.5,
            "amp_2.bass": 5.0,
            "amp_2.middle": 5.0,
            "amp_2.treble": 5.0,
            "amp_2.master": 6.0
        })
        .to_string();
        let (st, _, body) = dispatch(&ctx, "/api/params", "POST", setup.as_bytes());
        assert_eq!(st, 200, "setup amp_2 falhou: {}", String::from_utf8_lossy(&body));

        assert!(pq.push(DspParamUpdate::Amp2Model(a)).is_ok());

        let mut output = vec![0.0_f32; TOTAL_AMOSTRAS];
        let mut ch = dsp.lock().unwrap();
        let mut pos = 0;
        while pos < EVENTO_AMOSTRA {
            let end = pos + BLOCK_SIZE;
            ch.process(&input[pos..end], &mut output[pos..end]);
            pos = end;
        }

        let (allocs, deallocs) = measure_allocs(|| {
            let _ = pq.push(DspParamUpdate::Amp2Model(b));
            let mut p = pos;
            while p < TOTAL_AMOSTRAS {
                let end = (p + BLOCK_SIZE).min(TOTAL_AMOSTRAS);
                ch.process(&input[p..end], &mut output[p..end]);
                p = end;
            }
        });
        assert_eq!(
            allocs, 0,
            "AUD-10 AMP 2 alocou {allocs} vez(es) em {a:?} -> {b:?}"
        );
        assert_eq!(
            deallocs, 0,
            "AUD-10 AMP 2 desalocou {deallocs} vez(es) em {a:?} -> {b:?}"
        );

        let ratio = razao_transiente(&output, EVENTO_AMOSTRA, SR);
        max_ratio = max_ratio.max(ratio);
        if ratio > 2.0 {
            falhas.push(format!("{a:?} -> {b:?}: razao_transiente={ratio:.3}"));
        }
    }

    assert!(
        falhas.is_empty(),
        "AUD-10 AMP 2 falhou (max={max_ratio:.3} > 2.0):\n{}",
        falhas.join("\n")
    );
}
