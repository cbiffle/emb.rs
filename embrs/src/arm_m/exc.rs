/// ARMvx-M interrupt and exception handlers are merely functions conforming to
/// the C ABI.
pub type Handler = extern "C" fn();

/// The reset vector is special: it must not return.  We can model this nicely
/// in Rust's type system as a diverging function.  We additionally mark the
/// reset handler as `unsafe` because it must do scary stuff, including zeroing
/// BSS.  Allowing a safe program to call it directly would be bad.
pub type ResetHandler = unsafe extern "C" fn() -> !;

/// Represents an ARMvx-M exception table.  This is the common table of vectors
/// used for handling interrupts and initializing the processor on -M
/// processors.
///
/// Vectors are described using the `Handler` type, which is merely a function
/// reference.  Most vectors in the table are modeled as `Option<Handler>`,
/// relying on the null pointer optimization to store zero for `None`.
/// Applications that don't use a particular vector can thus omit it.
///
/// Note that processors will typically have a *two-part* vector table: first
/// come the exception vectors (described here), immediately followed by
/// vendor-specific interrupt vectors handled through the NVIC.  We model such
/// tables in two parts, and concatenate them at link time.  Thus, you will not
/// find vendor-specific vectors here.
#[repr(C, packed)]
pub struct ExceptionTable {
    /// ARMvx-M processors load their initial stack pointer from the first word
    /// of the vector table.  This will be the contents of `sp` on entry to
    /// `reset` below.
    ///
    /// Remember that ARM uses a "full descending" stack, so `sp` points to the
    /// most recently *used* cell of the stack.  Thus, the initial `sp` when the
    /// stack is empty often points just past the end of RAM.  We model it here
    /// as a `const` pointer to discourage such an invalid address from being
    /// dereferenced.
    pub initial_stack: *const u32,

    /// Reset vector.  At reset, the processor loads its stack pointer from
    /// `initial_stack` (above) and then enters this function using the ARM
    /// AAPCS C ABI.
    pub reset: ResetHandler,

    // Architecturally defined exception vectors (i.e. those that are
    // vendor-independent) begin here.  The architectural vector table includes
    // five reserved entries.  We model them here as `pub` because certain
    // naughty vendors (looking at you, NXP) have a loose interpretation of the
    // term "reserved" and stuff things into them anyway.

    /// Non-Maskable Interrupt handler.
    pub nmi:          Option<Handler>,
    /// Hard Fault handler.
    pub hard_fault:   Option<Handler>,
    /// Memory Management Fault handler.
    pub mm_fault:     Option<Handler>,
    /// Bus Fault handler.
    pub bus_fault:    Option<Handler>,
    /// Usage Fault handler.
    pub usage_fault:  Option<Handler>,
    pub _reserved0:   Option<Handler>,
    pub _reserved1:   Option<Handler>,
    pub _reserved2:   Option<Handler>,
    pub _reserved3:   Option<Handler>,
    /// Supervisor Call (`SVC`) handler.
    pub sv_call:      Option<Handler>,
    /// Debug Monitor handler.
    pub debug_mon:    Option<Handler>,
    pub _reserved4:   Option<Handler>,
    /// PendSV handler.
    pub pend_sv:      Option<Handler>,
    /// SysTick handler.
    pub sys_tick:     Option<Handler>,
}

/// An exception table that is empty except for the stack pointer and reset
/// vector.  This can work in a system that is willing to lock up at any fault.
/// In practice, this is used with functional struct update syntax like so:
///
///     pub static VECTORS : ExceptionTable = ExceptionTable {
///       hard_fault: Some(my_hard_fault_handler),
///       .. empty_exception_table(stack_pointer, reset_handler)
///     };
pub const fn empty_exception_table(initial_stack: *const u32,
                                   reset: ResetHandler) -> ExceptionTable {
    ExceptionTable {
        initial_stack: initial_stack,
        reset: reset,

        nmi: None,
        hard_fault: None,
        mm_fault: None,
        bus_fault: None,
        usage_fault: None,
        _reserved0: None,
        _reserved1: None,
        _reserved2: None,
        _reserved3: None,
        sv_call: None,
        debug_mon: None,
        _reserved4: None,
        pend_sv: None,
        sys_tick: None,
    }
}

/// Most programs will have at least one `ExceptionTable` `static`: the one that
/// gets deposited into ROM and read at processor startup.
///
/// To support a `static` `ExceptionTable`, the type must be `Sync`.  It is
/// *almost* `Sync` out of the box.  The exception: the pointer used for the
/// `initial_stack` item.
///
/// To hack around this, we stamp `ExceptionTable` as `Sync`.  This is probably
/// not the right solution.
unsafe impl Sync for ExceptionTable {}
