#![no_std]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriverKind {
    Nvme,
    Sata,
    Ahci,
    UsbHid,
    UsbStorage,
    Pci,
    NetEthernet,
    NetWifi,
    Bluetooth,
    AudioHda,
    AudioUsb,
    GpuDisplay,
    GpuRender,
    Camera,
    Touchpad,
    Touchscreen,
    Sensors,
    PowerAcpi,
    Printer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriverStatus {
    Planned,
    Experimental,
    Stable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DriverDescriptor {
    pub kind: DriverKind,
    pub status: DriverStatus,
}

pub const fn default_driver_matrix() -> [DriverDescriptor; 19] {
    [
        DriverDescriptor {
            kind: DriverKind::Nvme,
            status: DriverStatus::Experimental,
        },
        DriverDescriptor {
            kind: DriverKind::Sata,
            status: DriverStatus::Experimental,
        },
        DriverDescriptor {
            kind: DriverKind::Ahci,
            status: DriverStatus::Experimental,
        },
        DriverDescriptor {
            kind: DriverKind::UsbHid,
            status: DriverStatus::Stable,
        },
        DriverDescriptor {
            kind: DriverKind::UsbStorage,
            status: DriverStatus::Experimental,
        },
        DriverDescriptor {
            kind: DriverKind::Pci,
            status: DriverStatus::Stable,
        },
        DriverDescriptor {
            kind: DriverKind::NetEthernet,
            status: DriverStatus::Experimental,
        },
        DriverDescriptor {
            kind: DriverKind::NetWifi,
            status: DriverStatus::Planned,
        },
        DriverDescriptor {
            kind: DriverKind::Bluetooth,
            status: DriverStatus::Planned,
        },
        DriverDescriptor {
            kind: DriverKind::AudioHda,
            status: DriverStatus::Planned,
        },
        DriverDescriptor {
            kind: DriverKind::AudioUsb,
            status: DriverStatus::Planned,
        },
        DriverDescriptor {
            kind: DriverKind::GpuDisplay,
            status: DriverStatus::Experimental,
        },
        DriverDescriptor {
            kind: DriverKind::GpuRender,
            status: DriverStatus::Planned,
        },
        DriverDescriptor {
            kind: DriverKind::Camera,
            status: DriverStatus::Planned,
        },
        DriverDescriptor {
            kind: DriverKind::Touchpad,
            status: DriverStatus::Experimental,
        },
        DriverDescriptor {
            kind: DriverKind::Touchscreen,
            status: DriverStatus::Planned,
        },
        DriverDescriptor {
            kind: DriverKind::Sensors,
            status: DriverStatus::Planned,
        },
        DriverDescriptor {
            kind: DriverKind::PowerAcpi,
            status: DriverStatus::Experimental,
        },
        DriverDescriptor {
            kind: DriverKind::Printer,
            status: DriverStatus::Planned,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_matrix_is_broad() {
        let matrix = default_driver_matrix();
        assert!(matrix.len() >= 15);
        assert!(matrix.iter().any(|d| d.kind == DriverKind::UsbHid));
        assert!(matrix.iter().any(|d| d.kind == DriverKind::GpuDisplay));
    }
}
