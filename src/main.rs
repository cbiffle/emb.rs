#![feature(asm)]
#![feature(const_fn)]
#![feature(lang_items)]

#![no_std]
#![no_main]

extern crate embrs;

use embrs::arm_m::{self, exc};
use embrs::stm32f4::rcc::{RCC, AhbPeripheral};
use embrs::stm32f4::gpio::{self, GPIOD};

/******************************************************************************/

// Items required by some part of core or the runtime.

/// This will be invoked on `panic!`.  We don't currently panic, and there are
/// some pieces missing to enable it (particularly `abort`), but the compiler
/// looks for this when `no_std` is enabled so we have to provide it.
#[lang = "panic_fmt"]
pub extern fn panic_fmt(_msg: core::fmt::Arguments,
                        _file: &'static str,
                        _line: u32) -> ! {
    loop {}
}

/******************************************************************************/

// Application environment.

extern {
    /// This symbol is exported by the linker script, and defines the initial
    /// stack pointer.
    static __STACK_BASE: u32;
}

/******************************************************************************/

// Application.

/// This function will be "called" by the processor at reset.  Note that none of
/// the C or Rust environment has been established --- in particular, this
/// function is responsible for initializing any global data it might need!
pub unsafe extern fn reset_handler() -> ! {
    arm_m::startup::initialize_runtime();
    app()
}

/// The application entry point.  We're no longer `unsafe`.
fn app() -> ! {
    RCC.enable_clock(AhbPeripheral::GpioD);

    let pins = gpio::P12 | gpio::P13;

    GPIOD.set_mode(pins, gpio::Mode::Gpio);
    GPIOD.set_output_type(pins, gpio::OutputType::PushPull);

    loop {
        GPIOD.set(pins);
        hackish_delay();
        GPIOD.clear(pins);
        hackish_delay();
    }
}

fn hackish_delay() {
    for _ in 0 .. 1000000 {
        unsafe { asm!("nop") }
    }
}

/// For predictability, I've mapped all architectural vectors to this routine.
/// Since we aren't enabling peripherals or faults, we can technically only take
/// NMI and HardFault --- but if someone builds on this code, they might trigger
/// something else.
extern "C" fn trap() { loop {} }

/// The ROM vector table.  This is marked as the program entry point in the
/// linker script, ensuring that any object reachable from this table is
/// preserved in the output binary.
///
/// This is placed in ROM by the linker script because of its assigned
/// `link_section`.  Note that it is not `mut`.
///
/// The `no_mangle` attribute is currently necessary for two reasons:
///
/// 1. To give the table an easily predictable name for use in the linker
///    script.
/// 2. Because `no_mangle` appears to, currently, be the only way to ensure that
///    this symbol is left visible to the linker.
#[no_mangle]
#[link_section=".isr_vector"]
pub static ISR_VECTORS : exc::ExceptionTable = exc::ExceptionTable {
    nmi: Some(trap),
    hard_fault: Some(trap),
    mm_fault: Some(trap),
    bus_fault: Some(trap),
    usage_fault: Some(trap),
    sv_call: Some(trap),
    debug_mon: Some(trap),
    pend_sv: Some(trap),
    sys_tick: Some(trap),

    .. exc::empty_exception_table(unsafe { &__STACK_BASE },
                                  reset_handler)
};
