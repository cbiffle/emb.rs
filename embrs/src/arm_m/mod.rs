pub mod exc;
pub mod nvic;
pub mod reg;
pub mod sys_tick;

#[cfg(target_os = "none")]
pub mod startup;

/// Sets the processor's `PRIMASK` register to `val`.
#[inline]
pub fn set_primask(val: bool) {
    unsafe {
        asm!("msr PRIMASK, $0"
             :: "r"(val)
             :: "volatile")
    }
}

/// Generates an instruction synchronization barrier (`ISB`) instruction.  For
/// other types of barriers, see Rust's fence operations.
#[inline]
pub fn instruction_synchronization_barrier() {
    unsafe {
        asm!("isb" :::: "volatile")
    }
}
