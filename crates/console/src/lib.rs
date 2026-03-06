#![no_std]
#![allow(static_mut_refs)]

pub mod fb;
pub mod serial;

use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use fb::FramebufferConsole;
use sadas_boot_protocol::FramebufferInfo;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        }
    }
}

static LOCK: AtomicBool = AtomicBool::new(false);
static mut FB: Option<FramebufferConsole> = None;
static LAST_HASH: AtomicU64 = AtomicU64::new(0);
static LAST_COUNT: AtomicU32 = AtomicU32::new(0);
static DROPPED: AtomicU32 = AtomicU32::new(0);
static SERIAL_READY: AtomicBool = AtomicBool::new(false);
static TICKS: AtomicUsize = AtomicUsize::new(0);

pub fn init_serial() {
    serial::init_com1();
    SERIAL_READY.store(true, Ordering::SeqCst);
}

pub fn init_framebuffer(info: FramebufferInfo) {
    with_lock(|| unsafe {
        FB = FramebufferConsole::new(info);
    });
}

pub fn log(level: Level, args: fmt::Arguments<'_>) {
    let mut buf = StackBuf::new();
    let _ = write!(&mut buf, "[sadas][{}] {}", level.as_str(), args);
    buf.push_byte(b'\n');
    log_bytes(buf.as_bytes());
}

pub fn log_bytes(bytes: &[u8]) {
    let tick = TICKS.fetch_add(1, Ordering::Relaxed);
    if rate_limited(bytes, tick as u64) {
        return;
    }

    if SERIAL_READY.load(Ordering::Relaxed) {
        serial::write_bytes(bytes);
    }

    with_lock(|| unsafe {
        if let Some(fb) = FB.as_mut() {
            if let Ok(text) = core::str::from_utf8(bytes) {
                fb.write_str(text);
            }
        }
    });
}

pub fn logging_backend(bytes: &[u8]) {
    log_bytes(bytes);
}

fn rate_limited(bytes: &[u8], salt: u64) -> bool {
    let hash = hash(bytes) ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let last = LAST_HASH.load(Ordering::Relaxed);

    if hash == last {
        let count = LAST_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        if count > 8 {
            DROPPED.fetch_add(1, Ordering::Relaxed);
            return true;
        }
    } else {
        let dropped = DROPPED.swap(0, Ordering::Relaxed);
        LAST_HASH.store(hash, Ordering::Relaxed);
        LAST_COUNT.store(0, Ordering::Relaxed);
        if dropped > 0 {
            let mut msg = StackBuf::new();
            let _ = write!(
                &mut msg,
                "[sadas][WARN] rate-limit dropped {} log lines\n",
                dropped
            );
            if SERIAL_READY.load(Ordering::Relaxed) {
                serial::write_bytes(msg.as_bytes());
            }
        }
    }

    false
}

fn hash(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn with_lock<F: FnOnce()>(f: F) {
    while LOCK
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {}
    f();
    LOCK.store(false, Ordering::Release);
}

struct StackBuf {
    data: [u8; 320],
    len: usize,
}

impl StackBuf {
    const fn new() -> Self {
        Self {
            data: [0; 320],
            len: 0,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }

    fn push_byte(&mut self, b: u8) {
        if self.len < self.data.len() {
            self.data[self.len] = b;
            self.len += 1;
        }
    }
}

impl Write for StackBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &b in s.as_bytes() {
            self.push_byte(b);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Debug, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Info, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Warn, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log($crate::Level::Error, format_args!($($arg)*))
    };
}
