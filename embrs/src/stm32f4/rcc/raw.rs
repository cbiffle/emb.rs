//! Reset and Clock Control (RCC) raw register interface.

use arm_m::reg::Reg;

/// The RCC's hardware register layout.
#[repr(C, packed)]
pub struct Registers {
    pub cr:            Reg<u32>,
    pub pllcfgr:       Reg<u32>,
    pub cfgr:          Reg<u32>,
    pub cir:           Reg<u32>,
    /// AHB peripheral reset registers AHB1RSTR - AHB3RSTR.
    ///
    /// Note that they are numbered from zero in this array.
    pub ahb_rstr:      [Reg<u32>; 3],
    pub _reserved_1c:  Reg<u32>,
    /// APB peripheral reset registers APB1RSTR - APB2RSTR.
    ///
    /// Note that they are numbered from zero in this array.
    pub apb_rstr:      [Reg<u32>; 2],
    pub _reserved_28:  Reg<u32>,
    pub _reserved_2c:  Reg<u32>,
    /// AHB clock enable registers AHB1ENR - AHB3ENR.
    ///
    /// Note that they are numbered from zero in this array.
    pub ahb_enr:       [Reg<u32>; 3],
    pub _reserved_3c:  Reg<u32>,
    /// APB clock enable registers APB1ENR - APB2ENR.
    ///
    /// Note that they are numbered from zero in this array.
    pub apb_enr:       [Reg<u32>; 2],
    pub _reserved_48:  Reg<u32>,
    pub _reserved_4c:  Reg<u32>,
    /// AHB low power clock enable registers AHB1LPENR - AHB3LPENR.
    ///
    /// Note that they are numbered from zero in this array.
    pub ahb_lpenr:     [Reg<u32>; 3],
    pub _reserved_5c:  Reg<u32>,
    /// APB low power clock enable registers APB1LPENR - APB2LPENR.
    ///
    /// Note that they are numbered from zero in this array.
    pub apb_lpenr:     [Reg<u32>; 2],
    pub _reserved_68:  Reg<u32>,
    pub _reserved_6c:  Reg<u32>,
    pub bdcr:          Reg<u32>,
    pub csr:           Reg<u32>,
    pub _reserved_78:  Reg<u32>,
    pub _reserved_7c:  Reg<u32>,
    pub sscgr:         Reg<u32>,
    pub plli2scfgr:    Reg<u32>,
    #[cfg(feature = "soc_family:stm32f4[23]")]
    pub pllsaicfgr:    Reg<u32>,
    #[cfg(feature = "soc_family:stm32f4[23]")]
    pub dckcfgr:       Reg<u32>,
}

bit_wrappers! {
    /// Wrapper for the Clock Control Register bits.
    pub struct Cr(pub u32);
    /// Wrapper for the Clock Configuration Register bits.
    pub struct Cfgr(pub u32);
    /// Wrapper for the PLL Configuration Register bits.
    pub struct Pllcfgr(pub u32);
}

impl Cr {
    bitfield_accessors! {
        /// Ready flag for the PLLI2S.
        #[cfg(feature = "soc_family:stm32f4[23]")]
        pub total [27] get_plli2srdy / with_plli2srdy: bool,
        /// Turns the PLLI2S on/off.
        #[cfg(feature = "soc_family:stm32f4[23]")]
        pub total [26] get_plli2son / with_plli2son: bool,
        /// Ready flag for the main PLL.
        pub total [25] get_pllrdy / with_pllrdy: bool,
        /// Turns the main PLL on/off.
        pub total [24] get_pllon / with_pllon: bool,
        /// Turns the Clock Security System (CSS) on/off.
        pub total [19] get_csson / with_csson: bool,
        /// When `true`, bypasses the HSE oscillator, using the external clock
        /// signal directly where the HSE clock would otherwise be used.
        pub total [18] get_hsebyp / with_hsebyp: bool,
        /// Ready flag for the HSE oscillator.
        pub total [17] get_hserdy / with_hserdy: bool,
        /// Turns the HSE oscillator on/off.
        pub total [16] get_hseon / with_hseon: bool,
        /// Internal HSI calibration bits, set by hardware at startup.
        pub total [15:8] get_hsical / with_hsical: u8,
        /// HSI trim adjusts the frequency of the HSI oscillator.
        pub total [7:3] get_hsitrim / with_hsitrim: u32,
        /// Ready flag for the HSI oscillator.
        pub total [1] get_hsirdy / with_hsirdy: bool,
        /// Turns the HSI oscillator on/off.
        pub total [0] get_hsion / with_hsion: bool,
    }
}

