//! Testes de auditoria 2026-11-20 — Tempo real no host desktop (AUD-02, AUD-06, AUD-07)

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use ampforge_dsp::denormal::mxcsr_atual;
use sonix_desktop::audio::RealtimeDspChain;

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
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

fn measure_allocs<F: FnOnce()>(f: F) -> (usize, usize) {
    THREAD_ALLOCS.with(|c| c.set(0));
    THREAD_DEALLOCS.with(|c| c.set(0));
    COUNTING_ACTIVE.with(|a| a.set(true));
    f();
    COUNTING_ACTIVE.with(|a| a.set(false));
    let allocs = THREAD_ALLOCS.with(|c| c.get());
    let deallocs = THREAD_DEALLOCS.with(|c| c.get());
    (allocs, deallocs)
}

#[test]
fn aud_02_process_roda_com_ftz_daz() {
    let mut chain = RealtimeDspChain::new(48_000.0);
    chain.prepare(48_000.0, 128);
    let before = mxcsr_atual();

    let mut in_buf = vec![0.0_f32; 128];
    let mut out_buf = vec![0.0_f32; 128];
    for (i, s) in in_buf.iter_mut().enumerate() {
        *s = (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / 48_000.0)).sin() * 0.25;
    }

    chain.process(&in_buf, &mut out_buf);
    let after = mxcsr_atual();

    assert_eq!(
        chain.mxcsr_observado() & 0x8040,
        0x8040,
        "AUD-02: process() deve rodar com FTZ e DAZ (0x8040) ligados, observou 0x{:04x}",
        chain.mxcsr_observado()
    );
    assert_eq!(
        after, before,
        "AUD-02: MXCSR do thread chamador não deve ser alterado após process()"
    );
}

#[test]
fn aud_02_zero_alloc() {
    let mut chain = RealtimeDspChain::new(48_000.0);
    chain.prepare(48_000.0, 128);
    let in_buf = vec![0.01_f32; 128];
    let mut out_buf = vec![0.0_f32; 128];

    // Warmup fora da contagem
    for _ in 0..8 {
        chain.process(&in_buf, &mut out_buf);
    }

    let (allocs, deallocs) = measure_allocs(|| {
        for _ in 0..64 {
            chain.process(&in_buf, &mut out_buf);
        }
    });

    assert_eq!(allocs, 0, "AUD-02 zero_alloc falhou: {} alocações", allocs);
    assert_eq!(deallocs, 0, "AUD-02 zero_alloc falhou: {} desalocações", deallocs);
}
