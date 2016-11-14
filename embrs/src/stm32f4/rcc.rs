//! Reset and Clock Control (RCC) support.

use arm_m;
use arm_m::reg::{AtomicReg, Reg};
use super::flash::FLASH;

/// At startup, before the RCC has been reconfigured, the STM32F4 runs at 16MHz.
pub const BOOT_CLOCK_HZ : u32 = 16_000_000;

/// The RCC's hardware register layout.
#[repr(C, packed)]
struct Registers {
    cr:            Reg<u32>,
    pllcfgr:       Reg<u32>,
    cfgr:          Reg<u32>,
    cir:           Reg<u32>,
    ahb1rstr:      Reg<u32>,
    ahb2rstr:      Reg<u32>,
    ahb3rstr:      Reg<u32>,
    _reserved_1c:  Reg<u32>,
    apb1rstr:      Reg<u32>,
    apb2rstr:      Reg<u32>,
    _reserved_28:  Reg<u32>,
    _reserved_2c:  Reg<u32>,
    ahb1enr:       Reg<u32>,
    ahb2enr:       Reg<u32>,
    ahb3enr:       Reg<u32>,
    _reserved_3c:  Reg<u32>,
    apb1enr:       Reg<u32>,
    apb2enr:       Reg<u32>,
    _reserved_48:  Reg<u32>,
    _reserved_4c:  Reg<u32>,
    ahb1lpenr:     Reg<u32>,
    ahb2lpenr:     Reg<u32>,
    ahb3lpenr:     Reg<u32>,
    _reserved_5c:  Reg<u32>,
    apb1lpenr:     Reg<u32>,
    apb2lpenr:     Reg<u32>,
    _reserved_68:  Reg<u32>,
    _reserved_6c:  Reg<u32>,
    bdcr:          Reg<u32>,
    csr:           Reg<u32>,
    _reserved_78:  Reg<u32>,
    _reserved_7c:  Reg<u32>,
    sscgr:         Reg<u32>,
    plli2scfgr:    Reg<u32>,
    pllsaicfgr:    Reg<u32>,
    dckcfgr:       Reg<u32>,
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
        pub total [27] get_plli2srdy / with_plli2srdy: bool,
        /// Turns the PLLI2S on/off.
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


const RCC_ADDRESS : usize = 0x40023800_usize;

/// RCC driver.
pub struct Rcc;

/// A clock configuration when using the High Speed External (HSE) crystal
/// oscillator and internal PLL.
pub struct ClockConfig {
    /// Frequency of external crystal.
    ///
    /// This is used to answer queries about the current clock speeds, but does
    /// not affect clock settings.
    pub crystal_hz: f32,
    /// Divisor used to derive the PLL input frequency from the crystal.  This
    /// maps to the `PLLM` field of `Pllcfgr`.
    pub crystal_divisor: u32,
    /// Multiplier used to derive the VCO frequency from the PLL input
    /// frequency.  This maps to the `PLLN` field of `Pllcfgr`.
    pub vco_multiplier: u32,
    /// Divisor used to derive the PLL general system clock output from the VCO
    /// frequency.  This maps to the `PLLP` field of `Pllcfgr`.
    pub general_divisor: Pllp,
    /// Divisor used to derive the PLL48 output from the VCO frequency.  This
    /// maps to the `PLLQ` field of `Pllcfgr`.
    pub pll48_divisor: u32,

    /// Optional divisor used to derive the AHB clock from the system clock.
    pub ahb_divisor: Option<AhbPrescaler>,
    /// Optional divisor used to derive the APB1 clock from the system clock.
    pub apb1_divisor: Option<ApbPrescaler>,
    /// Optional divisor used to derive the APB2 clock from the system clock.
    pub apb2_divisor: Option<ApbPrescaler>,

