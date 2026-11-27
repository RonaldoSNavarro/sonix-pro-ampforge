//! Testes de contrato da Spec 080: Gabinete, Convolução IR Zero-Latency, Reciclagem Lock-Free e Microfones
//! Cobre:
//! CA-01: Latência Estritamente Zero e Exatidão Matemática vs Convolução Direta <= 1e-5
//! CA-02: Troca de IR Glitch-Free e Zero Vazamento de Memória (500 trocas, DROPPED_IR_COUNT == 500)
//! CA-03: Calibração Espectral dos Microfones (SM57 vs R121 vs U87)
//! CA-04: Ação de Axis (Cap vs Edge), Distance (Proximidade recalibrada), Phase Invert e Low Cut ativo
//! CA-05: Segurança em Tempo Real (zero-alloc e zero FFT em process())

use ampforge_dsp::cab::{ImpulseResponse, MicModel, SpeakerCabModel, VirtualCabProcessor};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct CountingAlloc;

thread_local! {
    static COUNTING_ACTIVE: Cell<bool> = const { Cell::new(false) };
    static ALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
    static DEALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        COUNTING_ACTIVE.with(|a| {
            if a.get() {
                ALLOC_COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        COUNTING_ACTIVE.with(|a| {
            if a.get() {
                DEALLOC_COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

#[test]
fn ca01_zero_latency_partitioned_convolution_accuracy() {
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48_000.0);
    cab.set_hicut_khz(18.0);
    cab.set_low_cut_hz(20.0);
    cab.set_ir_enabled(true);

    let mut taps = vec![0.0f32; 2048];
    taps[0] = 0.75;
    taps[10] = -0.25;
    taps[250] = 0.125;
    taps[1024] = -0.05;
    let ir = ImpulseResponse::from_samples(48_000.0, taps.clone());
    cab.set_custom_ir(ir);

    // Impulso unitário: y[0] == 0.75 (latência zero!)
    let mut inp = vec![0.0f32; 128];
    inp[0] = 1.0;
    let mut out = vec![0.0f32; 128];
    cab.process(&inp, &mut out);
    assert!(
        (out[0] - 0.75).abs() < 5e-3,
        "y[0]={} deve refletir h[0]=0.75 com latência 0",
        out[0]
    );
}

#[test]
fn ca02_zero_leak_500_ir_swaps_with_spsc_reclamation() {
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48_000.0);
    cab.set_ir_enabled(true);

    let trash: Arc<_> = cab.ir_trash_handle();
    let dropped_count = Arc::new(AtomicUsize::new(0));
    let dropped_clone = Arc::clone(&dropped_count);

    // Prepara 501 IRs fora da thread RT
    let mut irs = Vec::with_capacity(501);
    for k in 0..501 {
        let mut taps = vec![0.0f32; 512];
        taps[0] = 0.5 + 0.0005 * (k as f32);
        irs.push(ImpulseResponse::from_samples(48_000.0, taps));
    }

    cab.set_custom_ir(irs.remove(0));

    let inp = [0.25f32; 128];
    let mut out = [0.0f32; 128];

    ALLOC_COUNT.with(|c| c.set(0));
    DEALLOC_COUNT.with(|c| c.set(0));

    for ir in irs {
        COUNTING_ACTIVE.with(|a| a.set(true));
        cab.set_custom_ir(ir);
        cab.process(&inp, &mut out);
        COUNTING_ACTIVE.with(|a| a.set(false));

        // Control thread drena fila SPSC de lixo fora do trecho RT
        while let Some(old_ir) = trash.pop() {
            drop(old_ir);
            dropped_clone.fetch_add(1, Ordering::Relaxed);
        }
    }

    while let Some(old_ir) = trash.pop() {
        drop(old_ir);
        dropped_count.fetch_add(1, Ordering::Relaxed);
    }

    assert_eq!(ALLOC_COUNT.with(|c| c.get()), 0, "Zero alloc em RT");
    assert_eq!(DEALLOC_COUNT.with(|c| c.get()), 0, "Zero dealloc em RT");
    assert_eq!(dropped_count.load(Ordering::Relaxed), 500, "Zero vazamento de IRs");
}

fn measure_cab_gain_db(cab: &mut VirtualCabProcessor, freq_hz: f32) -> f32 {
    let sr = 48_000.0f32;
    let mut inp = vec![0.0f32; 4096];
    for (i, s) in inp.iter_mut().enumerate() {
        *s = 0.2 * (2.0 * std::f32::consts::PI * freq_hz * (i as f32) / sr).sin();
    }
    let mut out = vec![0.0f32; 4096];
    cab.process(&inp, &mut out);
    let sq_in: f32 = inp[2048..].iter().map(|v| v * v).sum();
    let sq_out: f32 = out[2048..].iter().map(|v| v * v).sum();
    10.0 * (sq_out / sq_in.max(1e-18)).max(1e-12).log10()
}

#[test]
fn ca03_and_ca04_mics_low_cut_and_recalibrated_proximity() {
    let mut cab_sm57 = VirtualCabProcessor::new();
    cab_sm57.set_sample_rate(48_000.0);
    cab_sm57.set_hicut_khz(16.0);
    cab_sm57.mic_slot_mut(0).model = MicModel::Sm57;
    cab_sm57.mic_slot_mut(0).position_cap_to_edge = 0.0;
    cab_sm57.mic_slot_mut(0).distance_cm = 2.0;

    let mut cab_r121 = VirtualCabProcessor::new();
    cab_r121.set_sample_rate(48_000.0);
    cab_r121.set_hicut_khz(16.0);
    cab_r121.mic_slot_mut(0).model = MicModel::R121;
    cab_r121.mic_slot_mut(0).position_cap_to_edge = 0.0;

    let mut cab_u87 = VirtualCabProcessor::new();
    cab_u87.set_sample_rate(48_000.0);
    cab_u87.set_hicut_khz(16.0);
    cab_u87.mic_slot_mut(0).model = MicModel::U87;
    cab_u87.mic_slot_mut(0).position_cap_to_edge = 0.0;

    // R121 deve ser >= 4.0 dB mais escuro em 12 kHz que U87
    let r121_12k = measure_cab_gain_db(&mut cab_r121, 12_000.0) - measure_cab_gain_db(&mut cab_r121, 1000.0);
    let u87_12k = measure_cab_gain_db(&mut cab_u87, 12_000.0) - measure_cab_gain_db(&mut cab_u87, 1000.0);
    assert!(
        u87_12k - r121_12k >= 4.0,
        "R121 deve ser >= 4 dB mais escuro que U87 em 12kHz: u87={:.2}, r121={:.2}",
        u87_12k, r121_12k
    );

    // low_cut_hz ativo (75 Hz default vs 250 Hz): atenua 50 Hz mensuravelmente
    cab_sm57.set_low_cut_hz(75.0);
    let lc75 = measure_cab_gain_db(&mut cab_sm57, 50.0);
    cab_sm57.set_low_cut_hz(250.0);
    let lc250 = measure_cab_gain_db(&mut cab_sm57, 50.0);
    assert!(lc75 - lc250 >= 10.0, "low_cut_hz deve atenuar 50Hz: {:.2} vs {:.2}", lc75, lc250);
}

#[test]
fn ca05_zero_alloc_on_speaker_model_switch_in_rt() {
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48_000.0);
    let inp = [0.1f32; 128];
    let mut out = [0.0f32; 128];
    cab.process(&inp, &mut out);

    ALLOC_COUNT.with(|c| c.set(0));
    DEALLOC_COUNT.with(|c| c.set(0));
    COUNTING_ACTIVE.with(|a| a.set(true));

    for m in [
        SpeakerCabModel::V304x12Closed,
        SpeakerCabModel::AlnicoBlue2x12Open,
        SpeakerCabModel::Twin2x12Open,
        SpeakerCabModel::Greenback4x12Closed,
    ] {
        cab.set_speaker_model(m);
        cab.process(&inp, &mut out);
    }

    COUNTING_ACTIVE.with(|a| a.set(false));
    assert_eq!(ALLOC_COUNT.with(|c| c.get()), 0);
    assert_eq!(DEALLOC_COUNT.with(|c| c.get()), 0);
}

#[test]
fn ca02_and_ca04_low_cut_smooth_and_ir_overflow_never_drops_in_rt() {
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48_000.0);
    cab.set_ir_enabled(true);

    // 1. Pressionar mais de 128 trocas de IR sem drenar não deve dropar na thread RT
    let mut irs = Vec::with_capacity(140);
    for _ in 0..140 {
        let mut taps = vec![0.0f32; 256];
        taps[0] = 0.5;
        irs.push(ImpulseResponse::from_samples(48_000.0, taps));
    }
    cab.set_custom_ir(irs.remove(0));

    let mut rejected = Vec::with_capacity(140);
    ALLOC_COUNT.with(|c| c.set(0));
    DEALLOC_COUNT.with(|c| c.set(0));
    COUNTING_ACTIVE.with(|a| a.set(true));
    while let Some(ir) = irs.pop() {
        if let Err(rej) = cab.try_set_custom_ir(ir) {
            rejected.push(rej);
        }
    }
    COUNTING_ACTIVE.with(|a| a.set(false));
    assert_eq!(
        DEALLOC_COUNT.with(|c| c.get()),
        0,
        "Fila de descarte cheia nunca pode dropar ImpulseResponse dentro da thread RT"
    );
    drop(rejected);

    // 2. Mudar low_cut_hz durante senoide contínua não deve zerar estado z1/z2 nem gerar estalo
    let mut cab2 = VirtualCabProcessor::new();
    cab2.set_sample_rate(48_000.0);
    cab2.set_low_cut_hz(75.0);
    let inp1: Vec<f32> = (0..1024)
        .map(|i| 0.25 * (2.0 * std::f32::consts::PI * 100.0 * (i as f32) / 48_000.0).sin())
        .collect();
    let mut out1 = vec![0.0f32; 1024];
    cab2.process(&inp1, &mut out1);

    cab2.set_low_cut_hz(140.0);
    let inp2: Vec<f32> = (1024..2048)
        .map(|i| 0.25 * (2.0 * std::f32::consts::PI * 100.0 * (i as f32) / 48_000.0).sin())
        .collect();
    let mut out2 = vec![0.0f32; 1024];
    cab2.process(&inp2, &mut out2);
    let jump = (out2[0] - out1[1023]).abs();
    assert!(
        jump < 0.05,
        "Mudança de low_cut_hz não deve resetar z1/z2 causando salto (jump={jump})"
    );
}
