#![feature(asm)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(naked_functions)]

#![no_std]

#![no_builtins]

#![deny(
    unused_import_braces,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    )]

#![recursion_limit = "300"]

#[macro_use]
extern crate bitflags;

pub mod bits;

pub mod arm_m;
pub mod lang;
pub mod stm32f4;
