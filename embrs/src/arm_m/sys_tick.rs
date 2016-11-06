//! ARMv7-M SysTick timer support.

use arm_m::reg::Reg;

#[repr(C, packed)]
struct Registers {
    csr:   Reg<u32>,
    rvr:   Reg<u32>,
    cvr:   Reg<u32>,
    calib: Reg<u32>,
}

const SYS_TICK_ADDRESS : usize = 0xe000e010;

pub struct SysTick;

impl SysTick {
    // TODO: this peripheral, unusually for something designed by ARM, contains
    // read-to-clear bits and R/W bits without inherent atomic updates.  So this
    // API is probably wrong.

    fn reg(&self) -> &'static Registers {
        unsafe { &*(SYS_TICK_ADDRESS as *const Registers) }
    }

    pub fn read_csr(&self) -> Csr {
        Csr(self.reg().csr.get())
    }

    pub fn write_csr(&self, v: Csr) {
        self.reg().csr.set(v.into())
    }

    pub fn read_rvr(&self) -> u32 {
        self.reg().rvr.get()
    }

    pub fn write_rvr(&self, v: u32) {
        self.reg().rvr.set(v)
    }

    pub fn read_cvr(&self) -> u32 {
        self.reg().cvr.get()
    }

    pub fn write_cvr(&self, v: u32) {
        self.reg().cvr.set(v)
    }
}

#[derive(Copy, Clone)]
pub struct Csr(u32);

#[derive(Copy, Clone)]
pub enum ClkSource {
    ExternalReference = 0,
    ProcessorClock = 1,
}

impl Csr {
    #[inline]
    pub fn get_enable(self) -> bool {
        (self.0 & (1 << 0)) != 0
    }

    #[inline]
    pub fn with_enable(self, v: bool) -> Self {
        Csr((self.0 & !(1 << 0)) | ((v as u32) << 0))
    }

    #[inline]
    pub fn get_tickint(self) -> bool {
        (self.0 & (1 << 1)) != 0
    }

    #[inline]
    pub fn with_tickint(self, v: bool) -> Self {
        Csr((self.0 & !(1 << 1)) | ((v as u32) << 1))
    }

    #[inline]
    pub fn get_clksource(self) -> ClkSource {
        match (self.0 & (1 << 2)) != 0 {
            false => ClkSource::ExternalReference,
            true => ClkSource::ProcessorClock,
        }
    }

    #[inline]
    pub fn with_clksource(self, v: ClkSource) -> Self {
        Csr((self.0 & !(1 << 2)) | ((v as u32) << 2))
    }

    #[inline]
    pub fn get_countflag(self) -> bool {
        (self.0 & (1 << 16)) != 0
    }
}

impl From<u32> for Csr {
    fn from(v: u32) -> Csr {
        Csr(v)
    }
}

impl From<Csr> for u32 {
    fn from(v: Csr) -> u32 {
        v.0
    }
}

pub static SYS_TICK : SysTick = SysTick;
