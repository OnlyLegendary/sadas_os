#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcpiInfo {
    pub rsdp_addr: u64,
    pub madt_addr: u64,
    pub local_apic_addr: u32,
    pub io_apic_addr: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcpiError {
    MissingRsdp,
    MissingMadt,
    Invalid,
}

pub fn discover_apic(rsdp_addr: u64, madt_bytes: &[u8]) -> Result<AcpiInfo, AcpiError> {
    if rsdp_addr == 0 {
        return Err(AcpiError::MissingRsdp);
    }
    if madt_bytes.len() < 44 {
        return Err(AcpiError::MissingMadt);
    }

    let local_apic_addr = u32::from_le_bytes([
        madt_bytes[36],
        madt_bytes[37],
        madt_bytes[38],
        madt_bytes[39],
    ]);

    let mut io_apic_addr = 0u32;
    let mut offset = 44usize;
    while offset + 2 <= madt_bytes.len() {
        let ty = madt_bytes[offset];
        let len = madt_bytes[offset + 1] as usize;
        if len < 2 || offset + len > madt_bytes.len() {
            return Err(AcpiError::Invalid);
        }

        if ty == 1 && len >= 12 {
            io_apic_addr = u32::from_le_bytes([
                madt_bytes[offset + 4],
                madt_bytes[offset + 5],
                madt_bytes[offset + 6],
                madt_bytes[offset + 7],
            ]);
            break;
        }
        offset += len;
    }

    Ok(AcpiInfo {
        rsdp_addr,
        madt_addr: 0,
        local_apic_addr,
        io_apic_addr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_madt_lapic_and_ioapic() {
        let mut madt = [0u8; 56];
        madt[36..40].copy_from_slice(&0xFEE0_0000u32.to_le_bytes());
        madt[44] = 1; // IO APIC
        madt[45] = 12;
        madt[48..52].copy_from_slice(&0xFEC0_0000u32.to_le_bytes());

        let info = discover_apic(0x1000, &madt).expect("acpi");
        assert_eq!(info.local_apic_addr, 0xFEE0_0000);
        assert_eq!(info.io_apic_addr, 0xFEC0_0000);
    }
}
