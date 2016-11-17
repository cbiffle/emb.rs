//! Universal Synchronous/Asychronous Receiver/Transmitter (USART) support.

use arm_m::reg::Reg;

#[repr(C, packed)]
pub struct Registers {
    pub sr:   Reg<u32>,
    pub dr:   Reg<u32>,
    pub brr:  Reg<u32>,
    pub cr1:  Reg<u32>,
    pub cr2:  Reg<u32>,
    pub cr3:  Reg<u32>,
    pub gtpr: Reg<u32>,
}

bit_wrappers! {
    pub struct Sr(pub u32);
    pub struct Dr(pub u32);
    pub struct Brr(pub u32);
    pub struct Cr1(pub u32);
    pub struct Cr2(pub u32);
    pub struct Cr3(pub u32);
    pub struct Gtpr(pub u32);
}

impl Sr {
    bitfield_accessors! {
        pub total [9] get_cts / with_cts: bool,
        pub total [8] get_lbd / with_lbd: bool,
        pub total [7] get_txe / with_txe: bool,
        pub total [6] get_tc / with_tc: bool,
        pub total [5] get_rxne / with_rxne: bool,
        pub total [4] get_idle / with_idle: bool,
        pub total [3] get_ore / with_ore: bool,
        pub total [2] get_nf / with_nf: bool,
        pub total [1] get_fe / with_fe: bool,
        pub total [0] get_pe / with_pe: bool,
    }
}

impl Dr {
    bitfield_accessors! {
        pub total [7:0] get_data / with_data: u8,
    }
}

impl Brr {
    bitfield_accessors! {
        pub total [15:4] get_mantissa / with_mantissa: u32,
        pub total [3:0] get_fraction / with_fraction: u32,
    }
}

impl Cr1 {
    bitfield_accessors! {
        pub total [15] get_over8 / with_over8: bool,
        pub total [13] get_ue / with_ue: bool,
        pub total [12] get_m / with_m: WordLength,
        pub total [11] get_wake / with_wake: WakeupMethod,
        pub total [10] get_pce / with_pce: bool,
        pub total [ 9] get_ps / with_ps: Parity,
        pub total [ 8] get_peie / with_peie: bool,
        pub total [ 7] get_txeie / with_txeie: bool,
        pub total [ 6] get_tcie / with_tcie: bool,
        pub total [ 5] get_rxneie / with_rxneie: bool,
        pub total [ 4] get_idleie / with_idleie: bool,
        pub total [ 3] get_te / with_te: bool,
        pub total [ 2] get_re / with_re: bool,
        pub total [ 1] get_rwu / with_rwu: bool,
        pub total [ 0] get_sbk / with_sbk: bool,

    }
}

impl Cr2 {
    bitfield_accessors! {
        pub total [14] get_linen / with_linen: bool,
        pub total [13:12] get_stop / with_stop: StopBits,
        pub total [11] get_clken / with_clken: bool,
        pub total [10] get_cpol / with_cpol: ClockPolarity,
        pub total [ 9] get_cpha / with_cpha: ClockPhase,
        pub total [ 8] get_lbcl / with_lbcl: bool,
        pub total [ 6] get_lbdie / with_lbdie: bool,
        pub total [ 5] get_lbdl / with_lbdl: BreakLength,
        pub total [3:0] get_add / with_add: u32,
    }
}

impl Cr3 {
    bitfield_accessors! {
        pub total [11] get_onebit / with_onebit: SampleMethod,
        pub total [10] get_ctsie / with_ctsie: bool,
        pub total [ 9] get_ctse / with_ctse: bool,
        pub total [ 8] get_rtse / with_rtse: bool,
        pub total [ 7] get_dmat / with_dmat: bool,
        pub total [ 6] get_dmar / with_dmar: bool,
        pub total [ 5] get_scen / with_scen: bool,
        pub total [ 4] get_nack / with_nack: bool,
        pub total [ 3] get_hdsel / with_hdsel: bool,
        pub total [ 2] get_irlp / with_irlp: bool,
        pub total [ 1] get_iren / with_iren: bool,
        pub total [ 0] get_eie / with_eie: bool,
    }
}

impl Gtpr {
    bitfield_accessors! {
        pub total [15:8] get_gt / with_gt: u8,
        pub total [ 7:0] get_psc / with_psc: u8,
    }
}

bit_enums! {
    pub bit_enum WordLength {
        EightBits = 0,
        NineBits = 1,
    }

    pub bit_enum WakeupMethod {
        IdleLine = 0,
        AddressMark = 1,
    }

    pub bit_enum Parity {
        Even = 0,
        Odd = 1,
    }

    pub bit_enum StopBits {
        One = 0b00,
        Half = 0b01,
        Two = 0b10,
        OneAndAHalf = 0b11,
    }

    pub bit_enum ClockPolarity {
        IdleLow = 0,
        IdleHigh = 1,
    }

    pub bit_enum ClockPhase {
        CaptureOnFirstEdge = 0,
        CaptureOnSecondEdge = 1,
    }

    pub bit_enum BreakLength {
        TenBits = 0,
        ElevenBits = 1,
    }

    pub bit_enum SampleMethod {
        ThreeBit = 0,
        OneBit = 1,
    }
}
