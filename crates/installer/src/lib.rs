#![no_std]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallStep {
    VerifyImage,
    PartitionDisk,
    DeploySystem,
    ConfigureBoot,
    FirstBootSeal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstallProfile {
    pub encrypted_disk: bool,
    pub dual_root_slots: bool,
    pub unattended_updates: bool,
}

impl InstallProfile {
    pub const fn secure_desktop() -> Self {
        Self {
            encrypted_disk: true,
            dual_root_slots: true,
            unattended_updates: false,
        }
    }

    pub const fn steps() -> [InstallStep; 5] {
        [
            InstallStep::VerifyImage,
            InstallStep::PartitionDisk,
            InstallStep::DeploySystem,
            InstallStep::ConfigureBoot,
            InstallStep::FirstBootSeal,
        ]
    }
}
