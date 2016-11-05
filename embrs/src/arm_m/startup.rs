//! Rust runtime startup support for ARMvx-M bare metal targets.
//!
//! This is not *inherently* ARM-specific, but it just so happens that the
//! ARMvx-M boot sequence establishes enough of Rust's ABI that we can do
//! startup without resorting to assembly code or linker magic.
//!
//! To make use of this module, call `initialize_runtime` *immediately* upon
//! entry to your reset vector:
//!
//! ```
//! pub unsafe fn reset_handler() -> ! {
//!     arm_m::startup::initialize_runtime();
//!     loop {}
//! }

#![no_builtins]

// Symbols exported by compatible linker scripts.
extern {
    static _data_load: u32;
    static mut _data: u32;
    static mut _edata: u32;
    static mut _bss: u32;
    static mut _ebss: u32;
}

/// Establishes the basic guarantees required by the Rust runtime, by
/// initializing any initialized variables and zeroing BSS.
///
/// # Safety
///
/// It is safe to call this exactly once, shortly after reset.  Using it later
/// would trash memory with unpredictable results.
#[inline(never)]
pub unsafe fn initialize_runtime() {
    initialize_data();
    zero_bss()
}

#[inline]
unsafe fn initialize_data() {
    let mut src: *const u32 = &_data_load;
    let mut dst: *mut u32 = &mut _data;

    while dst != &mut _edata {
        *dst = *src;
        dst = dst.offset(1);
        src = src.offset(1);
    }
}

#[inline]
unsafe fn zero_bss() {
    let mut dst: *mut u32 = &mut _bss;
    while dst != &mut _ebss {
        *dst = 0u32;
        dst = dst.offset(1);
    }
}
