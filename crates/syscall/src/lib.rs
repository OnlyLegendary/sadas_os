#![no_std]

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyscallNumber {
    Write = 0,
    Read = 1,
    Exit = 2,
    Spawn = 3,
    Wait = 4,
    Open = 5,
    Close = 6,
    ReadDir = 7,
    Stat = 8,
    Mmap = 9,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyscallRequest {
    pub nr: SyscallNumber,
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyscallResponse {
    pub status: i64,
    pub value: u64,
}

impl SyscallResponse {
    pub const fn ok(value: u64) -> Self {
        Self { status: 0, value }
    }

    pub const fn err(code: i64) -> Self {
        Self {
            status: -code,
            value: 0,
        }
    }
}
