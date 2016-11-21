//! ARMv7-M System Control Block support.

use arm_m::reg::Reg;

#[repr(C, packed)]
struct Registers {
    pub cpuid:   Reg<u32>,
    pub icsr:    Reg<u32>,
    pub vtor:    Reg<u32>,
    pub aircr:   Reg<u32>,
    pub scr:     Reg<u32>,
    pub ccr:     Reg<u32>,
    pub shpr:    [Reg<u32>; 3],
    pub shcsr:   Reg<u32>,
    pub cfsr:    Reg<u32>,
    pub hfsr:    Reg<u32>,
    pub dfsr:    Reg<u32>,
    pub mmfar:   Reg<u32>,
    pub bfar:    Reg<u32>,
    pub afsr:    Reg<u32>,

    _reserved:   [Reg<u32>; 18],

    pub cpacr:   Reg<u32>,
}

const SCB_ADDRESS : usize = 0xe000ed00;

pub struct Scb;

pub static SCB : Scb = Scb;

bit_wrappers! {
    pub struct Cpacr(pub u32);
}

bit_enums! {
    pub bit_enum CpAccess {
        None = 0b00,
        Privileged = 0b01,
        Full = 0b11,
    }
}

impl Cpacr {
    bitfield_accessors! {
        pub [23:22] get_cp11 / with_cp11: CpAccess,
        pub [21:20] get_cp10 / with_cp10: CpAccess,
    }
}

macro_rules! reg_accessors {
    ($name:ident, $ty:ident, $read:ident, $write:ident, $update:ident) => {
        pub fn $read(&self) -> $ty {
            $ty(self.reg().$name.get())
        }

        pub fn $write(&self, v: $ty) {
            self.reg().$name.set(v.0)
        }

        pub fn $update<F: FnOnce($ty) -> $ty>(&self, f: F) {
            self.$write(f(self.$read()))
        }
    };
}

impl Scb {
    fn reg(&self) -> &'static Registers {
        unsafe { &*(SCB_ADDRESS as *const Registers) }
    }

    reg_accessors!(cpacr, Cpacr, read_cpacr, write_cpacr, update_cpacr);
}

#[cfg(feature = "cpu:cortex-m4f")]
#[repr(C, packed)]
struct FpRegisters {
    pub fpccr:   Reg<u32>,
    pub fpcar:   Reg<u32>,
    pub fpdscr:  Reg<u32>,
    pub mvfr:    [Reg<u32>; 2],
}

#[cfg(feature = "cpu:cortex-m4f")]
const SCB_FP_ADDRESS : usize = 0xe000ef34;

#[cfg(feature = "cpu:cortex-m4f")]
pub struct ScbFp;

#[cfg(feature = "cpu:cortex-m4f")]
pub static SCB_FP : ScbFp = ScbFp;

bit_wrappers! {
    pub struct Fpccr(pub u32);
}

impl Fpccr {
    bitfield_accessors! {
        pub [31] get_aspen / with_aspen: bool,
        pub [30] get_lspen / with_lspen: bool,

        pub [8] get_monrdy / with_monrdy: bool,
        pub [6] get_bfrdy / with_bfrdy: bool,
        pub [5] get_mmrdy / with_mmrdy: bool,
        pub [4] get_hfrdy / with_hfrdy: bool,
        pub [3] get_thread / with_thread: bool,
        pub [1] get_user / with_user: bool,
        pub [0] get_lspact / with_lspact: bool,
    }
}

impl ScbFp {
    fn reg(&self) -> &'static FpRegisters {
        unsafe { &*(SCB_FP_ADDRESS as *const FpRegisters) }
    }

    reg_accessors!(fpccr, Fpccr, read_fpccr, write_fpccr, update_fpccr);
}


