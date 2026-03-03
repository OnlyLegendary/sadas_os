#![no_std]

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Syscall {
    Yield = 0,
    Send = 1,
    Receive = 2,
    MapMemory = 3,
    SetTheme = 4,
    SetPrivacyLevel = 5,
    CreateSurface = 6,
    CreateWindow = 7,
    SetWindowLayout = 8,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThemeMode {
    Light = 0,
    Dark = 1,
    HighContrast = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivacyLevel {
    Relaxed = 0,
    Standard = 1,
    Strict = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowState {
    Tiled = 0,
    Floating = 1,
    Fullscreen = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SysResult {
    pub code: u32,
    pub value: u64,
}
