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
}
