//! Lock-free Single-Producer Single-Consumer (SPSC) Queues for Command and Telemetry
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DspCommand {
    SetParameter {
        block_id: usize,
        param_id: usize,
        value: f32,
    },
    BypassBlock {
        block_id: usize,
        bypassed: bool,
    },
    SelectChannel {
        block_id: usize,
        channel_index: usize,
    },
    SwapGraph {
        topology_id: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TelemetrySnapshot {
    pub cpu_usage_pct: f32,
    pub input_peak_db_l: f32,
    pub input_peak_db_r: f32,
    pub output_peak_db_l: f32,
    pub output_peak_db_r: f32,
    pub gain_reduction_db: f32,
}

impl Default for TelemetrySnapshot {
    fn default() -> Self {
        Self {
            cpu_usage_pct: 0.0,
            input_peak_db_l: -120.0,
            input_peak_db_r: -120.0,
            output_peak_db_l: -120.0,
            output_peak_db_r: -120.0,
            gain_reduction_db: 0.0,
        }
    }
}

pub struct SpscBuffer<T: Copy, const CAP: usize> {
    buffer: Box<[UnsafeCell<MaybeUninit<T>>; CAP]>,
    head: AtomicUsize,
    _pad1: [u8; 56],
    tail: AtomicUsize,
    _pad2: [u8; 56],
}

unsafe impl<T: Copy + Send, const CAP: usize> Sync for SpscBuffer<T, CAP> {}
unsafe impl<T: Copy + Send, const CAP: usize> Send for SpscBuffer<T, CAP> {}

impl<T: Copy, const CAP: usize> SpscBuffer<T, CAP> {
    pub fn new() -> Self {
        assert!(CAP > 1 && (CAP & (CAP - 1)) == 0, "CAP deve ser potência de 2");
        let buffer = Box::new(std::array::from_fn(|_| UnsafeCell::new(MaybeUninit::uninit())));
        Self {
            buffer,
            head: AtomicUsize::new(0),
            _pad1: [0; 56],
            tail: AtomicUsize::new(0),
            _pad2: [0; 56],
        }
    }

    #[inline]
    pub fn push(&self, item: T) -> Result<(), T> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        if head.wrapping_sub(tail) >= CAP {
            return Err(item); // Fila cheia
        }

        let idx = head & (CAP - 1);
        unsafe {
            let slot = self.buffer[idx].get();
            (*slot).write(item);
        }

        self.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    #[inline]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head.wrapping_sub(tail)
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        CAP
    }

    #[inline]
    pub fn remaining_capacity(&self) -> usize {
        CAP.saturating_sub(self.len())
    }

    #[inline]
    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if tail == head {
            return None; // Fila vazia
        }

        let idx = tail & (CAP - 1);
        let val = unsafe {
            let slot = self.buffer[idx].get();
            (*slot).assume_init_read()
        };

        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(val)
    }

    /// Push com descarte do elemento mais antigo se estiver cheio (ideal para telemetria em tempo real)
    #[inline]
    pub fn push_overwrite(&self, item: T) {
        let head = self.head.load(Ordering::Relaxed);
        let mut tail = self.tail.load(Ordering::Acquire);

        while head.wrapping_sub(tail) >= CAP {
            match self.tail.compare_exchange_weak(
                tail,
                tail.wrapping_add(1),
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(actual) => tail = actual,
            }
        }

        let idx = head & (CAP - 1);
        unsafe {
            let slot = self.buffer[idx].get();
            (*slot).write(item);
        }
        self.head.store(head.wrapping_add(1), Ordering::Release);
    }
}

impl<T: Copy, const CAP: usize> Default for SpscBuffer<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SpscCommandQueue {
    inner: Arc<SpscBuffer<DspCommand, 2048>>,
}

impl SpscCommandQueue {
    pub fn new() -> (Self, Self) {
        let inner = Arc::new(SpscBuffer::new());
        (Self { inner: Arc::clone(&inner) }, Self { inner })
    }

    #[inline]
    pub fn push(&self, cmd: DspCommand) -> Result<(), DspCommand> {
        self.inner.push(cmd)
    }

    #[inline]
    pub fn pop(&self) -> Option<DspCommand> {
        self.inner.pop()
    }
}

pub struct SpscTelemetryQueue {
    inner: Arc<SpscBuffer<TelemetrySnapshot, 512>>,
}

impl SpscTelemetryQueue {
    pub fn new() -> (Self, Self) {
        let inner = Arc::new(SpscBuffer::new());
        (Self { inner: Arc::clone(&inner) }, Self { inner })
    }

    #[inline]
    pub fn push_overwrite(&self, snapshot: TelemetrySnapshot) {
        self.inner.push_overwrite(snapshot);
    }

    #[inline]
    pub fn pop(&self) -> Option<TelemetrySnapshot> {
        self.inner.pop()
    }
}

/// Fila SPSC lock-free genérica para transferência de propriedade de recursos pesados
/// (IR, buffers DI, tabelas pré-alocadas) entre threads sem alocação/desalocação na thread de áudio.
pub struct SpscOwnedQueue<T, const CAP: usize> {
    buffer: Box<[UnsafeCell<MaybeUninit<T>>; CAP]>,
    head: AtomicUsize,
    _pad1: [u8; 56],
    tail: AtomicUsize,
    _pad2: [u8; 56],
}

unsafe impl<T: Send, const CAP: usize> Sync for SpscOwnedQueue<T, CAP> {}
unsafe impl<T: Send, const CAP: usize> Send for SpscOwnedQueue<T, CAP> {}

impl<T, const CAP: usize> SpscOwnedQueue<T, CAP> {
    pub fn new() -> Self {
        assert!(CAP > 1 && (CAP & (CAP - 1)) == 0, "CAP deve ser potência de 2");
        let buffer = Box::new(std::array::from_fn(|_| UnsafeCell::new(MaybeUninit::uninit())));
        Self {
            buffer,
            head: AtomicUsize::new(0),
            _pad1: [0; 56],
            tail: AtomicUsize::new(0),
            _pad2: [0; 56],
        }
    }

    #[inline]
    pub fn push(&self, item: T) -> Result<(), T> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head.wrapping_sub(tail) >= CAP {
            return Err(item);
        }
        let idx = head & (CAP - 1);
        unsafe {
            let slot = self.buffer[idx].get();
            (*slot).write(item);
        }
        self.head.store(head.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    #[inline]
    pub fn pop(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        if tail == head {
            return None;
        }
        let idx = tail & (CAP - 1);
        let val = unsafe {
            let slot = self.buffer[idx].get();
            (*slot).assume_init_read()
        };
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(val)
    }
}

impl<T, const CAP: usize> Drop for SpscOwnedQueue<T, CAP> {
    fn drop(&mut self) {
        while let Some(item) = self.pop() {
            drop(item);
        }
    }
}

impl<T, const CAP: usize> Default for SpscOwnedQueue<T, CAP> {
    fn default() -> Self {
        Self::new()
    }
}
