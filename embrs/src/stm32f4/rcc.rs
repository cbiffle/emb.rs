//! Reset and Clock Control (RCC) support.

use arm_m;
use arm_m::reg::{AtomicReg, Reg};

/// At startup, before the RCC has been reconfigured, the STM32F4 runs at 16MHz.
pub const BOOT_CLOCK_HZ : u32 = 16_000_000;

/// The RCC's hardware register layout.
#[repr(C, packed)]
struct Registers {
    cr:            Reg<u32>,
    pllcfgr:       Reg<u32>,
    cfgr:          Reg<u32>,
    cir:           Reg<u32>,
    ahb1rstr:      Reg<u32>,
    ahb2rstr:      Reg<u32>,
    ahb3rstr:      Reg<u32>,
    _reserved_1c:  Reg<u32>,
    apb1rstr:      Reg<u32>,
    apb2rstr:      Reg<u32>,
    _reserved_28:  Reg<u32>,
    _reserved_2c:  Reg<u32>,
    ahb1enr:       Reg<u32>,
    ahb2enr:       Reg<u32>,
    ahb3enr:       Reg<u32>,
    _reserved_3c:  Reg<u32>,
    apb1enr:       Reg<u32>,
    apb2enr:       Reg<u32>,
    _reserved_48:  Reg<u32>,
    _reserved_4c:  Reg<u32>,
    ahb1lpenr:     Reg<u32>,
    ahb2lpenr:     Reg<u32>,
    ahb3lpenr:     Reg<u32>,
    _reserved_5c:  Reg<u32>,
    apb1lpenr:     Reg<u32>,
    apb2lpenr:     Reg<u32>,
    _reserved_68:  Reg<u32>,
    _reserved_6c:  Reg<u32>,
    bdcr:          Reg<u32>,
    csr:           Reg<u32>,
    _reserved_78:  Reg<u32>,
    _reserved_7c:  Reg<u32>,
    sscgr:         Reg<u32>,
    plli2scfgr:    Reg<u32>,
    pllsaicfgr:    Reg<u32>,
    dckcfgr:       Reg<u32>,
}

const RCC_ADDRESS : usize = 0x40023800_usize;

/// RCC driver.
pub struct Rcc;

/// Describes types that name peripherals in the RCC.  This is used to fake
/// overloading on `ApbPeripheral` and `AhbPeripheral`.  It isn't designed to
/// be implemented by types outside this module.
pub trait PeripheralName {
    /// Alters the RCC to enable the clock for the named peripheral.
    ///
    /// # Panics
    ///
    /// If the named peripheral's clock cannot be controlled.  Controllable
    /// clocks have a bit allocated in one of the RCC's `AxBxENR` registers.
    /// Check the STM32F4 Reference Manual.
    fn enable_clock(self, rcc: &Rcc);
}

impl Rcc {
    fn reg(&self) -> &Registers {
        unsafe {
            &*(RCC_ADDRESS as *const Registers)
        }
    }

    /// Enables clock to peripheral `p` if that clock can be controlled.
    ///
    /// The implementation uses barriers to ensure that the clock is enabled
    /// before return.  This works around ST's erratum 2.1.13.
    ///
    /// # Panics
    ///
    /// If `p`'s clock cannot be controlled.  Controllable clocks have a bit
    /// allocated in one of the RCC's `AxBxENR` registers.  Check the STM32F4
    /// Reference Manual.
    pub fn enable_clock<P: PeripheralName>(&self, p: P) {
        // re-dispatch for bus-specific behavior
        p.enable_clock(self);
        // ensure the write took effect.
        arm_m::data_synchronization_barrier();
    }
}

/// Names the processor's AHB buses.  This can be seen as a bounded-range
/// integer type if you squint.
#[derive(Copy, Clone)]
pub enum AhbBus {
    Ahb1,
    Ahb2,
    Ahb3,
}

/// Names the processor's AHB-connected peripherals.
#[derive(Copy, Clone, Debug)]
pub enum AhbPeripheral {
    GpioA,
    GpioB,
    GpioC,
    GpioD,
    // TODO: this is obviously not comprehensive!
}

impl AhbPeripheral {
    /// Retrieves metadata about a peripheral: its bus, and the bit indices
    /// for its reset and clock-enable control bits.
    ///
    /// Note: In C++ I would have encoded this information into the enum value.
    /// I technically have that option in Rust, but doing it this way exposes
    /// more information to the compiler and is leading to better code
    /// generation, with fewer unnecessary range checks, in practice.
    pub fn describe(self) -> (AhbBus, Option<u32>, Option<u32>) {
        match self {
            AhbPeripheral::GpioA => (AhbBus::Ahb1, Some(0), Some(0)),
            AhbPeripheral::GpioB => (AhbBus::Ahb1, Some(1), Some(1)),
            AhbPeripheral::GpioC => (AhbBus::Ahb1, Some(2), Some(2)),
            AhbPeripheral::GpioD => (AhbBus::Ahb1, Some(3), Some(3)),
        }
    }
}

impl PeripheralName for AhbPeripheral {
    fn enable_clock(self, rcc: &Rcc) {
        if let (bus, _, Some(ena)) = self.describe() {
            let reg = match bus {
                AhbBus::Ahb1 => &rcc.reg().ahb1enr,
                AhbBus::Ahb2 => &rcc.reg().ahb2enr,
                AhbBus::Ahb3 => &rcc.reg().ahb3enr,
            };

            reg.atomic_or(1 << ena)
        } else {
            panic!("cannot control clock for {:?}", self)
        }
    }
}

/// Names the processor's APB buses.  This can be seen as a bounded-range
/// integer type if you squint.
#[derive(Copy, Clone)]
pub enum ApbBus {
    Apb1,
    Apb2,
}

/// Names the processor's APB-connected peripherals.
#[derive(Copy, Clone)]
pub enum ApbPeripheral {
    Tim2,
    Tim3,
    // TODO: this is obviously not comprehensive!
}

impl ApbPeripheral {
    /// Retrieves metadata about a peripheral: its bus, and its control bit
    /// index in the reset and clock-enable registers.
    ///
    /// (APB peripherals use consistent reset and clock-enable bit positions,
    /// so we don't have to model them separately as we do for AHB.  Also, all
    /// the APB peripherals allow both clock and reset control, so we don't need
    /// `Option`.  This also improves code generation.)
    pub fn describe(self) -> (ApbBus, u32) {
        match self {
            ApbPeripheral::Tim2 => (ApbBus::Apb1, 0),
            ApbPeripheral::Tim3 => (ApbBus::Apb1, 1),
        }
    }
}

impl PeripheralName for ApbPeripheral {
    fn enable_clock(self, rcc: &Rcc) {
        let (bus, idx) = self.describe();
        let reg = match bus {
            ApbBus::Apb1 => &rcc.reg().apb1enr,
            ApbBus::Apb2 => &rcc.reg().apb2enr,
        };

        reg.atomic_or(1 << idx)
    }
}

/// Shared instance of the `Rcc` driver.
pub static RCC: Rcc = Rcc;
