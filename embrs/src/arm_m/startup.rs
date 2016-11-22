//! Rust runtime startup support for ARMvx-M bare metal targets.
//!
//! To make use of this module, specify `_reset_vector` as your reset vector,
//! and define your application entry point like so:
//!
//!
//! ```
//! #[no_mangle]
//! pub extern fn embrs_main() -> ! {
//!     // code here
//!     loop {}
//! }
//! ```

#![macro_use]

use arm_m;
use arm_m::scb::{self, SCB};

#[inline(never)]
#[no_mangle]
#[naked]
pub unsafe extern fn _reset_vector() {
    asm!(r#"
    .extern _data_load, _data, _edata, _bss, _ebss
    .extern _embrs_init_array_start, _embrs_init_array_end
    .extern embrs_main

    @ Initialize data.
    ldr r0, =_data_load
    ldr r1, =_data
    ldr r2, =_edata
    b 1f

0:  ldr r3, [r0], #4
    str r3, [r1], #4
1:  cmp r1, r2
    bne 0b

    @ Zero BSS.
    ldr r0, =_bss
    ldr r1, =_ebss
    movs r2, #0
    b 1f

0:  str r2, [r0], #4
1:  cmp r0, r1
    bne 0b

    @ Call any pre-main Rust hooks.
    ldr r4, =_embrs_init_array_start
    ldr r5, =_embrs_init_array_end
    b 1f

0:  ldr r0, [r4], #4
    blx r0
1:  cmp r4, r5
    bne 0b

    @ Jump to application main.
    movs r0, #0
    movs r1, #1
    b embrs_main
    "#)
}

/// The emb.rs startup routine can call functions after data is initialized, but
/// before main.  Functions must be of this type.
pub type InitHook = extern fn() -> ();

/// Defines one or more init hooks, which are functions that will be called
/// after the basic Rust runtime invariants have been established, but before
/// main.
///
/// Due to limitations in the Rust tooling, init hooks will produce `pub static`
/// symbols in their defining module, raising the possibility that clients could
/// call your init hooks directly.  Sorry.
///
/// Syntax:
///
/// ```
/// extern fn my_init_hook() {
///     activate_lasers()
/// }
///
/// embrs_init_hooks! {
///     pub init_hook MY_INIT_HOOK = my_init_hook;
/// }
/// ```
macro_rules! embrs_init_hooks {
    (
        $(
            $(#[$m:meta])*
            pub init_hook $name:ident = $f:ident;
        )*
    ) => {
        $(
            $(#[$m])*
            #[link_section = ".embrs_init_array"]
            #[no_mangle]
            #[allow(dead_code, private_no_mangle_statics)]
            pub static $name : $crate::arm_m::startup::InitHook = $f;
        )*
    };
}

#[cfg(feature = "cpu:cortex-m4f")]
extern fn enable_cortex_m4_fpu() {
    SCB.update_cpacr(|v| v.with_cp11(scb::CpAccess::Full)
                     .with_cp10(scb::CpAccess::Full));
    arm_m::instruction_synchronization_barrier()
}

embrs_init_hooks! {
    #[cfg(feature = "cpu:cortex-m4f")]
    pub init_hook EMBRS_FPU_ON = enable_cortex_m4_fpu;
}
