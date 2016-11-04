//! Interrupt support for the STM32F4.
//!
//! This module adapts the primitive NVIC support in `arm_m::nvic` to the
//! STM32F4 line of SoCs.  It provides:
//! - `struct InterruptTable` for modeling the vendor-specific vector table.
//! - `trait NvicExt` to extend the NVIC with operations using STM32F4-specific
//!   vector numbers and widths.

use arm_m::nvic;

/// Re-export the type used for interrupt vectors on ARMv7-M.
pub use arm_m::exc::Handler;

/// The STM32F4's vendor-specific (NVIC) vector table.  This is separate from
/// the ARMv7-M Exception Table, and must be placed immediately after it in ROM
/// or RAM.
#[repr(C, packed)]
pub struct InterruptTable {
    pub wwdg: Option<Handler>,
    pub pvd: Option<Handler>,
    pub tamp_stamp: Option<Handler>,
    pub rtc_wkup: Option<Handler>,
    pub flash: Option<Handler>,
    pub rcc: Option<Handler>,
    pub exti0: Option<Handler>,
    pub exti1: Option<Handler>,
    pub exti2: Option<Handler>,
    pub exti3: Option<Handler>,
    pub exti4: Option<Handler>,
    pub dma1_stream0: Option<Handler>,
    pub dma1_stream1: Option<Handler>,
    pub dma1_stream2: Option<Handler>,
    pub dma1_stream3: Option<Handler>,
    pub dma1_stream4: Option<Handler>,
    pub dma1_stream5: Option<Handler>,
    pub dma1_stream6: Option<Handler>,
    pub adc: Option<Handler>,

    pub can1_tx: Option<Handler>,
    pub can1_rx0: Option<Handler>,
    pub can1_rx1: Option<Handler>,
    pub can1_sce: Option<Handler>,
    pub exti9_5: Option<Handler>,
    pub tim1_brk_tim9: Option<Handler>,
    pub tim1_up_tim10: Option<Handler>,
    pub tim1_trg_com_tim11: Option<Handler>,
    pub tim1_cc: Option<Handler>,
    pub tim2: Option<Handler>,
    pub tim3: Option<Handler>,
    pub tim4: Option<Handler>,
    pub i2c1_ev: Option<Handler>,
    pub i2c1_er: Option<Handler>,
    pub i2c2_ev: Option<Handler>,
    pub i2c2_er: Option<Handler>,
    pub spi1: Option<Handler>,
    pub spi2: Option<Handler>,
    pub usart1: Option<Handler>,
    pub usart2: Option<Handler>,
    pub usart3: Option<Handler>,
    pub exti15_10: Option<Handler>,
    pub rtc_alarm: Option<Handler>,
    pub otg_fs_wkup: Option<Handler>,
    pub tim8_brk_tim12: Option<Handler>,
    pub tim8_up_tim13: Option<Handler>,
    pub tim8_trg_com_tim14: Option<Handler>,
    pub tim8_cc: Option<Handler>,
    pub dma1_stream7: Option<Handler>,
    pub fsmc: Option<Handler>,
    pub sdio: Option<Handler>,
    pub tim5: Option<Handler>,
    pub spi3: Option<Handler>,
    pub uart4: Option<Handler>,
    pub uart5: Option<Handler>,
    pub tim6_dac: Option<Handler>,
    pub tim7: Option<Handler>,
    pub dma2_stream0: Option<Handler>,
    pub dma2_stream1: Option<Handler>,
    pub dma2_stream2: Option<Handler>,
    pub dma2_stream3: Option<Handler>,
    pub dma2_stream4: Option<Handler>,
    pub eth: Option<Handler>,
    pub eth_wkup: Option<Handler>,
    pub can2_tx: Option<Handler>,
    pub can2_rx0: Option<Handler>,
    pub can2_rx1: Option<Handler>,
    pub can2_sce: Option<Handler>,
    pub otg_fs: Option<Handler>,
    pub dma2_stream5: Option<Handler>,
    pub dma2_stream6: Option<Handler>,
    pub dma2_stream7: Option<Handler>,
    pub usart6: Option<Handler>,
    pub i2c3_ev: Option<Handler>,
    pub i2c3_er: Option<Handler>,
    pub otg_hs_ep1_out: Option<Handler>,
    pub otg_hs_ep1_in: Option<Handler>,
    pub otg_hs_wkup: Option<Handler>,
    pub otg_hs: Option<Handler>,
    pub dcmi: Option<Handler>,
    pub cryp: Option<Handler>,
    pub hash_rng: Option<Handler>,
    pub fpu: Option<Handler>,

