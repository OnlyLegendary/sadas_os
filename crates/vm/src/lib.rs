#![no_std]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VmProfile {
    pub page_size: u32,
    pub userspace_aslr: bool,
    pub kernel_guard_pages: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryLayout {
    pub kernel_start: u64,
    pub kernel_end: u64,
    pub user_start: u64,
    pub user_end: u64,
}

impl VmProfile {
    pub const fn secure_default() -> Self {
        Self {
            page_size: 4096,
            userspace_aslr: true,
            kernel_guard_pages: true,
        }
    }

    pub const fn baseline_layout() -> MemoryLayout {
        MemoryLayout {
            kernel_start: 0xFFFF_8000_0000_0000,
            kernel_end: 0xFFFF_FFFF_FFFF_FFFF,
            user_start: 0x0000_0000_0040_0000,
            user_end: 0x0000_7FFF_FFFF_FFFF,
        }
    }
}