/// Wraps up a pattern we use repeatedly below, where we turn an enable flag and
/// a prescaler selection into an optional prescaler.
macro_rules! en_option_accessors {
    () => {};
    (
        $(#[$m:meta])*
        enable $get_en:ident / $with_en:ident
        value $get_div:ident / $with_div:ident : $ty:ty
        as $get_opt:ident / $with_opt:ident;

        $($rest:tt)*
    ) => {
        $(#[$m])*
        pub fn $get_opt(self) -> Option<$ty> {
            if self.$get_en() {
                Some(self.$get_div())
            } else {
                None
            }
        }

        $(#[$m])*
        pub fn $with_opt(self, v: Option<$ty>) -> Self {
            if let Some(wrapped) = v {
                self.$with_div(wrapped).$with_en(true)
            } else {
                self.$with_en(false)
            }
        }

        en_option_accessors!{$($rest)*}
    };
}

impl Cfgr {
    bitfield_accessors! {
        /// Controls the clock output on the MCO2 pin.
        pub total [31:30] get_mco2 / with_mco2: Mco2,
        /// Raw enable for the MCO2 prescaler; see `get_mco2` and `with_mco2`.
        pub total [29]    get_mco2pre_en / with_mco2pre_en: bool,
        /// Raw divisor for the MCO2 prescaler; see `get_mco2` and `with_mco2`.
        pub total [28:27] get_mco2pre_div / with_mco2pre_div: McoPre,
        /// Raw enable for the MCO1 prescaler; see `get_mco1` and `with_mco1`.
        pub total [26]    get_mco1pre_en / with_mco1pre_en: bool,
        /// Raw divisor for the MCO1 prescaler; see `get_mco1` and `with_mco1`.
        pub total [25:24] get_mco1pre_div / with_mco1pre_div: McoPre,
        /// Selects the clock fed to the I2S peripheral(s).
        pub total [23]    get_i2ssrc / with_i2ssrc: I2sSrc,
        /// Controls the clock output on the MCO1 pin.
        pub total [22:21] get_mco1 / with_mco1: Mco1,
        // TODO RTCPRE here
        /// Raw enable for the APB2 prescaler; see `get_ppre2` and `with_ppre2`.
        pub total [15]    get_ppre2_en / with_ppre2_en: bool,
        /// Raw divisor for the APB2 prescaler; see `get_ppre2` and
        /// `with_ppre2`.
        pub total [14:13] get_ppre2_div / with_ppre2_div: ApbPrescaler,
        /// Raw enable for the APB1 prescaler; see `get_ppre1` and `with_ppre1`.
        pub total [12]    get_ppre1_en / with_ppre1_en: bool,
        /// Raw divisor for the APB1 prescaler; see `get_ppre1` and
        /// `with_ppre1`.
        pub total [11:10] get_ppre1_div / with_ppre1_div: ApbPrescaler,
        /// Raw enable for the AHB prescaler; see `get_hpre` and `with_hpre`.
        pub total [ 7]    get_hpre_en / with_hpre_en: bool,
        /// Raw divisor for the AHB prescaler; see `get_hpre` and `with_hpre`.
        pub total [ 6: 4] get_hpre_div / with_hpre_div: AhbPrescaler,
        /// Reads as the currently selected system clock source.  After writing
        /// `Cfgr` with a new value chosen by `with_sw`, applications can
        /// read `Cfgr` and check this field to find out when their setting has
        /// taken effect.
        pub       [ 3: 2] get_sws / with_sws: ClockSwitch,
        /// Selects the system clock source.  Selections written to `Cfgr` do
        /// not take effect immediately; monitor by re-reading and checking
        /// `get_sws`.
        pub       [ 1: 0] get_sw / with_sw: ClockSwitch,
    }

    en_option_accessors!{
        /// Selects the (optional) prescaler used on the MCO2 output.
        ///
        /// This maps to the MCO2PRE field described in ST's documentation, but
        /// wraps up all the "don't care" patterns in `None`.  If you need to
        /// write a specific "don't care" pattern for some reason, see the raw
        /// accessors `with_mco2pre_div` and `with_mco2pre_en`.
        enable get_mco2pre_en / with_mco2pre_en
        value get_mco2pre_div / with_mco2pre_div : McoPre
        as get_mco2pre / with_mco2pre;

        /// Selects the (optional) prescaler used on the MCO1 output.
        ///
        /// This maps to the MCO1PRE field described in ST's documentation, but
        /// wraps up all the "don't care" patterns in `None`.  If you need to
        /// write a specific "don't care" pattern for some reason, see the raw
        /// accessors `with_mco1pre_div` and `with_mco1pre_en`.
        enable get_mco1pre_en / with_mco1pre_en
        value get_mco1pre_div / with_mco1pre_div : McoPre
        as get_mco1pre / with_mco1pre;

        /// Selects the (optional) prescaler used to derive the APB2 clock from
        /// the AHB clock.
        ///
        /// This maps to the PPRE2 field described in ST's documentation, but
        /// wraps up all the "don't care" patterns in `None`.  If you need to
        /// write a specific "don't care" pattern for some reason, see the raw
        /// accessors `with_ppre2_div` and `with_ppre2_en`.
        enable get_ppre2_en / with_ppre2_en
        value get_ppre2_div / with_ppre2_div : ApbPrescaler
        as get_ppre2 / with_ppre2;

        /// Selects the (optional) prescaler used to derive the APB1 clock from
        /// the AHB clock.
        ///
        /// This maps to the PPRE1 field described in ST's documentation, but
        /// wraps up all the "don't care" patterns in `None`.  If you need to
        /// write a specific "don't care" pattern for some reason, see the raw
        /// accessors `with_ppre1_div` and `with_ppre1_en`.
        enable get_ppre1_en / with_ppre1_en
        value get_ppre1_div / with_ppre1_div : ApbPrescaler
        as get_ppre1 / with_ppre1;

        /// Selects the (optional) prescaler used to derive the AHB clock from
        /// the system clock.
        ///
        /// This maps to the HPRE field described in ST's documentation, but
        /// wraps up all the "don't care" patterns in `None`.  If you need to
        /// write a specific "don't care" pattern for some reason, see the raw
        /// accessors `with_hpre_div` and `with_hpre_en`.
        enable get_hpre_en / with_hpre_en
        value get_hpre_div / with_hpre_div : AhbPrescaler
        as get_hpre / with_hpre;

    }
}

bit_enums! {
    /// Clocks that can be output on the MCO2 pin.
    pub bit_enum Mco2 {
        Sysclk = 0b00,
        Plli2s = 0b01,
        Hse = 0b10,
        Pll = 0b11,
    }

    /// Prescaler options for the MCOx pins.
    pub bit_enum McoPre {
        Div2 = 0b00,
        Div3 = 0b01,
        Div4 = 0b10,
        Div5 = 0b11,
    }

    /// Clocks that can be used to feed the I2S peripheral(s).
    pub bit_enum I2sSrc {
        Plli2s = 0,
        I2sCkin = 1,
    }

    /// Clocks that can be output on the MCO1 pin.
    pub bit_enum Mco1 {
        Hsi = 0b00,
        Lse = 0b01,
        Hse = 0b10,
        Pll = 0b11,
    }

    // TODO model RTCPRE

    /// Prescaler options for the APB clocks (relative to the AHB clock).
    pub bit_enum ApbPrescaler {
        Div2  = 0b00,
        Div4  = 0b01,
        Div8  = 0b10,
        Div16 = 0b11,
    }

    /// Prescaler options for the AHB clocks (relative to the system clock).
    pub bit_enum AhbPrescaler {
        Div2   = 0b000,
        Div4   = 0b001,
        Div8   = 0b010,
        Div16  = 0b011,
        Div64  = 0b100,
        Div128 = 0b101,
        Div256 = 0b110,
        Div512 = 0b111,
    }

    /// Clocks that can be used as the system clock source.
    pub bit_enum ClockSwitch {
        Hsi = 0b00,
        Hse = 0b01,
        Pll = 0b10,
    }
}

pub trait ClockDivisor {
    fn to_divisor(self) -> u32;
}

impl ClockDivisor for ApbPrescaler {
    fn to_divisor(self) -> u32 {
        match self {
            ApbPrescaler::Div2  =>  2,
            ApbPrescaler::Div4  =>  4,
            ApbPrescaler::Div8  =>  8,
            ApbPrescaler::Div16 => 16,
        }
    }
}

impl<T: ClockDivisor> ClockDivisor for Option<T> {
    fn to_divisor(self) -> u32 {
        self.map(|v| v.to_divisor()).unwrap_or(1)
    }
}

impl ClockDivisor for AhbPrescaler {
    fn to_divisor(self) -> u32 {
        match self {
            AhbPrescaler::Div2   =>   2,
            AhbPrescaler::Div4   =>   4,
            AhbPrescaler::Div8   =>   8,
            AhbPrescaler::Div16  =>  16,
            AhbPrescaler::Div64  =>  64,
            AhbPrescaler::Div128 => 128,
            AhbPrescaler::Div256 => 256,
            AhbPrescaler::Div512 => 512,
        }
    }
}

impl Pllcfgr {
    bitfield_accessors! {
        /// Prescaler for the PLL48 domain.
        ///
        /// Derives the clock used for USB OTG FS, SDIO, and RNG from the VCO
        /// frequency.
        pub total [27:24] get_pllq / with_pllq: u32,
        /// Input clock for both the main PLL and PLLI2S.
        pub total [22]    get_pllsrc / with_pllsrc: PllSource,
        /// Prescaler for the system clock domain.
        ///
        /// Derives the PLL's system clock output from the VCO frequency.  Note
        /// that this determines the PLL's system clock *output*; to make this
        /// the actual system clock, the PLL must be selected in `Cfgr`.
        pub total [17:16] get_pllp / with_pllp: Pllp,
        /// Multiplication factor for the VCO.
        ///
        /// Determines the internal VCO frequency by multiplying the PLL input
        /// frequency.
        pub total [14: 6] get_plln / with_plln: u32,
        /// Prescaler for the PLL input frequency.
        ///
        /// Derives the PLL input frequency from the PLL source.
        pub total [ 5: 0] get_pllm / with_pllm: u32,
    }
}

bit_enums! {
    /// Options for the PLL source clock.
    pub bit_enum PllSource {
        Hsi = 0,
        Hse = 1,
    }

    /// Options for deriving the system clock from the PLL's VCO.
    pub bit_enum Pllp {
        Div2 = 0b00,
        Div4 = 0b01,
        Div6 = 0b10,
        Div8 = 0b11,
    }
}

impl ClockDivisor for Pllp {
    fn to_divisor(self) -> u32 {
        match self {
            Pllp::Div2 => 2,
            Pllp::Div4 => 4,
            Pllp::Div6 => 6,
            Pllp::Div8 => 8,
        }
    }
}

pub const RCC_ADDRESS : usize = 0x40023800_usize;
