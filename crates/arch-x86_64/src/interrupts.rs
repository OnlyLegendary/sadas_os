#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct IdtEntry {
    pub vector: u8,
    pub present: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FaultInfo {
    pub vector: u8,
    pub error_code: u64,
    pub instruction_pointer: u64,
    pub cr2: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdtState {
    pub loaded: bool,
    pub entries_installed: usize,
}

pub struct InterruptController {
    entries: [IdtEntry; 256],
    loaded: bool,
}

impl InterruptController {
    pub const fn new() -> Self {
        Self {
            entries: [IdtEntry {
                vector: 0,
                present: false,
            }; 256],
            loaded: false,
        }
    }

    pub fn install_exception_handlers(&mut self) {
        for vector in 0u8..=31 {
            self.entries[vector as usize] = IdtEntry {
                vector,
                present: true,
            };
        }
        self.entries[14] = IdtEntry {
            vector: 14,
            present: true,
        };
    }

    pub fn load(&mut self) {
        self.loaded = true;
    }

    pub fn state(&self) -> IdtState {
        let mut count = 0;
        for e in &self.entries {
            if e.present {
                count += 1;
            }
        }
        IdtState {
            loaded: self.loaded,
            entries_installed: count,
        }
    }
}

pub fn format_page_fault(info: FaultInfo, out: &mut [u8; 160]) -> usize {
    let mut w = Writer::new(out);
    w.push_str("[sadas][ERROR] page fault vec=");
    w.push_u64(info.vector as u64);
    w.push_str(" ip=");
    w.push_hex(info.instruction_pointer);
    w.push_str(" cr2=");
    w.push_hex(info.cr2);
    w.push_str(" ec=");
    w.push_hex(info.error_code);
    w.push_byte(b'\n');
    w.len()
}

struct Writer<'a> {
    out: &'a mut [u8],
    len: usize,
}

impl<'a> Writer<'a> {
    fn new(out: &'a mut [u8]) -> Self {
        Self { out, len: 0 }
    }
    fn len(&self) -> usize {
        self.len
    }
    fn push_byte(&mut self, b: u8) {
        if self.len < self.out.len() {
            self.out[self.len] = b;
            self.len += 1;
        }
    }
    fn push_str(&mut self, s: &str) {
        for &b in s.as_bytes() {
            self.push_byte(b);
        }
    }
    fn push_u64(&mut self, mut n: u64) {
        let mut buf = [0u8; 20];
        let mut i = buf.len();
        if n == 0 {
            i -= 1;
            buf[i] = b'0';
        }
        while n > 0 {
            i -= 1;
            buf[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        for &b in &buf[i..] {
            self.push_byte(b);
        }
    }
    fn push_hex(&mut self, mut n: u64) {
        self.push_str("0x");
        let mut buf = [0u8; 16];
        let mut i = buf.len();
        if n == 0 {
            i -= 1;
            buf[i] = b'0';
        }
        while n > 0 {
            i -= 1;
            let d = (n & 0xF) as u8;
            buf[i] = if d < 10 { b'0' + d } else { b'a' + (d - 10) };
            n >>= 4;
        }
        for &b in &buf[i..] {
            self.push_byte(b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_exception_vectors() {
        let mut idt = InterruptController::new();
        idt.install_exception_handlers();
        idt.load();
        let state = idt.state();
        assert!(state.loaded);
        assert!(state.entries_installed >= 32);
    }
}
