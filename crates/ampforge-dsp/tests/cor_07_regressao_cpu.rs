use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::cab::VirtualCabProcessor;
use ampforge_dsp::fx::{DriveModel, DriveProcessor};

// COR-09: contador por thread, mesmo esquema da Spec 037 CA-7
// (contract_037_realtime_safety.rs). Só conta o que acontece no thread
// que ligou a contagem; o executor de testes e os demais casos do binário
// não interferem.
struct CountingAlloc;

thread_local! {
    static COUNTING_ACTIVE: Cell<bool> = const { Cell::new(false) };
    static THREAD_ALLOCS: Cell<usize> = const { Cell::new(0) };
    static THREAD_DEALLOCS: Cell<usize> = const { Cell::new(0) };
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
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

fn measure_allocs<F: FnOnce()>(f: F) -> (usize, usize) {
    THREAD_ALLOCS.with(|c| c.set(0));
    THREAD_DEALLOCS.with(|c| c.set(0));
    COUNTING_ACTIVE.with(|a| a.set(true));
    f();
    COUNTING_ACTIVE.with(|a| a.set(false));
    (
        THREAD_ALLOCS.with(|c| c.get()),
        THREAD_DEALLOCS.with(|c| c.get()),
    )
}

fn gen_signal(len: usize, sr: f32) -> Vec<f32> {
    (0..len)
        .map(|i| {
            let t = i as f32 / sr;
            0.25 * (2.0 * std::f32::consts::PI * 220.0 * t).sin()
                + 0.15 * (2.0 * std::f32::consts::PI * 880.0 * t).sin()
        })
        .collect()
}

#[test]
fn cor_07_zero_alloc() {
    let sr = 96_000.0;
    let block = 32;
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);

    let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive.set_sample_rate(sr);
    drive.set_gain(6.5);

    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp.set_sample_rate(sr);

    let inp = gen_signal(block, sr);
    let mut out = vec![0.0f32; block];

    // Warmup
    cab.process(&inp, &mut out);
    drive.process(&inp, &mut out);
    amp.process(&inp, &mut out);

    let (allocs, deallocs) = measure_allocs(|| {
        for _ in 0..2000 {
            drive.process(&inp, &mut out);
            amp.process(&inp, &mut out);
            cab.process(&inp, &mut out);
        }
    });
    assert_eq!(allocs, 0, "COR-09 zero_alloc: alocações em Drive/Amp/Cab");
    assert_eq!(deallocs, 0, "COR-09 zero_alloc: desalocações em Drive/Amp/Cab");
}
