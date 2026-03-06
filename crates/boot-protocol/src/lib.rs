#![no_std]

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PixelFormat {
    Rgb,
    Bgr,
    U8,
    Unknown,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FramebufferInfo {
    pub address: u64,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryMapInfo {
    pub entries_addr: u64,
    pub entry_count: u64,
    pub entry_size: u64,
    pub descriptor_version: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InitfsInfo {
    pub address: u64,
    pub size: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootInfo {
    pub memory_map: MemoryMapInfo,
    pub framebuffer: FramebufferInfo,
    pub rsdp_addr: u64,
    pub initfs: InitfsInfo,
    pub cmdline_addr: u64,
    pub cmdline_len: u64,
}

impl BootInfo {
    pub const fn empty() -> Self {
        Self {
            memory_map: MemoryMapInfo {
                entries_addr: 0,
                entry_count: 0,
                entry_size: 0,
                descriptor_version: 0,
            },
            framebuffer: FramebufferInfo {
                address: 0,
                size: 0,
                width: 0,
                height: 0,
                stride: 0,
                format: PixelFormat::Unknown,
            },
            rsdp_addr: 0,
            initfs: InitfsInfo {
                address: 0,
                size: 0,
            },
            cmdline_addr: 0,
            cmdline_len: 0,
        }
    }
}
