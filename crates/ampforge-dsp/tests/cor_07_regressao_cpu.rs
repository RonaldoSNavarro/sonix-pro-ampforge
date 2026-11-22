use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::cab::VirtualCabProcessor;
use ampforge_dsp::fx::{DriveModel, DriveProcessor};

struct CountingAlloc;
static COUNTING_ACTIVE: AtomicBool = AtomicBool::new(false);
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static TEST_SERIAL: Mutex<()> = Mutex::new(());

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING_ACTIVE.load(Ordering::Relaxed) {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if COUNTING_ACTIVE.load(Ordering::Relaxed) {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) }
    }
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
    let _guard = TEST_SERIAL.lock().unwrap();
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

    ALLOC_COUNT.store(0, Ordering::SeqCst);
    COUNTING_ACTIVE.store(true, Ordering::SeqCst);
    for _ in 0..2000 {
        drive.process(&inp, &mut out);
        amp.process(&inp, &mut out);
        cab.process(&inp, &mut out);
    }
    COUNTING_ACTIVE.store(false, Ordering::SeqCst);
    assert_eq!(
        ALLOC_COUNT.load(Ordering::SeqCst),
        0,
        "COR-07 zero_alloc falhou em Drive/Amp/Cab"
    );
}

#[test]
fn cor_07_cab_speaker_fir_saida_igual() {
    let _guard = TEST_SERIAL.lock().unwrap();
    let sr = 48_000.0;
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);
    let inp = gen_signal(512, sr);
    let mut out1 = vec![0.0f32; 512];
    cab.process(&inp, &mut out1);

    // Determinismo e invariância de bloco: 512 em blocos de 32 deve bater com bloco único de 512
    let mut cab2 = VirtualCabProcessor::new();
    cab2.set_sample_rate(sr);
    let mut out2 = vec![0.0f32; 512];
    for ch in 0..16 {
        let s = ch * 32;
        cab2.process(&inp[s..s + 32], &mut out2[s..s + 32]);
    }

    let mut max_rel = 0.0f32;
    for i in 0..512 {
        let denom = out1[i].abs().max(1e-6);
        let rel = (out1[i] - out2[i]).abs() / denom;
        if rel > max_rel {
            max_rel = rel;
        }
    }
    assert!(
        max_rel <= 1e-6,
        "COR-07 cab speaker FIR divergiu: max_rel={max_rel}"
    );
}

#[test]
fn cor_07_drive_ts808_saida_igual() {
    let _guard = TEST_SERIAL.lock().unwrap();
    let sr = 48_000.0;
    let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive.set_sample_rate(sr);
    drive.set_gain(6.5);
    let inp = gen_signal(512, sr);
    let mut out = vec![0.0f32; 512];
    drive.process(&inp, &mut out);

    // Verifica que soft_clip_8 com sqrt().sqrt().sqrt() preserva energia e determinismo
    let rms = (out.iter().map(|&x| x * x).sum::<f32>() / out.len() as f32).sqrt();
    assert!(rms > 0.05 && rms.is_finite(), "COR-07 TS808 RMS inválido: {rms}");

    let mut drive2 = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive2.set_sample_rate(sr);
    drive2.set_gain(6.5);
    let mut out2 = vec![0.0f32; 512];
    for ch in 0..16 {
        let s = ch * 32;
        drive2.process(&inp[s..s + 32], &mut out2[s..s + 32]);
    }
    let max_diff = out
        .iter()
        .zip(out2.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(max_diff <= 1e-6, "COR-07 TS808 max_diff={max_diff}");
}

#[test]
fn cor_07_amp_saida_igual() {
    let _guard = TEST_SERIAL.lock().unwrap();
    let sr = 48_000.0;
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp.set_sample_rate(sr);
    let inp = gen_signal(512, sr);
    let mut out1 = vec![0.0f32; 512];
    amp.process(&inp, &mut out1);

    let mut amp2 = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp2.set_sample_rate(sr);
    let mut out2 = vec![0.0f32; 512];
    for ch in 0..16 {
        let s = ch * 32;
        amp2.process(&inp[s..s + 32], &mut out2[s..s + 32]);
    }
    let max_diff = out1
        .iter()
        .zip(out2.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(max_diff <= 1e-6, "COR-07 amp max_diff={max_diff}");
}
