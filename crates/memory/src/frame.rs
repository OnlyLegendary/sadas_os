use crate::PAGE_SIZE;

pub const MAX_FRAMES: usize = 32_768; // 128 MiB worth of 4KiB frames tracked in fixed metadata.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MemoryType {
    Reserved = 0,
    Conventional = 7,
    BootServicesCode = 3,
    BootServicesData = 4,
    LoaderCode = 1,
    LoaderData = 2,
    AcpiReclaim = 9,
    AcpiNvs = 10,
    Mmio = 11,
    Unknown = u32::MAX,
}

impl MemoryType {
    pub fn is_usable(self) -> bool {
        matches!(
            self,
            MemoryType::Conventional
                | MemoryType::BootServicesCode
                | MemoryType::BootServicesData
                | MemoryType::LoaderCode
                | MemoryType::LoaderData
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct MemoryDescriptor {
    pub ty: MemoryType,
    pub physical_start: u64,
    pub page_count: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReservedRange {
    pub start: u64,
    pub len: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameStats {
    pub tracked_frames: usize,
    pub usable_frames: usize,
    pub allocated_frames: usize,
    pub free_frames: usize,
}

/// Fixed-size frame allocator backed by a frame-usage bitmap.
///
/// Safety invariants:
/// - Frame addresses are always page-aligned.
/// - `alloc()` returns unique frames until released.
/// - Reserved ranges are never handed out.
pub struct FrameAllocator {
    frame_addrs: [u64; MAX_FRAMES],
    used: [bool; MAX_FRAMES],
    count: usize,
    allocated: usize,
}

impl FrameAllocator {
    pub const fn empty() -> Self {
        Self {
            frame_addrs: [0; MAX_FRAMES],
            used: [true; MAX_FRAMES],
            count: 0,
            allocated: 0,
        }
    }

    pub fn initialize(&mut self, map: &[MemoryDescriptor], reserved: &[ReservedRange]) {
        self.count = 0;
        self.allocated = 0;

        for slot in self.used.iter_mut() {
            *slot = true;
        }

        for desc in map {
            if !desc.ty.is_usable() {
                continue;
            }

            let region_start = align_up(desc.physical_start as usize, PAGE_SIZE) as u64;
            let region_len = desc.page_count.saturating_mul(PAGE_SIZE as u64);
            let region_end = desc.physical_start.saturating_add(region_len);

            let mut addr = region_start;
            while addr.saturating_add(PAGE_SIZE as u64) <= region_end && self.count < MAX_FRAMES {
                if !is_reserved(addr, PAGE_SIZE as u64, reserved) {
                    self.frame_addrs[self.count] = addr;
                    self.used[self.count] = false;
                    self.count += 1;
                }
                addr = addr.saturating_add(PAGE_SIZE as u64);
            }
        }
    }

    pub fn alloc(&mut self) -> Option<u64> {
        for i in 0..self.count {
            if !self.used[i] {
                self.used[i] = true;
                self.allocated += 1;
                return Some(self.frame_addrs[i]);
            }
        }
        None
    }

    pub fn free(&mut self, frame_addr: u64) -> bool {
        for i in 0..self.count {
            if self.frame_addrs[i] == frame_addr {
                if self.used[i] {
                    self.used[i] = false;
                    self.allocated = self.allocated.saturating_sub(1);
                    return true;
                }
                return false;
            }
        }
        false
    }

    pub fn stats(&self) -> FrameStats {
        FrameStats {
            tracked_frames: self.count,
            usable_frames: self.count,
            allocated_frames: self.allocated,
            free_frames: self.count.saturating_sub(self.allocated),
        }
    }
}

fn is_reserved(start: u64, len: u64, reserved: &[ReservedRange]) -> bool {
    let end = start.saturating_add(len);
    reserved.iter().any(|r| {
        let r_end = r.start.saturating_add(r.len);
        start < r_end && r.start < end
    })
}

fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (value + align - 1) & !(align - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_and_respects_reserved_ranges() {
        let mut fa = FrameAllocator::empty();
        let map = [MemoryDescriptor {
            ty: MemoryType::Conventional,
            physical_start: 0x1000,
            page_count: 8,
        }];
        let reserved = [ReservedRange {
            start: 0x3000,
            len: PAGE_SIZE as u64,
        }];

        fa.initialize(&map, &reserved);
        let s = fa.stats();
        assert_eq!(s.tracked_frames, 7);

        let mut seen_reserved = false;
        for _ in 0..7 {
            let f = fa.alloc().expect("frame");
            if f == 0x3000 {
                seen_reserved = true;
            }
        }
        assert!(!seen_reserved);
        assert!(fa.alloc().is_none());
    }
}
