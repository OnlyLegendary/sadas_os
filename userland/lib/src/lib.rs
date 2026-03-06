use sadas_syscall::{SyscallNumber, SyscallRequest, SyscallResponse};

pub fn syscall(req: SyscallRequest) -> SyscallResponse {
    // Host model of kernel trap for phase integration.
    match req.nr {
        SyscallNumber::Write => SyscallResponse::ok(req.arg1),
        SyscallNumber::Read => SyscallResponse::ok(0),
        SyscallNumber::Exit => SyscallResponse::ok(0),
        SyscallNumber::Spawn => SyscallResponse::ok(42),
        SyscallNumber::Wait => SyscallResponse::ok(42),
        SyscallNumber::Open => SyscallResponse::ok(3),
        SyscallNumber::Close => SyscallResponse::ok(0),
        SyscallNumber::ReadDir => SyscallResponse::ok(0),
        SyscallNumber::Stat => SyscallResponse::ok(0),
        SyscallNumber::Mmap => SyscallResponse::ok(req.arg1),
    }
}

pub fn write_console(bytes: &[u8]) -> SyscallResponse {
    syscall(SyscallRequest {
        nr: SyscallNumber::Write,
        arg0: 1,
        arg1: bytes.len() as u64,
        arg2: bytes.as_ptr() as u64,
        arg3: 0,
    })
}