    pub uart7: Option<Handler>,
    pub uart8: Option<Handler>,
    pub spi4: Option<Handler>,
    pub spi5: Option<Handler>,
    pub spi6: Option<Handler>,
    pub sai1: Option<Handler>,
    pub ltdc: Option<Handler>,
    pub ltdc_er: Option<Handler>,
    pub dma2d: Option<Handler>,
}

/// An `InterruptTable` with all vectors omitted.  This can be used with struct
/// update syntax to easily declare a vector table containing only a few
/// entries:
///
///     static VECTOR_TABLE : InterruptTable = InterruptTable {
///         adc: Some(my_adc_handler),
///         .. EMPTY_TABLE
///     };
pub const EMPTY_TABLE : InterruptTable = InterruptTable {
    wwdg: None,
    pvd: None,
    tamp_stamp: None,
    rtc_wkup: None,
    flash: None,
    rcc: None,
    exti0: None,
    exti1: None,
    exti2: None,
    exti3: None,
    exti4: None,
    dma1_stream0: None,
    dma1_stream1: None,
    dma1_stream2: None,
    dma1_stream3: None,
    dma1_stream4: None,
    dma1_stream5: None,
    dma1_stream6: None,
    adc: None,

    can1_tx: None,
    can1_rx0: None,
    can1_rx1: None,
    can1_sce: None,
    exti9_5: None,
    tim1_brk_tim9: None,
    tim1_up_tim10: None,
    tim1_trg_com_tim11: None,
    tim1_cc: None,
    tim2: None,
    tim3: None,
    tim4: None,
    i2c1_ev: None,
    i2c1_er: None,
    i2c2_ev: None,
    i2c2_er: None,
    spi1: None,
    spi2: None,
    usart1: None,
    usart2: None,
    usart3: None,
    exti15_10: None,
    rtc_alarm: None,
    otg_fs_wkup: None,
    tim8_brk_tim12: None,
    tim8_up_tim13: None,
    tim8_trg_com_tim14: None,
    tim8_cc: None,
    dma1_stream7: None,
    fsmc: None,
    sdio: None,
    tim5: None,
    spi3: None,
    uart4: None,
    uart5: None,
    tim6_dac: None,
    tim7: None,
    dma2_stream0: None,
    dma2_stream1: None,
    dma2_stream2: None,
    dma2_stream3: None,
    dma2_stream4: None,
    eth: None,
    eth_wkup: None,
    can2_tx: None,
    can2_rx0: None,
    can2_rx1: None,
    can2_sce: None,
    otg_fs: None,
    dma2_stream5: None,
    dma2_stream6: None,
    dma2_stream7: None,
    usart6: None,
    i2c3_ev: None,
    i2c3_er: None,
    otg_hs_ep1_out: None,
    otg_hs_ep1_in: None,
    otg_hs_wkup: None,
    otg_hs: None,
    dcmi: None,
    cryp: None,
    hash_rng: None,
    fpu: None,

    uart7: None,
    uart8: None,
    spi4: None,
    spi5: None,
    spi6: None,
    sai1: None,
    ltdc: None,
    ltdc_er: None,
    dma2d: None,
};

/// Enumeration of the STM32F4 interrupts.  This can be used to name an
/// interrupt vector, like an integer, but without the risk of receiving
/// out-of-range values.
#[derive(Clone, Copy)]
pub enum Interrupt {
    Wwdg = 0,
    Pvd,
    TampStamp,
    RtcWkup,
    Flash,
    Rcc,
    Exti0,
    Exti1,
    Exti2,
    Exti3,
    Exti4,
    Dma1Stream0,
    Dma1Stream1,
    Dma1Stream2,
    Dma1Stream3,
    Dma1Stream4,
    Dma1Stream5,
    Dma1Stream6,
    Adc,

