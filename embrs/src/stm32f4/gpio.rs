//! General Purpose I/O (GPIO) support.
//!
//! The STM32F4 GPIO unit is responsible for the pin-twiddling we usually
//! imagine as "GPIO," but also for routing peripheral functions to the outside
//! world.

#![allow(trivial_numeric_casts)]  // required for bitflags :-(

use arm_m::reg::{AtomicReg,Reg};

/// A GPIO port's memory mapped registers.  Instances of this type are placed by
/// the linker script to alias the actual registers.
#[repr(C, packed)]
struct Registers {
    moder:   Reg<u32>,
    otyper:  Reg<u32>,
    ospeedr: Reg<u32>,
    pupdr:   Reg<u32>,
    idr:     Reg<u32>,
    odr:     Reg<u32>,
    bsrr:    Reg<u32>,
    lckr:    Reg<u32>,
    afrl:    Reg<u32>,
    afrh:    Reg<u32>,
}

/// Possible modes of a GPIO pin.
#[derive(Clone, Copy)]
pub enum Mode {
    /// High-impedance state with a Schmitt trigger input filter.  `OutputType`
    /// and `OutputSpeed` are ignored in `Input` state, but `Pull` can be
    /// applied.
    Input     = 0b00,
    /// Digital output state, controlled through the GPIO port.
    Gpio      = 0b01,
    /// Peripheral alternate functions.
    Alternate = 0b10,
    /// Analog mode for use with the DAC and ADC.
    Analog    = 0b11,
}

/// Available pin output drive types.
#[derive(Clone, Copy)]
pub enum OutputType {
    /// Pin is driven both high and low.
    PushPull = 0,
    /// Pin is only pulled low, high side drive transistor is disabled.
    OpenDrain = 1,
}

/// Available pin output speeds.  This controls output slew rate filtering.
/// The mapping of these settings to transition speeds is part-specific and
/// can be found in the datasheet.
#[derive(Clone, Copy)]
pub enum Speed {
    Low       = 0b00,
    Medium    = 0b01,
    High      = 0b10,
    VeryHigh  = 0b11,
}

/// Pull up/down resistor configuration.
#[derive(Clone, Copy)]
pub enum Pull {
    /// No internal pull resistors enabled.
    None      = 0b00,
    /// Internal pull-up resistor enabled.
    Up        = 0b01,
    /// Internal pull-down resistor enabled.
    Down      = 0b10,
    /* 0b11 is reserved */
}

/// Alternate function selection.  The specific meaning of AFx depends on the
/// pin.
#[derive(Clone, Copy)]
pub enum Function {
    AF0, AF1, AF2, AF3, AF4, AF5, AF6, AF7,
    AF8, AF9, AF10, AF11, AF12, AF13, AF14, AF15,
}

bitflags! {
    /// Names a group of pins on a single GPIO port.  The STM32F4 GPIO is
    /// designed so that most operations can be applied to any subset of pins
    /// for the same cost as a single pin, so we expose that oddity here.
    /// All pin configuration/alteration methods take a `PinMask` to select
    /// the affected pins.
    pub flags PinMask: u16 {
        const P0 = 1 << 0,
        const P1 = 1 << 1,
        const P2 = 1 << 2,
        const P3 = 1 << 3,
        const P4 = 1 << 4,
        const P5 = 1 << 5,
        const P6 = 1 << 6,
        const P7 = 1 << 7,
        const P8 = 1 << 8,
        const P9 = 1 << 9,
        const P10 = 1 << 10,
        const P11 = 1 << 11,
        const P12 = 1 << 12,
        const P13 = 1 << 13,
        const P14 = 1 << 14,
        const P15 = 1 << 15,
    }
}

/// GPIO port driver.
pub struct GpioPort {
    reg: *const Registers,
}

unsafe impl Sync for GpioPort {}

impl GpioPort {
    /// Changes the mode of the pins selected by `pins` to `mode`.
    pub fn set_mode(&self, pins: PinMask, mode: Mode) {
        Self::update_2(pins, mode as u32, unsafe { &self.reg().moder })
    }

    /// Changes the output type of the pins selected by `pins` to `ot`.
    pub fn set_output_type(&self, pins: PinMask, ot: OutputType) {
        Self::update_1(pins, ot as u32, unsafe { &self.reg().otyper })
    }

