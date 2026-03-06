use crate::PAGE_SIZE;

pub const MAX_MAPPINGS: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MapFlags {
    pub writable: bool,
    pub executable: bool,
    pub user: bool,
}

impl MapFlags {
    pub const KERNEL_RW: Self = Self {
        writable: true,
        executable: false,
        user: false,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Mapping {
    pub virt: u64,
    pub phys: u64,
    pub flags: MapFlags,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MapError {
    Misaligned,
    TableFull,
    DuplicateVirtual,
}

/// Early-page-table model for deterministic mapping decisions.
///
/// This model validates page alignment and duplicate mappings before
/// architecture-specific page-table writes are implemented.
pub struct PageMapper {
    mappings: [Option<Mapping>; MAX_MAPPINGS],
    count: usize,
}

impl PageMapper {
    pub const fn new() -> Self {
        Self {
            mappings: [None; MAX_MAPPINGS],
            count: 0,
        }
    }

    pub fn map_page(&mut self, virt: u64, phys: u64, flags: MapFlags) -> Result<(), MapError> {
        if !is_page_aligned(virt) || !is_page_aligned(phys) {
            return Err(MapError::Misaligned);
        }
        if self.count >= MAX_MAPPINGS {
            return Err(MapError::TableFull);
        }
        if self.mappings.iter().flatten().any(|m| m.virt == virt) {
            return Err(MapError::DuplicateVirtual);
        }

        self.mappings[self.count] = Some(Mapping { virt, phys, flags });
        self.count += 1;
        Ok(())
    }

    pub fn map_identity_region(
        &mut self,
        start: u64,
        len: u64,
        flags: MapFlags,
    ) -> Result<(), MapError> {
        if !is_page_aligned(start) || !is_page_aligned(len) {
            return Err(MapError::Misaligned);
        }

        let mut addr = start;
        let end = start.saturating_add(len);
        while addr < end {
            self.map_page(addr, addr, flags)?;
            addr = addr.saturating_add(PAGE_SIZE as u64);
        }

        Ok(())
    }

    pub fn mappings_count(&self) -> usize {
        self.count
    }
}

fn is_page_aligned(addr: u64) -> bool {
    (addr as usize) % PAGE_SIZE == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_misaligned_pages() {
        let mut p = PageMapper::new();
        assert_eq!(
            p.map_page(0x1003, 0x2000, MapFlags::KERNEL_RW),
            Err(MapError::Misaligned)
        );
    }

    #[test]
    fn maps_identity_region() {
        let mut p = PageMapper::new();
        p.map_identity_region(0x4000, 0x3000, MapFlags::KERNEL_RW)
            .expect("maps");
        assert_eq!(p.mappings_count(), 3);
    }
}
