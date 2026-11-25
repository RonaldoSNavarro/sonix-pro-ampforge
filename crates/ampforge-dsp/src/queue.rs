//! Lock-free Single-Producer Single-Consumer (SPSC) Queues for Command and Telemetry
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
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

// SAFETY: SpscBuffer garante acesso exclusivo por índice entre 1 produtor (head) e 1 consumidor (tail) quando T: Copy + Send.
unsafe impl<T: Copy + Send, const CAP: usize> Sync for SpscBuffer<T, CAP> {}
// SAFETY: SpscBuffer pode ser enviado entre threads quando T: Copy + Send.
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
        // SAFETY: `head - tail < CAP` garante que `idx` não está sendo lido pelo consumidor.
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
        // SAFETY: `tail != head` garante que `idx` contém item inicializado publicado pelo produtor.
        let val = unsafe {
            let slot = self.buffer[idx].get();
            (*slot).assume_init_read()
        };

        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        Some(val)
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

/// Estado empacotado do `TripleBuffer`: bits `0..2` = índice `ready` (`0..3`), bit `2` = flag `dirty`.
const TRIPLE_READY_MASK: u8 = 0b0011;
const TRIPLE_DIRTY_BIT: u8 = 0b0100;

struct TripleBufferInner<T> {
    slots: [UnsafeCell<T>; 3],
    state: AtomicU8,
    _pad: [u8; 63],
}

// SAFETY: O protocolo de troca de índices garante que produtor, consumidor e slot `ready` nunca
// acessam o mesmo índice simultaneamente; T: Send permite transferência entre threads.
unsafe impl<T: Send> Sync for TripleBufferInner<T> {}
// SAFETY: `TripleBufferInner<T>` só expõe acesso via protocolo SPSC quando `T: Send`.
unsafe impl<T: Send> Send for TripleBufferInner<T> {}

/// Triple Buffer lock-free SPSC (`O(1)`, zero alocações no caminho crítico, sem UB).
pub struct TripleBuffer<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Clone + Send> TripleBuffer<T> {
    pub fn new(initial: T) -> (TripleProducer<T>, TripleConsumer<T>) {
        let inner = Arc::new(TripleBufferInner {
            slots: [
                UnsafeCell::new(initial.clone()),
                UnsafeCell::new(initial.clone()),
                UnsafeCell::new(initial),
            ],
            state: AtomicU8::new(1), // ready = 1, dirty = 0; producer = 0, consumer = 2
            _pad: [0; 63],
        });
        (
            TripleProducer {
                inner: Arc::clone(&inner),
                write_idx: 0,
            },
            TripleConsumer {
                inner,
                read_idx: 2,
            },
        )
    }
}

pub struct TripleProducer<T> {
    inner: Arc<TripleBufferInner<T>>,
    write_idx: u8,
}

impl<T: Send> TripleProducer<T> {
    #[inline]
    pub fn publish(&mut self, value: T) {
        let idx = self.write_idx as usize;
        // SAFETY: `self.write_idx` é de posse exclusiva do produtor até o swap atômico abaixo.
        unsafe {
            *self.inner.slots[idx].get() = value;
        }
        let mut cur = self.inner.state.load(Ordering::Relaxed);
        loop {
            let next = (self.write_idx & TRIPLE_READY_MASK) | TRIPLE_DIRTY_BIT;
            match self.inner.state.compare_exchange_weak(
                cur,
                next,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(prev) => {
                    self.write_idx = prev & TRIPLE_READY_MASK;
                    break;
                }
                Err(actual) => cur = actual,
            }
        }
    }
}

pub struct TripleConsumer<T> {
    inner: Arc<TripleBufferInner<T>>,
    read_idx: u8,
}

impl<T: Send> TripleConsumer<T> {
    #[inline]
    pub fn latest(&mut self) -> Option<&T> {
        let mut cur = self.inner.state.load(Ordering::Acquire);
        if (cur & TRIPLE_DIRTY_BIT) == 0 {
            return None;
        }
        loop {
            let next = self.read_idx & TRIPLE_READY_MASK; // dirty = 0
            match self.inner.state.compare_exchange_weak(
                cur,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(prev) => {
                    self.read_idx = prev & TRIPLE_READY_MASK;
                    break;
                }
                Err(actual) => {
                    if (actual & TRIPLE_DIRTY_BIT) == 0 {
                        return None;
                    }
                    cur = actual;
                }
            }
        }
        let idx = self.read_idx as usize;
        // SAFETY: Após o swap atômico, `self.read_idx` é de posse exclusiva do consumidor até a próxima chamada.
        Some(unsafe { &*self.inner.slots[idx].get() })
    }
}

pub struct SpscTelemetryQueue {
    inner: Arc<TripleBufferInner<TelemetrySnapshot>>,
    write_idx: AtomicU8,
    read_idx: AtomicU8,
    _prod_marker: std::marker::PhantomData<TripleProducer<TelemetrySnapshot>>,
    _cons_marker: std::marker::PhantomData<TripleConsumer<TelemetrySnapshot>>,
}

