//! Direct Memory Access (DMA) register layer.

#![allow(trivial_numeric_casts)]  // for bitflags :-(

use core::mem;
use arm_m::reg::Reg;
use bits;

/// Register layout of a DMA controller.
#[repr(C, packed)]
pub struct Dma {
    /// Interrupt status registers, described in the Reference Manual as LISR
    /// and HISR, but represented here as an array (in that order).
    pub isr:  [Reg<Ir>; 2],

    /// Interrupt flag clear registers, described in the Reference Manual as
    /// LIFCR and HIFCR, but represented here as an array (in that order).
    pub ifcr: [Reg<Ir>; 2],

    /// Control registers for the eight hardware DMA streams.
    pub stream: [Stream; 8],
}

/// Register layout of a single DMA stream.
#[repr(C, packed)]
pub struct Stream {
    /// Control register.
    pub cr:   Reg<Cr>,
    /// Number of data to transfer register.
    pub ndtr: Reg<Ndtr>,
    /// Peripheral address register.
    ///
    /// Note that this register holds the address for the "peripheral" side of
    /// the transfer -- the one where activity is governed by DRQs -- but it is
    /// not inherently restricted to the addresses of peripherals.
    pub par:  Reg<*const ()>,
    /// Memory address registers M0AR and M1AR.
    ///
    /// Note that this register holds the address for the "memory" side of
    /// the transfer -- the one where activity is not governed by DRQs -- but it
    /// is not inherently restricted to addresses in memory.
    ///
    /// `mar[1]` (or M1AR in the Reference Manual) comes into play in
    /// double-buffered mode.
    pub mar:  [Reg<*const ()>; 2],
    /// FIFO control register.
    pub fcr:  Reg<Fcr>,
}

/// Names of DMA streams.
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum StreamIndex {
    S0, S1, S2, S3, S4, S5, S6, S7
}

impl StreamIndex {
    /// Converts a stream index into the corresponding index into the interrupt
    /// register arrays `isr` and `ifcr`.
    pub fn get_ir_index(self) -> usize {
        (self as usize) / 4
    }

    /// Converts a stream index into the corresponding relative stream index
    /// within an interrupt register (`isr[x]` or `ifcr[x]`).
    pub fn get_rs_index(self) -> RelativeStreamIndex {
        unsafe {
            mem::transmute((self as u8) % 4)
        }
    }
}

bit_wrappers! {
    /// Interrupt Register type, used by both the Interrupt Status Registers and
    /// the Interrupt Flag Clear Registers.
    ///
    /// Interrupt registers contain an irregularly packed array of four five-bit
    /// fields, describing four streams.  `isr[0]` and `ifcr[0]` describe
    /// streams 0-3, while `isr[1]` and `ifcr[1]` describe streams 4-7.  Within
    /// a single register we refer to the four streams (whichever they may be)
    /// as *relative streams* 0-3.
    ///
    /// Accessing streams this way is a bit fiddly, so we provide higher-level
    /// accessors on `Dma`: `get_interrupt_flags` and `clear_interrupt_flags`.
    pub struct Ir(pub u32);
    pub struct Cr(pub u32);
    pub struct Ndtr(pub u32);
    pub struct Par(pub u32);
    pub struct Mar(pub u32);
    pub struct Fcr(pub u32);
}

bitflags! {
    /// The set of interrupt bit flags that can appear for a given DMA stream.
    /// These are used to indicate stream status (in the Interrupt Status
    /// Registers) and to change it (in the Interrupt Flag Clear Registers).
    pub flags InterruptFlags: u32 {
        const FIFO_ERROR = 1 << 0,
        const DIRECT_MODE_ERROR = 1 << 2,
        const TRANSFER_ERROR = 1 << 3,
        const HALF_TRANSFER = 1 << 4,
        const TRANSFER_COMPLETE = 1 << 5,
    }
}

impl bits::FromBits for InterruptFlags {
    fn from_bits(bits: u32) -> bits::BitsResult<Self> {
        InterruptFlags::from_bits(bits).ok_or(bits::BadBits(bits))
    }
}

impl bits::IntoBits for InterruptFlags {
    fn into_bits(self) -> u32 {
        self.bits()
    }
}

/// Names of relative streams within an interrupt register (`Ir`).
///
/// This is mostly used under the hood, but in case you need it: here it is.
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum RelativeStreamIndex {
    RS0, RS1, RS2, RS3,
}

impl Ir {
    bitfield_accessors! {
        /// Relative Stream 3, the fourth stream in this register.
        pub [27:22] get_rs3 / with_rs3: InterruptFlags,
        /// Relative Stream 2, the third stream in this register.
        pub [21:16] get_rs2 / with_rs2: InterruptFlags,
        /// Relative Stream 1, the second stream in this register.
        pub [11: 6] get_rs1 / with_rs1: InterruptFlags,
        /// Relative Stream 0, the first stream in this register.
        pub [ 5: 0] get_rs0 / with_rs0: InterruptFlags,
    }

    /// Gets the `InterruptFlags` for a relative stream by (runtime) index.
    pub fn get_rs(self, i: RelativeStreamIndex)
        -> bits::BitsResult<InterruptFlags> {
        match i {
            RelativeStreamIndex::RS0 => self.get_rs0(),
            RelativeStreamIndex::RS1 => self.get_rs1(),
            RelativeStreamIndex::RS2 => self.get_rs2(),
            RelativeStreamIndex::RS3 => self.get_rs3(),
        }
    }

