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

/// Generates an instruction synchronization barrier (`ISB`) instruction.
#[inline]
pub fn instruction_synchronization_barrier() {
    unsafe {
        asm!("isb" :::: "volatile")
    }
}

/// Generates a data synchronization barrier (`DSB`) instruction.
#[inline]
pub fn data_synchronization_barrier() {
    unsafe {
        asm!("dsb" :::: "volatile")
    }
}

/// Generates a data memory barrier (`DMB`) instruction.
#[inline]
pub fn data_memory_barrier() {
    unsafe {
        asm!("dmb" :::: "volatile")
    }
}

#[inline]
pub fn wait_for_interrupt() {
    unsafe {
        asm!("wfi" :::: "volatile")
    }
}
