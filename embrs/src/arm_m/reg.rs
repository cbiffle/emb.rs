//! Support for memory-mapped registers of various sizes.

use core::intrinsics::{volatile_load, volatile_store};

/// A register whose contents can be represented as `T`.  The contents are
/// accessed using `volatile` operations only, ensuring that apparently dead
/// loads and stores are not optimized away.
///
/// `Reg` doesn't work like `Cell` by allowing interior mutability.  Mutations
/// to a `Reg` must come through a `&mut`.  This is due to complex interaction
/// between `#[repr(C)]`, `UnsafeCell`, and `volatile_load`.
#[repr(C, packed)]
pub struct Reg<T> {
    value: T,
}

impl<T> Reg<T> {
    /// Reads the contents of the register using a volatile load.
    pub fn get(&self) -> T {
        unsafe { volatile_load(&self.value) }
    }

    /// Replaces the contents of the register using a volatile store.
    pub fn set(&mut self, value: T) {
        unsafe {
            volatile_store(&mut self.value, value)
        }
    }
}

/// Additional features that become available when a register contains a
/// hardware-supported atomic type.
pub trait AtomicReg {
    type Type;

    /// Clears any bits in the register that are also set in `clear`.
    ///
    /// The effect is atomic from the perspective of other threads or
    /// interrupts; if there is a race (e.g. an interrupt) the update sequence
    /// will restart.  This means this function can produce many volatile loads,
    /// but only one store with the final result.
    fn atomic_nand(&mut self, clear: Self::Type);

    /// Sets any bits in the register that are also set in `set`.
    ///
    /// The effect is atomic from the perspective of other threads or
    /// interrupts; if there is a race (e.g. an interrupt) the update sequence
    /// will restart.  This means this function can produce many volatile loads,
    /// but only one store with the final result.
    fn atomic_or(&mut self, set: Self::Type);

    /// Clears any bits in the register that are also set in `clear`, and sets
    /// any bits set in `set`, in that order.
    ///
    /// The effect is atomic from the perspective of other threads or
    /// interrupts; if there is a race (e.g. an interrupt) the update sequence
    /// will restart.  This means this function can produce many volatile loads,
    /// but only one store with the final result.
    fn atomic_nand_and_or(&mut self, clear: Self::Type, set: Self::Type);
}

// Implementation shorthand for the atomic RMW sequence on ARMv7M
macro_rules! atomic_rmw {
    ($addr:expr, $ty:ident, $code:expr, $($arg:expr),+) => {
        loop {
            unsafe {
                let tmp: u32;
                asm!(concat!("ldrex", ex_suffix!($ty), " $0, [$1]\n",
                             $code, "\n",
                             "strex", ex_suffix!($ty), " $0, $0, [$1]")
                     : "=&r"(tmp)
                     : "r"(&mut $addr as *mut $ty),
                       $("r"($arg)),+
                     : "memory"
                     : "volatile");
                if tmp == 0 { break }
            }
        }
    };
}

macro_rules! ex_suffix {
    (u32) => { "" };
    (i32) => { "" };
    (u16) => { "h" };
    (i16) => { "h" };
    (u8) => { "b" };
    (i8) => { "b" };
}

macro_rules! ex_impl {
    ($ty:ident) => {
        impl AtomicReg for Reg<$ty> {
            type Type = $ty;

            fn atomic_nand(&mut self, clear: $ty) {
                atomic_rmw!(self.value, $ty,
                            "bics $0, $2",
                            clear)
            }

            fn atomic_or(&mut self, set: $ty) {
                atomic_rmw!(self.value, $ty,
                            "orrs $0, $2",
                            set)
            }

            fn atomic_nand_and_or(&mut self, clear: $ty, set: $ty) {
                atomic_rmw!(self.value, $ty,
                            "bics $0, $2\n\
                            orrs $0, $3",
                            clear, set)
            }
        }

    };
}

ex_impl!(u32);
ex_impl!(u16);
ex_impl!(u8);