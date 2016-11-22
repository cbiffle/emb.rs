//! An example program for the STM32F4DISCOVERY board.

#![feature(asm)]
#![feature(const_fn)]
#![feature(lang_items)]

#![no_std]
#![no_main]

extern crate embrs;

use embrs::arm_m::{self, exc, sys_tick};
use embrs::stm32f4::rcc::{self, RCC, AhbPeripheral, ApbPeripheral};
use embrs::stm32f4::gpio::{self, GPIOA, GPIOD};

/******************************************************************************/

// Application.

/// The PinMask for the STM32F4DISCOVERY LEDs.
fn led_pins() -> gpio::PinMask {
    gpio::P12 | gpio::P13
}

/// Frequency of toggling (= half frequency of blinking).
const TOGGLE_HZ : u32 = 10;
const HZ : u32 = 160_000_000;

const CLOCKS : rcc::ClockConfig = rcc::ClockConfig {
    crystal_hz: 8_000_000_f32,
    crystal_divisor: 4,
    vco_multiplier: 160,
    general_divisor: rcc::SysPrescaler::Div2,
    pll48_divisor: 4,

    ahb_divisor: None,
    apb1_divisor: Some(rcc::ApbPrescaler::Div4),
    apb2_divisor: Some(rcc::ApbPrescaler::Div2),

    flash_latency: 5,
};


/// The application entry point.
#[no_mangle]
pub extern fn embrs_main() -> ! {
    RCC.configure_clocks(&CLOCKS);

    init_leds();
    init_uart();

    // Configure the SysTick timer to generate interrupts at our toggle
    // frequency.
    let cycles_per_toggle = HZ / TOGGLE_HZ;
    sys_tick::SYS_TICK.write_rvr(cycles_per_toggle - 1);

    sys_tick::SYS_TICK.write_csr(
        sys_tick::SYS_TICK.read_csr()
        .with_enable(true)
        .with_tickint(true)
        .with_clksource(sys_tick::ClkSource::ProcessorClock));

    // Put the processor in an idle state waiting for interrupts from SysTick.
    loop {
        arm_m::wait_for_interrupt();
    }
}

fn init_leds() {
    // Enable clock to GPIOD so we can mess with its registers.
    RCC.enable_clock(AhbPeripheral::GpioD);

    // Configure our pins for push-pull digital output.
    GPIOD.set_mode(led_pins(), gpio::Mode::Gpio);
    GPIOD.set_output_type(led_pins(), gpio::OutputType::PushPull);
}

fn init_uart() {
    use embrs::stm32f4::usart::*;

    // Enable clock to USART2.
    RCC.enable_clock(ApbPeripheral::Usart2);

    USART2.update_cr1(|v| v.with_ue(true));

    let speeds = CLOCKS.compute_speeds();

    let clk = speeds.get_clock_for(ApbPeripheral::Usart2);
    let brr = (clk / 115200_f32 + 0.5) as u32;

    USART2.update_brr(|v| v.with_mantissa(brr >> 4)
                      .with_fraction(brr & 0xF));

    USART2.update_cr1(|v| v.with_te(true));

    RCC.enable_clock(AhbPeripheral::GpioA);
    // Configure its TX pin (PA2) as AF7
    GPIOA.set_alternate_function(gpio::P2, gpio::Function::AF7);
    GPIOA.set_mode(gpio::P2, gpio::Mode::Alternate);
}

/// Interrupt handler that toggles our LEDs.
extern "C" fn toggle_isr() {
    use embrs::stm32f4::usart::*;

    if GPIOD.get(led_pins()).is_empty() {
        USART2.send8(b'1');
        GPIOD.set(led_pins())
    } else {
        USART2.send8(b'0');
        GPIOD.clear(led_pins())
    }
}

extern "C" fn trap() { loop {} }
extern "C" {
    fn _reset_vector() -> !;
}

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
    sys_tick: Some(toggle_isr),

    .. exc::empty_exception_table(unsafe { &__STACK_BASE },
                                  _reset_vector)
};

/******************************************************************************/

// Application environment.

extern {
    /// This symbol is exported by the linker script, and defines the initial
    /// stack pointer.
    static __STACK_BASE: u32;
}
