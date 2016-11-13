use arm_m::reg::Reg;

#[repr(C, packed)]
struct Registers {
    acr: Reg<u32>,
}

const FLASH_ADDRESS : usize = 0x40023c00;

bit_wrappers! {
    pub struct Acr(pub u32);
}

impl Acr {
    bitfield_accessors! {
        pub total [12] get_dcrst / with_dcrst: bool,
        pub total [11] get_icrst / with_icrst: bool,
        pub total [10] get_dcen / with_dcen: bool,
        pub total [9] get_icen / with_icen: bool,
        pub total [8] get_prften / with_prften: bool,
        pub total [2:0] get_latency / with_latency: u32,
    }
}

pub struct Flash;

impl Flash {
    fn reg(&self) -> &Registers {
        unsafe {
            &*(FLASH_ADDRESS as *const Registers)
        }
    }

    pub fn read_acr(&self) -> Acr {
        Acr(self.reg().acr.get())
    }

    pub fn write_acr(&self, v: Acr) {
        self.reg().acr.set(v.0)
    }

    pub fn update_acr<F: FnOnce(Acr) -> Acr>(&self, f: F) {
        self.write_acr(f(self.read_acr()))
    }
}

pub static FLASH : Flash = Flash;