    /// Number of wait states desired for Flash accesses.  This is not strictly
    /// part of the clock configuration, but it needs to be carefully changed at
    /// the same time as the CPU frequency if code might be running from Flash,
    /// so it's included here.
    pub flash_latency: u32,
}

/// Describes types that name peripherals in the RCC.  This is used to fake
/// overloading on `ApbPeripheral` and `AhbPeripheral`.  It isn't designed to
/// be implemented by types outside this module.
pub trait PeripheralName {
    /// Alters the RCC to enable the clock for the named peripheral.
    ///
    /// # Panics
    ///
    /// If the named peripheral's clock cannot be controlled.  Controllable
    /// clocks have a bit allocated in one of the RCC's `AxBxENR` registers.
    /// Check the STM32F4 Reference Manual.
    fn enable_clock(self, rcc: &Rcc);
}

impl Rcc {
    fn reg(&self) -> &Registers {
        unsafe {
            &*(RCC_ADDRESS as *const Registers)
        }
    }

    /// Enables clock to peripheral `p` if that clock can be controlled.
    ///
    /// The implementation uses barriers to ensure that the clock is enabled
    /// before return.  This works around ST's erratum 2.1.13.
    ///
    /// # Panics
    ///
    /// If `p`'s clock cannot be controlled.  Controllable clocks have a bit
    /// allocated in one of the RCC's `AxBxENR` registers.  Check the STM32F4
    /// Reference Manual.
    pub fn enable_clock<P: PeripheralName>(&self, p: P) {
        // re-dispatch for bus-specific behavior
        p.enable_clock(self);
        // ensure the write took effect.
        arm_m::data_synchronization_barrier();
    }

    pub fn read_cr(&self) -> Cr {
        Cr(self.reg().cr.get())
    }

    pub fn write_cr(&self, v: Cr) {
        self.reg().cr.set(v.0)
    }

    pub fn update_cr<F: FnOnce(Cr) -> Cr>(&self, f: F) {
        self.write_cr(f(self.read_cr()))
    }

    pub fn read_cfgr(&self) -> Cfgr {
        Cfgr(self.reg().cfgr.get())
    }

    pub fn write_cfgr(&self, v: Cfgr) {
        self.reg().cfgr.set(v.0)
    }

    pub fn update_cfgr<F: FnOnce(Cfgr) -> Cfgr>(&self, f: F) {
        self.write_cfgr(f(self.read_cfgr()))
    }

    pub fn read_pllcfgr(&self) -> Pllcfgr {
        Pllcfgr(self.reg().pllcfgr.get())
    }

    pub fn write_pllcfgr(&self, v: Pllcfgr) {
        self.reg().pllcfgr.set(v.0)
    }

    pub fn update_pllcfgr<F: FnOnce(Pllcfgr) -> Pllcfgr>(&self, f: F) {
        self.write_pllcfgr(f(self.read_pllcfgr()))
    }

