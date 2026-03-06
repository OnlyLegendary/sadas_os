#![no_std]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElfInfo {
    pub entry: u64,
    pub phoff: u64,
    pub phnum: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElfError {
    TooSmall,
    BadMagic,
    NotElf64,
    NotLittleEndian,
}

pub fn parse_elf64(image: &[u8]) -> Result<ElfInfo, ElfError> {
    if image.len() < 64 {
        return Err(ElfError::TooSmall);
    }
    if &image[0..4] != b"\x7fELF" {
        return Err(ElfError::BadMagic);
    }
    if image[4] != 2 {
        return Err(ElfError::NotElf64);
    }
    if image[5] != 1 {
        return Err(ElfError::NotLittleEndian);
    }

    let entry = u64::from_le_bytes(image[24..32].try_into().map_err(|_| ElfError::TooSmall)?);
    let phoff = u64::from_le_bytes(image[32..40].try_into().map_err(|_| ElfError::TooSmall)?);
    let phnum = u16::from_le_bytes(image[56..58].try_into().map_err(|_| ElfError::TooSmall)?);

    Ok(ElfInfo {
        entry,
        phoff,
        phnum,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_elf64_header() {
        let mut elf = [0u8; 64];
        elf[0..4].copy_from_slice(b"\x7fELF");
        elf[4] = 2;
        elf[5] = 1;
        elf[24..32].copy_from_slice(&0x400000u64.to_le_bytes());
        elf[32..40].copy_from_slice(&64u64.to_le_bytes());
        elf[56..58].copy_from_slice(&2u16.to_le_bytes());

        let info = parse_elf64(&elf).expect("elf");
        assert_eq!(info.entry, 0x400000);
        assert_eq!(info.phoff, 64);
        assert_eq!(info.phnum, 2);
    }
}
