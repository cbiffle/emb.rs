//! Support for the ARM Nested Vector Interrupt Controller, or NVIC.
//!
//! This is the interrupt controller used across all (current) M-profile
//! processors.

use core::sync::atomic;

use arm_m;
use arm_m::reg::Reg;

#[repr(C, packed)]
struct Registers {
    /// The Interrupt Set Enabled Registers have one bit for each potential
    /// interrupt source.  Writing ones causes the corresponding interrupt(s) to
    /// become enabled; others remain unchanged.
    iser: [Reg<u32>; 16], _reserved_after_iser: [Reg<u32>; 16],

    /// The Interrupt Clear Enabled Registers have one bit for each potential
    /// interrupt source.  Writing ones causes the corresponding interrupt(s) to
    /// become disabled; others remain unchanged.
    icer: [Reg<u32>; 16], _reserved_after_icer: [Reg<u32>; 16],

    /// The Interrupt Set Pending Registers have one bit for each potential
    /// interrupt source.  Writing ones causes the corresponding interrupt(s) to
    /// become pending; others remain unchanged.
    ispr: [Reg<u32>; 16], _reserved_after_ispr: [Reg<u32>; 16],

    /// The Interrupt Clear Pending Registers have one bit for each potential
    /// interrupt source.  Writing ones causes the corresponding interrupt(s) to
    /// become non-pending; others remain unchanged.
    icpr: [Reg<u32>; 16], _reserved_after_icpr: [Reg<u32>; 16],

    /// The Interrupt Active Bit Registers have one bit for each potential
    /// interrupt source.  The bit is 1 if the interrupt is active, 0 otherwise.
    iabr: [Reg<u32>; 16], _reserved_after_iabr: [Reg<u32>; 48],

    /// The Interrupt Priority Registers contain an 8-bit field for each
    /// potential interrupt source.  The field contains the interrupt's
    /// priority.  Note that SoC vendors may leave some LSBs of the field
    /// unimplemented.
    ///
    /// While described in the ARM as 32-bit registers, these registers are
    /// explicitly permitted for byte access, which is how we model them here.
    ipr: [Reg<u8>; 496],
}

extern {
    #[link_name="arm_m_nvic_NVIC"]
    static mut _NVIC: Registers;
}

/// Driver for the NVIC.
///
/// Because operations on the NVIC affect interrupts, which are asynchronous
/// events that can affect program order and make things difficult to reason
/// about, the methods on `Nvic` are very carefully specified.
pub struct Nvic {
    reg: *mut Registers,
}

impl Nvic {
    /// Ensures that an interrupt is enabled by the time this function returns.
    ///
    /// If the interrupt is pending, and the current execution priority allows
    /// it to preempt, the handler will have run *before this function returns*.
    ///
    /// You probably don't want to call this function.  The SoC layer's
    /// `NvicExt` trait provides an `enable_irq` method that is both more
    /// ergonomic (taking an enum instead of a `u32`) and *more performant*
    /// (because the enum lets us eliminate some range checks).
    #[inline]  // into the SoC layer
    pub fn enable_irq_raw(&self, irq: u32) {
        let (bank, index) = ((irq / 32) as usize, irq % 32);

        unsafe {
            self.reg().iser[bank].set(1 << index);
        }
        Self::write_barriers()
    }

    /// Ensures that an interrupt is disabled by the time this function returns.
    ///
    /// In the presence of a concurrent or pending interrupt from `irq`,
    /// assuming the current execution priority would allow it to preempt, its
    /// handler will either execute before this function returns, or will be
    /// deferred.
    ///
    /// Thus, code appearing after a call to `disable_irq_raw` in program order
    /// can assume it will not be preempted by this interrupt (assuming that
    /// some other concurrent activity, such as a separate interrupt, doesn't
    /// re-enable it).
    ///
    /// You probably don't want to call this function.  The SoC layer's
    /// `NvicExt` trait  provides a `disable_irq` method that is both more
    /// ergonomic (taking an enum instead of a `u32`) and *more performant*
    /// (because the enum lets us eliminate some range checks).
    #[inline]  // into the SoC layer
    pub fn disable_irq_raw(&self, irq: u32) {
        let (bank, index) = ((irq / 32) as usize, irq % 32);

        unsafe {
            self.reg().icer[bank].set(1 << index);
        }
        Self::write_barriers()
    }

    /// Sets the priority of an interrupt, synchronously.
    ///
    /// This may cause immediate preemption in the following cases:
    ///
    /// 1. If `priority` is eligible for preempt at the current execution
    ///    priority, and `irq` is pending.
    ///
    /// 2. If called from the handler from `irq` such that `priority` *lowers*
    ///    the current execution priority, and a different interrupt with
    ///    higher priority is pending.
    ///
    /// Note that changing the priority of an interrupt *while that interrupt's
    /// handler is executing or preempted* does not necessarily affect the
    /// current execution priority: the hardware ensures that doing so never
    /// produces a priority inversion between the current execution priority and
    /// any previously preempted handlers.
    ///
    /// You probably don't want to call this function.  The SoC layer's
    /// `NvicExt` trait  provides a `set_priority` method that is both more
    /// ergonomic (taking an enum instead of a `u32`) and *more performant*
    /// (because the enum lets us eliminate some range checks).
    #[inline]  // into the SoC layer
    pub fn set_priority_raw(&self, irq: u32, priority: u8) {
        unsafe {
            self.reg().ipr[irq as usize].set(priority);
        }
        Self::write_barriers()
    }

    /// Reads the priority of an interrupt.
    ///
    /// This operation is atomic with respect to `set_priority_raw`, but makes
    /// no particular guarantees about interaction with preempting interrupt
    /// handlers.
    #[inline]  // into the SoC layer
    pub fn get_priority_raw(&self, irq: u32) -> u8 {
        atomic::fence(atomic::Ordering::Acquire);

        unsafe {
            self.reg().ipr[irq as usize].get()
        }
    }

    unsafe fn reg(&self) -> &mut Registers {
        &mut *self.reg
    }

    #[inline]
    fn write_barriers() {
        // Data fence to ensure the write is not buffered (emits DMB).
        atomic::fence(atomic::Ordering::Release);
        // Instruction barrier to flush any instructions fetched before the
        // write completed.
        arm_m::instruction_synchronization_barrier()
    }
}

unsafe impl Sync for Nvic {}

/// Shared static instance of the `Nvic` driver.
pub static NVIC: Nvic = Nvic {
    reg: unsafe { &_NVIC as *const Registers as *mut Registers },
};
