//! Reset and Clock Control (RCC) support.
//!
//! This module provides a higher-level driver with some useful algorithms.  For
//! more direct access to the hardware, see the `raw` submodule.

use arm_m;
use arm_m::reg::AtomicReg;
use super::flash::FLASH;

pub mod raw;
pub use self::raw::{AhbPrescaler, ApbPrescaler, Cr, Cfgr, Pllcfgr};
pub use self::raw::Pllp as SysPrescaler;

use self::raw::ClockDivisor;

/// At startup, before the RCC has been reconfigured, the STM32F4 runs at 16MHz.
pub const BOOT_CLOCK_HZ : u32 = 16_000_000;

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
    pub general_divisor: SysPrescaler,
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

/// Packages up the various internal clock speeds, which can be computed from a
/// `ClockConfig`.  (We compute them all at once because they're
/// interdependent.)
pub struct ClockSpeeds {
    pub cpu: f32,
    pub ahb: f32,
    pub apb1: f32,
    pub apb2: f32,
    pub pll48: f32,
}

impl ClockSpeeds {
    pub fn get_clock_for<P: PeripheralName>(&self, p: P) -> f32 {
        p.get_clock(self)
    }
}

impl ClockConfig {
    pub fn compute_speeds(&self) -> ClockSpeeds {
        let vco_in_hz = self.crystal_hz / (self.crystal_divisor as f32);
        let vco_out_hz = vco_in_hz * (self.vco_multiplier as f32);
        let cpu = vco_out_hz / (self.general_divisor.to_divisor() as f32);
        ClockSpeeds {
            cpu: cpu,
            ahb: cpu / (self.ahb_divisor.to_divisor() as f32),
            apb1: cpu / (self.apb1_divisor.to_divisor() as f32),
            apb2: cpu / (self.apb2_divisor.to_divisor() as f32),
            pll48: vco_out_hz / (self.pll48_divisor as f32),
        }
    }
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

    /// Gets the clock speed for this peripheral, given the current speeds.
    fn get_clock(self, speeds: &ClockSpeeds) -> f32;
}

impl Rcc {
    fn reg(&self) -> &raw::Registers {
        unsafe {
            &*(raw::RCC_ADDRESS as *const raw::Registers)
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
        self.update_cfgr(|v| v.with_sw(raw::ClockSwitch::Hsi));
        while self.read_cfgr().get_sws() != Ok(raw::ClockSwitch::Hsi) {}

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
                            .with_pllsrc(raw::PllSource::Hse));

        // Turn on the PLL.
        self.update_cr(|v| v.with_pllon(true));
        while !self.read_cr().get_pllrdy() {}

        // Select the PLL as our clock source.
        self.update_cfgr(|v| v.with_sw(raw::ClockSwitch::Pll));
        while self.read_cfgr().get_sws() != Ok(raw::ClockSwitch::Pll) {}
    }
}

/// Names the processor's AHB buses.  This can be seen as a bounded-range
/// integer type if you squint.
#[derive(Copy, Clone)]
pub enum AhbBus {
    Ahb1 = 0,
    Ahb2 = 1,
    Ahb3 = 2,
}