    /// Changes the output speed of the pins selected by `pins` to `speed`.
    pub fn set_speed(&self, pins: PinMask, speed: Speed) {
        Self::update_2(pins, speed as u32, unsafe { &self.reg().ospeedr })
    }

    /// Changes the pull up/down configuration of the pins selected by `pins` to
    /// `pull`.
    pub fn set_pull(&self, pins: PinMask, pull: Pull) {
        Self::update_2(pins, pull as u32, unsafe { &self.reg().pupdr })
    }

    /// Selects an alternate function for the pins selected by `pins`.  For this
    /// to take effect, the pin's `Mode` must be set to `Alternate` using
    /// `set_mode`.
    pub fn set_alternate_function(&self, pins: PinMask, af: Function) {
        // This one's a bit hairier than the others, because the four-bit
        // select fields are split over two registers.  We process the two
        // individually using this helper.
        fn do_update(bits: u32, af: u32, reg: &Reg<u32>) {
            let (mask, setting) = {
                // See update_2 below for an explanation of this method.
                let mut places = 0u32;

                for i in 0..7 {
                    places |= (bits & (1 << i)) << (3 * i);
                }

                (0b1111 * places, af * places)
            };
            if mask != 0 { reg.atomic_nand_and_or(mask, setting) }
        }

        let af = af as u32;
        let pins = pins.bits() as u32;

        do_update(pins & 0xFF, af,
                  unsafe { &self.reg().afrl });
        do_update((pins >> 8) & 0xFF, af,
                  unsafe { &self.reg().afrh })
    }


    /// Reads the state of pins selected by `pins`.  The returned `PinMask`
    /// contains those pins that were observed as logic high.
    #[inline]
    pub fn get(&self, pins: PinMask) -> PinMask {
        PinMask::from_bits_truncate(
            unsafe { self.reg().idr.get() as u16 } & pins.bits())
    }

    /// Sets pins selected by `pins` to logic high.
    #[inline]
    pub fn set(&self, pins: PinMask) {
        unsafe { self.reg() }.bsrr.set(pins.bits() as u32)
    }

    /// Clears pins selected by `pins` to logic low.
    #[inline]
    pub fn clear(&self, pins: PinMask) {
        unsafe { self.reg() }.bsrr.set((pins.bits() as u32) << 16)
    }

    /// Internal shorthand for dereferencing our raw pointer.  This can vend
    /// an arbitrary number of apparently unique references to a single object,
    /// so it's marked `unsafe` here.  So long as we're careful about how we
    /// access the `Registers` it can be used correctly.
    unsafe fn reg(&self) -> &Registers {
        &*self.reg
    }

    /// Updates a word-packed array of 1-bit fields with `val`.  The elements
    /// that are updated are those included in `pins`; others are preserved.
    fn update_1(pins: PinMask, val: u32, reg: &Reg<u32>) {
        let mask = pins.bits() as u32;
        reg.atomic_nand_and_or(mask, mask * val)
    }

    /// Updates a word-packed array of 2-bit fields with `val`.  The elements
    /// that are updated are those included in `pins`; others are preserved.
    fn update_2(pins: PinMask, val: u32, reg: &Reg<u32>) {
        let (mask, setting) = {
            // This is admittedly a bit obscure, but generates the best code
            // of any implementation I've tried.  We exploit the fact that
            // 32-bit integer multiplication acts like 32 integer adds, each
            // controlled by one bit in the multiplicand.  Since multiplication
            // is super cheap on the M4, we can construct the mask and settings
            // values efficiently by generating `0b01` in each affected 2-bit
            // field and then multiplying.
            let pins = pins.bits() as u32;
            let mut places = 0u32;

            for i in 0..16 {
                places |= (pins & (1 << i)) << i;
            }

            (0b11 * places, val * places)
        };

        reg.atomic_nand_and_or(mask, setting)
    }
}

macro_rules! static_gpio {
    ($name:ident, $addr:expr) => {
        pub static $name: GpioPort = GpioPort {
            reg: $addr as *const Registers,
        };
    };
}

static_gpio!(GPIOD, 0x40020c00);