impl SpscTelemetryQueue {
    pub fn new() -> (Self, Self) {
        let (prod, cons) = TripleBuffer::new(TelemetrySnapshot::default());
        let inner = Arc::clone(&prod.inner);
        let w = prod.write_idx;
        let r = cons.read_idx;
        (
            Self {
                inner: Arc::clone(&inner),
                write_idx: AtomicU8::new(w),
                read_idx: AtomicU8::new(r),
                _prod_marker: std::marker::PhantomData,
                _cons_marker: std::marker::PhantomData,
            },
            Self {
                inner,
                write_idx: AtomicU8::new(w),
                read_idx: AtomicU8::new(r),
                _prod_marker: std::marker::PhantomData,
                _cons_marker: std::marker::PhantomData,
            },
        )
    }

    #[inline]
    pub fn push_overwrite(&self, snapshot: TelemetrySnapshot) {
        let w = self.write_idx.load(Ordering::Relaxed);
        // SAFETY: SPSC — único produtor escreve no slot exclusivo `w` antes de publicá-lo em `state`.
        unsafe {
            *self.inner.slots[w as usize].get() = snapshot;
        }
        let mut cur = self.inner.state.load(Ordering::Relaxed);
        loop {
            let next = (w & TRIPLE_READY_MASK) | TRIPLE_DIRTY_BIT;
            match self.inner.state.compare_exchange_weak(
                cur,
                next,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(prev) => {
                    self.write_idx.store(prev & TRIPLE_READY_MASK, Ordering::Relaxed);
                    break;
                }
                Err(actual) => cur = actual,
            }
        }
    }

    #[inline]
    pub fn pop(&self) -> Option<TelemetrySnapshot> {
        let r = self.read_idx.load(Ordering::Relaxed);
        let mut cur = self.inner.state.load(Ordering::Acquire);
        if (cur & TRIPLE_DIRTY_BIT) == 0 {
            return None;
        }
        let new_r = loop {
            let next = r & TRIPLE_READY_MASK;
            match self.inner.state.compare_exchange_weak(
                cur,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(prev) => break prev & TRIPLE_READY_MASK,
                Err(actual) => {
                    if (actual & TRIPLE_DIRTY_BIT) == 0 {
                        return None;
                    }
                    cur = actual;
                }
            }
        };
        self.read_idx.store(new_r, Ordering::Relaxed);
        // SAFETY: SPSC — após trocar `new_r` para posse exclusiva do consumidor, lê cópia `Copy` do snapshot.
        Some(unsafe { *self.inner.slots[new_r as usize].get() })
    }
}

/// Fila SPSC lock-free genérica para transferência de propriedade de recursos pesados
/// (IR, buffers DI, tabelas pré-alocadas) entre threads sem alocação/desalocação na thread de áudio.
pub struct SpscOwnedQueue<T, const CAP: usize> {
    buffer: Box<[UnsafeCell<MaybeUninit<T>>]>,
    head: AtomicUsize,
    _pad1: [u8; 56],
    tail: AtomicUsize,
    _pad2: [u8; 56],
}

// SAFETY: SpscOwnedQueue garante acesso exclusivo por índice entre 1 produtor (head) e 1 consumidor (tail) quando T: Send.
unsafe impl<T: Send, const CAP: usize> Sync for SpscOwnedQueue<T, CAP> {}
// SAFETY: SpscOwnedQueue pode ser enviada entre threads quando T: Send.
unsafe impl<T: Send, const CAP: usize> Send for SpscOwnedQueue<T, CAP> {}

impl<T, const CAP: usize> SpscOwnedQueue<T, CAP> {
    pub fn new() -> Self {
        assert!(CAP > 1 && (CAP & (CAP - 1)) == 0, "CAP deve ser potência de 2");
        let mut slots = Vec::with_capacity(CAP);
        for _ in 0..CAP {
            slots.push(UnsafeCell::new(MaybeUninit::uninit()));
        }
        Self {
            buffer: slots.into_boxed_slice(),
            head: AtomicUsize::new(0),
            _pad1: [0; 56],
            tail: AtomicUsize::new(0),
            _pad2: [0; 56],
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head.wrapping_sub(tail).min(CAP)
    }

    #[inline]
    pub fn remaining_capacity(&self) -> usize {
        CAP.saturating_sub(self.len())
    }

    #[inline]
    pub fn push(&self, item: T) -> Result<(), T> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head.wrapping_sub(tail) >= CAP {
            return Err(item);
        }
        let idx = head & (CAP - 1);
        // SAFETY: `head - tail < CAP` garante que `idx` não está sendo lido pelo consumidor.
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
        // SAFETY: `tail != head` garante que `idx` contém item inicializado publicado pelo produtor.
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