/// Internal utility macro used to reduce peripheral enum boilerplate.
macro_rules! peripheral_enum {
    (
        $(#[$m:meta])*
        pub enum $tyname:ident ($bty:ident) {
            $(
                $(#[$e_m:meta])*
                p $name:ident = $bus:tt | $idx:tt | $rst:tt | $clk:tt | $lp:tt,
            )*
        }
    ) => {
        $(#[$m])*
        #[derive(Copy, Clone, Eq, PartialEq)]
        #[repr(u32)]
        pub enum $tyname {
            $(
                $(#[$e_m])*
                $name = ($bty::$bus as u32) | ($idx << 8)
                    | ($rst << 16) | ($clk << 17) | ($lp << 18),
            )*
        }

        impl $tyname {
            #[inline]
            pub fn get_bus(self) -> $bty {
                let idx = (self as u32) & 0xF;
                unsafe { ::core::mem::transmute(idx as u8) }
            }

            #[inline]
            pub fn get_bit_index(self) -> u32 {
                ((self as u32) >> 8) & 0x1F
            }

            #[inline]
            pub fn has_rst(self) -> bool {
                ((self as u32) & (1 << 16)) != 0
            }

            #[inline]
            pub fn has_enr(self) -> bool {
                ((self as u32) & (1 << 17)) != 0
            }

            #[inline]
            pub fn has_lpenr(self) -> bool {
                ((self as u32) & (1 << 18)) != 0
            }
        }
    };
}

peripheral_enum! {
    /// Names the processor's AHB-connected peripherals, for the purposes of
    /// clock and reset domain control.
    pub enum AhbPeripheral (AhbBus) {
        //               bus    idx rst clk lp
        p GpioA        = Ahb1 |  0 | 1 | 1 | 1,
        p GpioB        = Ahb1 |  1 | 1 | 1 | 1,
        p GpioC        = Ahb1 |  2 | 1 | 1 | 1,
        p GpioD        = Ahb1 |  3 | 1 | 1 | 1,
        p GpioE        = Ahb1 |  4 | 1 | 1 | 1,
        p GpioF        = Ahb1 |  5 | 1 | 1 | 1,
        p GpioG        = Ahb1 |  6 | 1 | 1 | 1,
        p GpioH        = Ahb1 |  7 | 1 | 1 | 1,
        p GpioI        = Ahb1 |  8 | 1 | 1 | 1,
        #[cfg(feature = "soc_family:stm32f4[23]")]
        p GpioJ        = Ahb1 |  9 | 1 | 1 | 1,
        #[cfg(feature = "soc_family:stm32f4[23]")]
        p GpioK        = Ahb1 | 10 | 1 | 1 | 1,
        // 11 is unused
        p Crc          = Ahb1 | 12 | 1 | 1 | 1,
        // 13-14 are unused
        p FlashIface   = Ahb1 | 15 | 0 | 0 | 1,
        p Sram1        = Ahb1 | 16 | 0 | 0 | 1,
        p Sram2        = Ahb1 | 17 | 0 | 0 | 1,
        p BackupSram   = Ahb1 | 18 | 0 | 1 | 1,
        #[cfg(feature = "soc_family:stm32f4[23]")]
        p Sram3        = Ahb1 | 19 | 0 | 0 | 1,
        p CcmDataRam   = Ahb1 | 20 | 0 | 1 | 0,
        p Dma1         = Ahb1 | 21 | 1 | 1 | 1,
        p Dma2         = Ahb1 | 22 | 1 | 1 | 1,
        #[cfg(feature = "soc_family:stm32f4[23]")]
        p Dma2d        = Ahb1 | 23 | 1 | 1 | 1,
        // 24 is unused.
        p Ethernet     = Ahb1 | 25 | 1 | 1 | 1,
        p EthernetTx   = Ahb1 | 26 | 0 | 1 | 1,
        p EthernetRx   = Ahb1 | 27 | 0 | 1 | 1,
        p EthernetPtp  = Ahb1 | 28 | 0 | 1 | 1,
        p UsbOtgHs     = Ahb1 | 29 | 1 | 1 | 1,
        p UsbOtgHsUlpi = Ahb1 | 30 | 0 | 1 | 1,
        // 31 is unused.

        // AHB2
        p Dcmi         = Ahb2 |  0 | 1 | 1 | 1,
        // 1-3 unused
        p Cryp         = Ahb2 |  4 | 1 | 1 | 1,
        p Hash         = Ahb2 |  5 | 1 | 1 | 1,
        p Rng          = Ahb2 |  6 | 1 | 1 | 1,
        p UsbOtgFs     = Ahb2 |  7 | 1 | 1 | 1,
        // 8 - 31 unused

        // AHB3
        p Fsmc         = Ahb3 |  0 | 1 | 1 | 1,
        // 1 - 31 unused
    }
}

impl PeripheralName for AhbPeripheral {
    fn enable_clock(self, rcc: &Rcc) {
        if !self.has_enr() {
            panic!("cannot control clock for AHB{} idx {}",
                   (self.get_bus() as u32) + 1,
                   self.get_bit_index())
        }

        rcc.reg()
            .ahb_enr[self.get_bus() as usize]
            .atomic_or(1 << self.get_bit_index())
    }

    fn get_clock(self, speeds: &ClockSpeeds) -> f32 {
        speeds.ahb
    }
}

/// Names the processor's APB buses.  This can be seen as a bounded-range
/// integer type if you squint.
#[derive(Copy, Clone)]
pub enum ApbBus {
    Apb1 = 0,
    Apb2 = 1,
}

peripheral_enum! {
    /// Names the processor's APB-connected peripherals, for the purposes of
    /// clock and reset domain control.
    pub enum ApbPeripheral (ApbBus) {
        //               bus    idx rst clk lp
        p Tim2         = Apb1 |  0 | 1 | 1 | 1,
        p Tim3         = Apb1 |  1 | 1 | 1 | 1,
        p Tim4         = Apb1 |  2 | 1 | 1 | 1,
        p Tim5         = Apb1 |  3 | 1 | 1 | 1,
        p Tim6         = Apb1 |  4 | 1 | 1 | 1,
        p Tim7         = Apb1 |  5 | 1 | 1 | 1,
        p Tim12        = Apb1 |  6 | 1 | 1 | 1,
        p Tim13        = Apb1 |  7 | 1 | 1 | 1,
        p Tim14        = Apb1 |  8 | 1 | 1 | 1,
        // 9-10
        p Wwdg         = Apb1 | 11 | 1 | 1 | 1,
        // 12-13
        p Spi2         = Apb1 | 14 | 1 | 1 | 1,
        p Spi3         = Apb1 | 15 | 1 | 1 | 1,
        // 16
        p Usart2       = Apb1 | 17 | 1 | 1 | 1,
        p Usart3       = Apb1 | 18 | 1 | 1 | 1,
        p Uart4        = Apb1 | 19 | 1 | 1 | 1,
        p Uart5        = Apb1 | 20 | 1 | 1 | 1,
        p I2c1         = Apb1 | 21 | 1 | 1 | 1,
        p I2c2         = Apb1 | 22 | 1 | 1 | 1,
        p I2c3         = Apb1 | 23 | 1 | 1 | 1,
        // 24
        p Can1         = Apb1 | 25 | 1 | 1 | 1,
        p Can2         = Apb1 | 26 | 1 | 1 | 1,
        // 27
        p Pwr          = Apb1 | 28 | 1 | 1 | 1,
        p Dac          = Apb1 | 29 | 1 | 1 | 1,
        #[cfg(feature = "soc_family:stm32f4[23]")]
        p Uart7        = Apb1 | 30 | 1 | 1 | 1,
        #[cfg(feature = "soc_family:stm32f4[23]")]
        p Uart8        = Apb1 | 31 | 1 | 1 | 1,

        // APB2
        p Tim1         = Apb2 |  0 | 1 | 1 | 1,
        p Tim8         = Apb2 |  1 | 1 | 1 | 1,
        // 2-3
        p Usart1       = Apb2 |  4 | 1 | 1 | 1,
        p Usart6       = Apb2 |  5 | 1 | 1 | 1,
        // 6-7
        p Adc1         = Apb2 |  8 | 1 | 1 | 1,
        p Adc2         = Apb2 |  9 | 0 | 1 | 1,
        p Adc3         = Apb2 | 10 | 0 | 1 | 1,
        p Sdio         = Apb2 | 11 | 1 | 1 | 1,
        p Spi1         = Apb2 | 12 | 1 | 1 | 1,
        #[cfg(feature = "soc_family:stm32f4[23]")]
        p Spi4         = Apb2 | 13 | 1 | 1 | 1,
        p Syscfg       = Apb2 | 14 | 1 | 1 | 1,
        // 15
        p Tim9         = Apb2 | 16 | 1 | 1 | 1,
        p Tim10        = Apb2 | 17 | 1 | 1 | 1,
        p Tim11        = Apb2 | 18 | 1 | 1 | 1,
        // 19
        #[cfg(feature = "soc_family:stm32f4[23]")]
        p Spi5         = Apb2 | 20 | 1 | 1 | 1,
        #[cfg(feature = "soc_family:stm32f4[23]")]
        p Spi6         = Apb2 | 21 | 1 | 1 | 1,
        #[cfg(feature = "soc_family:stm32f4[23]")]
        p Sai1         = Apb2 | 22 | 1 | 1 | 1,
        // 23-25
        #[cfg(feature = "soc_family:stm32f4[23]")]
        p Ltdc         = Apb2 | 26 | 1 | 1 | 1,
        // 27-31
    }
}

impl PeripheralName for ApbPeripheral {
    fn enable_clock(self, rcc: &Rcc) {
        if !self.has_enr() {
            panic!("cannot control clock for APB{} idx {}",
                   (self.get_bus() as u32) + 1,
                   self.get_bit_index())
        }

        rcc.reg()
            .apb_enr[self.get_bus() as usize]
            .atomic_or(1 << self.get_bit_index())
    }
    fn get_clock(self, speeds: &ClockSpeeds) -> f32 {
        match self.get_bus() {
            ApbBus::Apb1 => speeds.apb1,
            ApbBus::Apb2 => speeds.apb2,
        }
    }
}

/// Shared instance of the `Rcc` driver.
pub static RCC: Rcc = Rcc;
