#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

pub type LogBackendFn = fn(&[u8]);

static LOGGER_BACKEND: AtomicUsize = AtomicUsize::new(0);

pub fn set_backend(backend: LogBackendFn) {
    LOGGER_BACKEND.store(backend as usize, Ordering::SeqCst);
}

pub fn log(level: LogLevel, msg: &str) {
    let backend_ptr = LOGGER_BACKEND.load(Ordering::SeqCst);
    if backend_ptr == 0 {
        return;
    }

    let mut buf = [0u8; 192];
    let len = format_line(&mut buf, level, msg);

    let backend: LogBackendFn = unsafe { core::mem::transmute::<usize, LogBackendFn>(backend_ptr) };
    backend(&buf[..len]);
}

pub fn info(msg: &str) {
    log(LogLevel::Info, msg);
}

pub fn debug(msg: &str) {
    log(LogLevel::Debug, msg);
}

pub fn warn(msg: &str) {
    log(LogLevel::Warn, msg);
}

pub fn error(msg: &str) {
    log(LogLevel::Error, msg);
}

pub fn format_line(buf: &mut [u8], level: LogLevel, msg: &str) -> usize {
    let prefix = b"[sadas][";
    let suffix = b"] ";
    let level_bytes = level.as_str().as_bytes();

    let mut i = 0;
    i = copy_into(buf, i, prefix);
    i = copy_into(buf, i, level_bytes);
    i = copy_into(buf, i, suffix);
    i = copy_into(buf, i, msg.as_bytes());

    if i < buf.len() {
        buf[i] = b'\n';
        i += 1;
    }

    i
}

fn copy_into(buf: &mut [u8], mut index: usize, src: &[u8]) -> usize {
    for &b in src {
        if index >= buf.len() {
            return index;
        }
        buf[index] = b;
        index += 1;
    }
    index
}

pub fn serial_com1_backend(bytes: &[u8]) {
    for &byte in bytes {
        unsafe {
            core::arch::asm!(
                "out dx, al",
                in("dx") 0x3F8u16,
                in("al") byte,
                options(nostack, nomem, preserves_flags)
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_labels_are_stable() {
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn formats_line_with_prefix_and_newline() {
        let mut buf = [0u8; 64];
        let len = format_line(&mut buf, LogLevel::Warn, "disk offline");
        let line = core::str::from_utf8(&buf[..len]).expect("utf8");
        assert_eq!(line, "[sadas][WARN] disk offline\n");
    }
}
