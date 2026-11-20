//! Controle de números subnormais (denormals) em hardware e *flush* explícito de estados (AUD-02).

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn ler_mxcsr() -> u32 {
    let mut v: u32 = 0;
    unsafe {
        core::arch::asm!(
            "stmxcsr [{p}]",
            p = in(reg) &mut v as *mut u32,
            options(nostack, preserves_flags)
        );
    }
    v
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn escrever_mxcsr(v: u32) {
    unsafe {
        core::arch::asm!(
            "ldmxcsr [{p}]",
            p = in(reg) &v as *const u32,
            options(nostack, preserves_flags, readonly)
        );
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
fn ler_mxcsr() -> u32 {
    0
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
fn escrever_mxcsr(_: u32) {}

pub const FTZ_DAZ: u32 = 0x8040; // bit 15 = FTZ, bit 6 = DAZ

pub struct DenormalGuard {
    anterior: u32,
}

impl DenormalGuard {
    #[inline(always)]
    pub fn new() -> Self {
        let anterior = ler_mxcsr();
        escrever_mxcsr(anterior | FTZ_DAZ);
        Self { anterior }
    }
}

impl Default for DenormalGuard {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DenormalGuard {
    #[inline(always)]
    fn drop(&mut self) {
        escrever_mxcsr(self.anterior);
    }
}

pub fn mxcsr_atual() -> u32 {
    ler_mxcsr()
}

#[inline(always)]
pub fn flush_f32(x: f32) -> f32 {
    if x.abs() < 1e-30 {
        0.0
    } else {
        x
    }
}

#[inline(always)]
pub fn flush_f64(x: f64) -> f64 {
    if x.abs() < 1e-30 {
        0.0
    } else {
        x
    }
}