    /// Reconfigures the RCC to the given `ClockConfig`.
    ///
    /// This is done via a two-step process, where we first switch to the 16MHz
    /// internal oscillator, and then reconfigure the PLL and HSE.  This method
    /// is conservative.  It allows applications to move between any two
    /// `ClockConfig` settings at will, but an application-specific clock
    /// switching algorithm could likely perform better.
    ///
    /// Note that this method also reconfigures the number of Flash wait states.
    pub fn configure_clocks(&self, cfg: &ClockConfig) {
        // Switch to the internal 16MHz oscillator while messing with the PLL.
        // First, ensure the HSI is enabled.
        self.update_cr(|v| v.with_hsion(true));
        while !self.read_cr().get_hsirdy() {}
        // Do the switch.
        self.update_cfgr(|v| v.with_sw(ClockSwitch::Hsi));
        while self.read_cfgr().get_sws() != Ok(ClockSwitch::Hsi) {}

        // Turn off the PLL so we can reconfigure it safely.
        self.update_cr(|v| v.with_pllon(false));
        while self.read_cr().get_pllrdy() {}

        // Apply divisors to both buses and Flash before increasing clock
        // frequency.  (Doing it in the other order may temporarily drive things
        // outside their rated range.)
        self.update_cfgr(|v|
                         v.with_hpre(cfg.ahb_divisor)
                         .with_ppre1(cfg.apb1_divisor)
                         .with_ppre2(cfg.apb2_divisor));

        FLASH.update_acr(|v| v.with_latency(cfg.flash_latency));

        // Switch on the external crystal oscillator.
        self.update_cr(|v| v.with_hseon(true));
        while !self.read_cr().get_hserdy() {}

        // Configure the PLL.
        self.update_pllcfgr(|v| v.with_pllm(cfg.crystal_divisor)
                            .with_plln(cfg.vco_multiplier)
                            .with_pllp(cfg.general_divisor)
                            .with_pllq(cfg.pll48_divisor)
                            .with_pllsrc(PllSource::Hse));

        // Turn on the PLL.
        self.update_cr(|v| v.with_pllon(true));
        while !self.read_cr().get_pllrdy() {}

        // Select the PLL as our clock source.
        self.update_cfgr(|v| v.with_sw(ClockSwitch::Pll));
        while self.read_cfgr().get_sws() != Ok(ClockSwitch::Pll) {}
    }
}

/// Names the processor's AHB buses.  This can be seen as a bounded-range
/// integer type if you squint.
#[derive(Copy, Clone)]
pub enum AhbBus {
    Ahb1,
    Ahb2,
    Ahb3,
}

/// Names the processor's AHB-connected peripherals.
#[derive(Copy, Clone, Debug)]
pub enum AhbPeripheral {
    GpioA,
    GpioB,
    GpioC,
    GpioD,
    // TODO: this is obviously not comprehensive!
}

impl AhbPeripheral {
    /// Retrieves metadata about a peripheral: its bus, and the bit indices
    /// for its reset and clock-enable control bits.
    ///
    /// Note: In C++ I would have encoded this information into the enum value.
    /// I technically have that option in Rust, but doing it this way exposes
    /// more information to the compiler and is leading to better code
    /// generation, with fewer unnecessary range checks, in practice.
    pub fn describe(self) -> (AhbBus, Option<u32>, Option<u32>) {
        match self {
            AhbPeripheral::GpioA => (AhbBus::Ahb1, Some(0), Some(0)),
            AhbPeripheral::GpioB => (AhbBus::Ahb1, Some(1), Some(1)),
            AhbPeripheral::GpioC => (AhbBus::Ahb1, Some(2), Some(2)),
            AhbPeripheral::GpioD => (AhbBus::Ahb1, Some(3), Some(3)),
        }
    }
}

impl PeripheralName for AhbPeripheral {
    fn enable_clock(self, rcc: &Rcc) {
        if let (bus, _, Some(ena)) = self.describe() {
            let reg = match bus {
                AhbBus::Ahb1 => &rcc.reg().ahb1enr,
                AhbBus::Ahb2 => &rcc.reg().ahb2enr,
                AhbBus::Ahb3 => &rcc.reg().ahb3enr,
            };

            reg.atomic_or(1 << ena)
        } else {
            panic!("cannot control clock for {:?}", self)
        }
    }
}

/// Names the processor's APB buses.  This can be seen as a bounded-range
/// integer type if you squint.
#[derive(Copy, Clone)]
pub enum ApbBus {
    Apb1,
    Apb2,
}

/// Names the processor's APB-connected peripherals.
#[derive(Copy, Clone)]
pub enum ApbPeripheral {
    Tim2,
    Tim3,
    // TODO: this is obviously not comprehensive!
}

impl ApbPeripheral {
    /// Retrieves metadata about a peripheral: its bus, and its control bit
    /// index in the reset and clock-enable registers.
    ///
    /// (APB peripherals use consistent reset and clock-enable bit positions,
    /// so we don't have to model them separately as we do for AHB.  Also, all
    /// the APB peripherals allow both clock and reset control, so we don't need
    /// `Option`.  This also improves code generation.)
    pub fn describe(self) -> (ApbBus, u32) {
        match self {
            ApbPeripheral::Tim2 => (ApbBus::Apb1, 0),
            ApbPeripheral::Tim3 => (ApbBus::Apb1, 1),
        }
    }
}

impl PeripheralName for ApbPeripheral {
    fn enable_clock(self, rcc: &Rcc) {
        let (bus, idx) = self.describe();
        let reg = match bus {
            ApbBus::Apb1 => &rcc.reg().apb1enr,
            ApbBus::Apb2 => &rcc.reg().apb2enr,
        };

        reg.atomic_or(1 << idx)
    }
}

/// Shared instance of the `Rcc` driver.
pub static RCC: Rcc = Rcc;
