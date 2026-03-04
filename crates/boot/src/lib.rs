#![no_std]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootStage {
    Firmware,
    Loader,
    KernelHandoff,
    Init,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootPlan {
    pub secure_boot_required: bool,
    pub measured_boot: bool,
    pub fallback_slot: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootStatusReport {
    pub stage: BootStage,
    pub message: &'static str,
}

impl BootPlan {
    pub const fn hardened() -> Self {
        Self {
            secure_boot_required: true,
            measured_boot: true,
            fallback_slot: true,
        }
    }

    pub const fn stage_order() -> [BootStage; 4] {
        [
            BootStage::Firmware,
            BootStage::Loader,
            BootStage::KernelHandoff,
            BootStage::Init,
        ]
    }

    pub const fn qemu_boot_stub_report() -> [BootStatusReport; 4] {
        [
            BootStatusReport {
                stage: BootStage::Firmware,
                message: "UEFI firmware entered",
            },
            BootStatusReport {
                stage: BootStage::Loader,
                message: "bootloader verified kernel image",
            },
            BootStatusReport {
                stage: BootStage::KernelHandoff,
                message: "kernel handoff successful",
            },
            BootStatusReport {
                stage: BootStage::Init,
                message: "Sadas OS boot stub: hello from init",
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_stub_ends_in_init() {
        let report = BootPlan::qemu_boot_stub_report();
        assert_eq!(report[3].stage, BootStage::Init);
        assert!(report[3].message.contains("hello"));
    }
}
