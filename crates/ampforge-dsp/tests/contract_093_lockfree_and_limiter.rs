//! Spec 093 — CA-1 (TripleBuffer Lock-Free) & CA-6 (OutputSoftLimiter Zero-Latency)

use ampforge_dsp::limiter::OutputSoftLimiter;
use ampforge_dsp::queue::TripleBuffer;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

#[derive(Clone, Debug, Default, PartialEq)]
struct TrackedSnapshot {
    seq: u64,
    rms: f32,
}

#[test]
fn test_ca_01_triple_buffer_spsc_latest_snapshot_zero_ub_and_drop_exact() {
    static ACTIVE_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct DropCounter {
        seq: u64,
    }
    impl Clone for DropCounter {
        fn clone(&self) -> Self {
            ACTIVE_COUNT.fetch_add(1, Ordering::Relaxed);
            Self { seq: self.seq }
        }
    }
    impl DropCounter {
        fn new(seq: u64) -> Self {
            ACTIVE_COUNT.fetch_add(1, Ordering::Relaxed);
            Self { seq }
        }
    }
    impl Drop for DropCounter {
        fn drop(&mut self) {
            ACTIVE_COUNT.fetch_sub(1, Ordering::Relaxed);
        }
    }

    {
        let (mut tx, mut rx) = TripleBuffer::new(DropCounter::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let stop_tx = Arc::clone(&stop);

        let producer = thread::spawn(move || {
            for seq in 1..=50_000u64 {
                tx.publish(DropCounter::new(seq));
            }
            stop_tx.store(true, Ordering::Release);
        });

        let mut last_seq = 0u64;
        while !stop.load(Ordering::Acquire) {
            if let Some(snap) = rx.latest() {
                assert!(
                    snap.seq >= last_seq,
                    "TripleBuffer deve entregar snapshots monotonicamente mais recentes ({} < {})",
                    snap.seq,
                    last_seq
                );
                last_seq = snap.seq;
            }
        }
        producer.join().expect("producer join");
        if let Some(snap) = rx.latest() {
            assert!(snap.seq >= last_seq);
            last_seq = snap.seq;
        }
        assert_eq!(last_seq, 50_000, "último snapshot disponível após término");
    }

    assert_eq!(
        ACTIVE_COUNT.load(Ordering::SeqCst),
        0,
        "TripleBuffer não pode vazar nem dar double-drop em valores com Drop"
    );

    let (mut tx2, mut rx2) = TripleBuffer::new(TrackedSnapshot::default());
    tx2.publish(TrackedSnapshot { seq: 42, rms: 0.25 });
    let read = rx2.latest().expect("snapshot lido");
    assert_eq!(read.seq, 42);
    assert!((read.rms - 0.25).abs() < 1e-6);
}

#[test]
fn test_ca_06_output_soft_limiter_transparent_below_minus_1dbfs_and_bounded() {
    let mut limiter = OutputSoftLimiter::new();

    // 1. Transparência em -12 dBFS (~0.25118864)
    let amp_m12 = 10.0_f32.powf(-12.0 / 20.0);
    for i in 0..1024 {
        let x = amp_m12 * ((i as f32) * 0.1).sin();
        let y = limiter.process_sample(x);
        assert!(
            (y - x).abs() < 1e-6,
            "Sinal em -12 dBFS deve ser 100% transparente (x={x}, y={y})"
        );
    }

    // 2. +3 dBFS: soft-knee gera menos distorção que hard clamp e permanece estritamente <= 1.0
    let amp_p3 = 10.0_f32.powf(3.0 / 20.0);
    let mut max_abs = 0.0_f32;
    let mut sum_diff_soft = 0.0_f64;
    let mut sum_diff_hard = 0.0_f64;
    for i in 0..2048 {
        let x = amp_p3 * ((i as f32) * 0.07).sin();
        let y = limiter.process_sample(x);
        assert!(y.is_finite() && y.abs() <= 1.0, "Saída fora de [-1.0, 1.0]: {y}");
        max_abs = max_abs.max(y.abs());
        let hard = x.clamp(-1.0, 1.0);
        // Acima de -1.0 dBFS, o joelho suave começa a comprimir antes do teto
        if x.abs() > 0.9 {
            sum_diff_soft += (y - hard).abs() as f64;
        }
        sum_diff_hard += hard.abs() as f64;
    }
    assert!(max_abs <= 1.0 && max_abs > 0.90, "Teto deve ficar entre 0.90 e 1.0 (obtido {max_abs})");
    assert!(sum_diff_soft > 0.0 && sum_diff_hard > 0.0);

    // 3. +12 dBFS e NaN/Inf
    assert!(limiter.process_sample(4.0).abs() <= 1.0);
    assert!(limiter.process_sample(-4.0).abs() <= 1.0);
    assert_eq!(limiter.process_sample(f32::NAN), 0.0);
    assert_eq!(limiter.process_sample(f32::INFINITY), 0.0);
    assert_eq!(limiter.process_sample(f32::NEG_INFINITY), 0.0);
}
