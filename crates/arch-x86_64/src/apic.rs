#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApicState {
    pub local_apic_enabled: bool,
    pub io_apic_enabled: bool,
    pub local_apic_addr: u64,
    pub io_apic_addr: u64,
}

pub struct ApicController {
    state: ApicState,
}

impl ApicController {
    pub const fn new() -> Self {
        Self {
            state: ApicState {
                local_apic_enabled: false,
                io_apic_enabled: false,
                local_apic_addr: 0,
                io_apic_addr: 0,
            },
        }
    }

    pub fn enable_local_apic(&mut self, addr: u64) {
        self.state.local_apic_addr = addr;
        self.state.local_apic_enabled = addr != 0;
    }

    pub fn enable_io_apic(&mut self, addr: u64) {
        self.state.io_apic_addr = addr;
        self.state.io_apic_enabled = addr != 0;
    }

    pub fn state(&self) -> ApicState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enables_local_and_io_apic() {
        let mut apic = ApicController::new();
        apic.enable_local_apic(0xfee0_0000);
        apic.enable_io_apic(0xfec0_0000);
        let s = apic.state();
        assert!(s.local_apic_enabled && s.io_apic_enabled);
    }
}