    /// Sets the `InterruptFlags` for a relative stream by (runtime) index.
    pub fn with_rs(self, i: RelativeStreamIndex, v: InterruptFlags) -> Self {
        match i {
            RelativeStreamIndex::RS0 => self.with_rs0(v),
            RelativeStreamIndex::RS1 => self.with_rs1(v),
            RelativeStreamIndex::RS2 => self.with_rs2(v),
            RelativeStreamIndex::RS3 => self.with_rs3(v),
        }
    }
}

impl Cr {
    bitfield_accessors! {
        pub total [27:25] get_chsel / with_chsel: Channel,
        pub total [24:23] get_mburst / with_mburst: Burst,
        pub total [22:21] get_pburst / with_pburst: Burst,
        pub total [ 19  ] get_ct / with_ct: Target,
        pub total [ 18  ] get_dbm / with_dbm: bool,
        pub total [17:16] get_pl / with_pl: Priority,
        pub total [ 15  ] get_pincos / with_pincos: bool,
        pub       [14:13] get_msize / with_msize: TransferSize,
        pub       [12:11] get_psize / with_psize: TransferSize,
        pub total [ 10  ] get_minc / with_minc: bool,
        pub total [  9  ] get_pinc / with_pinc: bool,
        pub total [  8  ] get_circ / with_circ: bool,
        pub       [ 7: 6] get_dir / with_dir: Direction,
        pub total [  5  ] get_pfctrl / with_pfctrl: bool,
        pub total [  4  ] get_tcie / with_tcie: bool,
        pub total [  3  ] get_htie / with_htie: bool,
        pub total [  2  ] get_teie / with_teie: bool,
        pub total [  1  ] get_dmeie / with_dmeie: bool,
        pub total [  0  ] get_en / with_en: bool,
    }
}

bit_enums! {
    /// Names the DRQ channels available on each stream.
    pub bit_enum Channel {
        Ch0 = 0b000,
        Ch1 = 0b001,
        Ch2 = 0b010,
        Ch3 = 0b011,
        Ch4 = 0b100,
        Ch5 = 0b101,
        Ch6 = 0b110,
        Ch7 = 0b111,
    }

    /// The possible sizes of burst transfer
    pub bit_enum Burst {
        Single = 0b00,
        Incr4 = 0b01,
        Incr8 = 0b10,
        Incr16 = 0b11,
    }

    /// Memory-side target selection in double-buffer mode.
    pub bit_enum Target {
        Memory0 = 0,
        Memory1 = 1,
    }

    /// Stream priority for arbitration among streams within a single DMA
    /// controller.
    pub bit_enum Priority {
        Low = 0b00,
        Medium = 0b01,
        High = 0b10,
        VeryHigh = 0b11,
    }

    /// Size of each DMA transfer.
    pub bit_enum TransferSize {
        Byte = 0b00,
        HalfWord = 0b01,
        Word = 0b10,
    }

    /// Transfer direction.  This controls whether peripheral DRQs trigger reads
    /// (`PeripheralToMemory`), writes (`MemoryToPeripheral`), or neither
    /// (`MemoryToMemory`, transfers are performed as fast as possible).
    pub bit_enum Direction {
        PeripheralToMemory = 0b00,
        MemoryToPeripheral = 0b01,
        MemoryToMemory = 0b10,
    }
}

impl Ndtr {
    bitfield_accessors! {
        pub total [15: 0] get_ndt / with_ndt: u16,
    }
}

impl Fcr {
    bitfield_accessors! {
        pub total [7] get_feie / with_feie: bool,
        pub       [5:3] get_fs / with_fs: FifoLevel,
        pub total [2] get_dmdis / with_dmdis: bool,
        pub total [1:0] get_fth / with_fth: FifoThreshold,
    }
}

bit_enums! {
    pub bit_enum FifoLevel {
        Under25  = 0b000,
        Under50  = 0b001,
        Under75  = 0b010,
        Under100 = 0b011,
        Empty    = 0b100,
        Full     = 0b101,
    }

    pub bit_enum FifoThreshold {
        At25 = 0b00,
        At50 = 0b01,
        At75 = 0b10,
        At100 = 0b11,
    }
}

impl Dma {
    /// Clears a set of interrupt flags for a particular stream.
    ///
    /// This winds up writing one of the two `ifcr` registers, but is more
    /// convenient than doing it by hand.
    pub fn clear_interrupt_flags(&self, s: StreamIndex, flags: InterruptFlags) {
        self.ifcr[s.get_ir_index()]
            .update(|v| v.with_rs(s.get_rs_index(), flags))
    }

    /// Reads the current set of interrupt flags for a particular stream.
    ///
    /// This winds up reading one of the two `isr` registers, but is more
    /// convenient than doing it by hand.
    pub fn get_interrupt_flags(&self, s: StreamIndex)
        -> bits::BitsResult<InterruptFlags> {
        self.isr[s.get_ir_index()]
            .get()
            .get_rs(s.get_rs_index())
    }
}

/// Produces a shared reference to DMA1.
#[inline]
pub fn dma1() -> &'static Dma {
    unsafe {
        &*(0x40026000 as *const Dma)
    }
}

/// Produces a shared reference to DMA2.
#[inline]
pub fn dma2() -> &'static Dma {
    unsafe {
        &*(0x40026400 as *const Dma)
    }
}