    Can1Tx,
    Can1Rx0,
    Can1Rx1,
    Can1Sce,
    Exti95,
    Tim1BrkTim9,
    Tim1UpTim10,
    Tim1TrgComTim11,
    Tim1Cc,
    Tim2,
    Tim3,
    Tim4,
    I2c1Ev,
    I2c1Er,
    I2c2Ev,
    I2c2Er,
    Spi1,
    Spi2,
    Usart1,
    Usart2,
    Usart3,
    Exti1510,
    RtcAlarm,
    OtgFsWkup,
    Tim8BrkTim12,
    Tim8UpTim13,
    Tim8TrgComTim14,
    Tim8Cc,
    Dma1Stream7,
    Fsmc,
    Sdio,
    Tim5,
    Spi3,
    Uart4,
    Uart5,
    Tim6Dac,
    Tim7,
    Dma2Stream0,
    Dma2Stream1,
    Dma2Stream2,
    Dma2Stream3,
    Dma2Stream4,
    Eth,
    EthWkup,
    Can2Tx,
    Can2Rx0,
    Can2Rx1,
    Can2Sce,
    OtgFs,
    Dma2Stream5,
    Dma2Stream6,
    Dma2Stream7,
    Usart6,
    I2c3Ev,
    I2c3Er,
    OtgHsEp1Out,
    OtgHsEp1In,
    OtgHsWkup,
    OtgHs,
    Dcmi,
    Cryp,
    HashRng,
    Fpu,

    Uart7,
    Uart8,
    Spi4,
    Spi5,
    Spi6,
    Sai1,
    Ltdc,
    LtdcEr,
    Dma2d,
}

const PRIO_SHIFT : u32 = 4;

/// Enumeration of the STM32F4 interrupt priority values.  The STM32F4 only
/// implements four bits of priority, or 16 levels.  This enumeration acts like
/// a four-bit integer to avoid needing to range check priority values at
/// runtime.
#[derive(Clone, Copy)]
pub enum Priority {
    P0 = 0, P1, P2, P3, P4, P5, P6, P7,
    P8, P9, P10, P11, P12, P13, P14, P15,
}

/// Extension trait for `arm_m::Nvic` adding operations that deal in
/// STM32F4-specific enumerations.
pub trait NvicExt {
    /// Ensures that an interrupt is enabled by the time this function returns.
    ///
    /// If the interrupt is pending, and the current execution priority allows
    /// it to preempt, the handler will have run *before this function returns*.
    ///
    /// This is a wrapper for `enable_irq_raw` that lets us omit the runtime
    /// range checks.
    fn enable_irq(&self, irq: Interrupt);

    /// Ensures that an interrupt is disabled by the time this function returns.
    ///
    /// In the presence of a concurrent or pending interrupt from `irq`,
    /// assuming the current execution priority would allow it to preempt, its
    /// handler will either execute before this function returns, or will be
    /// deferred.
    ///
    /// Thus, code appearing after a call to `disable_irq` in program order can
    /// assume it will not be preempted by this interrupt (assuming that
    /// some other concurrent activity, such as a separate interrupt, doesn't
    /// re-enable it).
    ///
    /// This is a wrapper for `disable_irq_raw` that lets us omit the runtime
    /// range checks.
    fn disable_irq(&self, irq: Interrupt);

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
    /// This is a wrapper for `set_priority_raw` that lets us omit the runtime
    /// range checks.
    fn set_priority(&self, irq: Interrupt, priority: Priority);

    /// Reads the priority of an interrupt.
    ///
    /// This operation is atomic with respect to `set_priority`, but makes
    /// no particular guarantees about interaction with preempting interrupt
    /// handlers.
    ///
    /// This is a wrapper for `get_priority_raw` that lets us omit the runtime
    /// range checks, and return a member of `Priority`.
    fn get_priority(&self, irq: Interrupt) -> Priority;
}

impl NvicExt for nvic::Nvic {
    fn enable_irq(&self, irq: Interrupt) {
        self.enable_irq_raw(irq as u32)
    }

    fn disable_irq(&self, irq: Interrupt) {
        self.disable_irq_raw(irq as u32)
    }

    fn set_priority(&self, irq: Interrupt, priority: Priority) {
        self.set_priority_raw(irq as u32, (priority as u8) << PRIO_SHIFT)
    }

    fn get_priority(&self, irq: Interrupt) -> Priority {
        // We're relying on the compiler to recognize this silliness.
        match self.get_priority_raw(irq as u32) >> PRIO_SHIFT {
            0 => Priority::P0,
            1 => Priority::P1,
            2 => Priority::P2,
            3 => Priority::P3,
            4 => Priority::P4,
            5 => Priority::P5,
            6 => Priority::P6,
            7 => Priority::P7,
            8 => Priority::P8,
            9 => Priority::P9,
            10 => Priority::P10,
            11 => Priority::P11,
            12 => Priority::P12,
            13 => Priority::P13,
            14 => Priority::P14,
            15 => Priority::P15,
            _ => unreachable!(),
        }
    }

}
