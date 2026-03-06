#![allow(clippy::cast_possible_truncation)]

const COM1: u16 = 0x3F8;

#[inline(always)]
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nostack, nomem, preserves_flags)
    );
}

#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nostack, nomem, preserves_flags)
    );
    value
}

pub fn init_com1() {
    unsafe {
        outb(COM1 + 1, 0x00); // disable interrupts
        outb(COM1 + 3, 0x80); // enable DLAB
        outb(COM1, 0x03); // divisor low (38400 baud)
        outb(COM1 + 1, 0x00); // divisor high
        outb(COM1 + 3, 0x03); // 8N1
        outb(COM1 + 2, 0xC7); // FIFO
        outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }
}

pub fn write_byte(byte: u8) {
    unsafe {
        while (inb(COM1 + 5) & 0x20) == 0 {}
        outb(COM1, byte);
    }
}

pub fn write_bytes(bytes: &[u8]) {
    for &b in bytes {
        write_byte(b);
    }
}
