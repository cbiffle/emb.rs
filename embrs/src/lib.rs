#![feature(asm)]
#![feature(const_fn)]
#![feature(core_intrinsics)]

#![no_std]

#![deny(
    unused_import_braces,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    )]

pub mod arm_m;
pub mod stm32f4;
