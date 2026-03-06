use sadas_syscall::{SyscallNumber, SyscallRequest};

fn main() {
    let _ = sadas_userland::write_console(b"[init] starting /bin/shell\n");
    let _ = sadas_userland::syscall(SyscallRequest {
        nr: SyscallNumber::Spawn,
        arg0: 0,
        arg1: 0,
        arg2: 0,
        arg3: 0,
    });
}
