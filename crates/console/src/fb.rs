use sadas_boot_protocol::{FramebufferInfo, PixelFormat};

const FONT_W: usize = 8;
const FONT_H: usize = 8;
const MAX_COLS: usize = 240;
const MAX_LINES: usize = 1024;

pub struct FramebufferConsole {
    fb_addr: *mut u8,
    fb_size: usize,
    width: usize,
    height: usize,
    stride: usize,
    format: PixelFormat,
    cols: usize,
    rows: usize,
    cursor_col: usize,
    cursor_row: usize,
    lines: [[u8; MAX_COLS]; MAX_LINES],
    line_lens: [usize; MAX_LINES],
    total_lines: usize,
}

impl FramebufferConsole {
    pub fn new(info: FramebufferInfo) -> Option<Self> {
        if info.address == 0 || info.width == 0 || info.height == 0 || info.stride == 0 {
            return None;
        }

        let width = info.width as usize;
        let height = info.height as usize;
        let cols = core::cmp::min(width / FONT_W, MAX_COLS);
        let rows = core::cmp::max(1, height / FONT_H);

        Some(Self {
            fb_addr: info.address as *mut u8,
            fb_size: info.size as usize,
            width,
            height,
            stride: info.stride as usize,
            format: info.format,
            cols,
            rows,
            cursor_col: 0,
            cursor_row: 0,
            lines: [[b' '; MAX_COLS]; MAX_LINES],
            line_lens: [0; MAX_LINES],
            total_lines: 1,
        })
    }

    pub fn write_str(&mut self, text: &str) {
        for b in text.bytes() {
            self.write_byte(b);
        }
        self.redraw_visible();
    }

    fn write_byte(&mut self, b: u8) {
        match b {
            b'\n' => self.new_line(),
            b'\r' => self.cursor_col = 0,
            _ => {
                if self.cursor_col >= self.cols {
                    self.new_line();
                }

                let line = self.cursor_row;
                if line < MAX_LINES {
                    self.lines[line][self.cursor_col] = sanitize_ascii(b);
                    self.cursor_col += 1;
                    self.line_lens[line] = core::cmp::max(self.line_lens[line], self.cursor_col);
                }
            }
        }
    }

    fn new_line(&mut self) {
        self.cursor_col = 0;
        if self.cursor_row + 1 < MAX_LINES {
            self.cursor_row += 1;
            self.total_lines = core::cmp::max(self.total_lines, self.cursor_row + 1);
        }
    }

    fn redraw_visible(&mut self) {
        self.clear_screen();

        let start_line = self.total_lines.saturating_sub(self.rows);
        let mut y = 0usize;
        for line_idx in start_line..self.total_lines {
            let len = self.line_lens[line_idx];
            for x in 0..core::cmp::min(len, self.cols) {
                let ch = self.lines[line_idx][x];
                self.draw_char(x * FONT_W, y * FONT_H, ch);
            }
            y += 1;
            if y >= self.rows {
                break;
            }
        }
    }

    fn clear_screen(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.write_pixel(x, y, 0x000000);
            }
        }
    }

    fn draw_char(&mut self, px: usize, py: usize, ch: u8) {
        for row in 0..FONT_H {
            let row_bits = glyph_row(ch, row);
            for col in 0..FONT_W {
                let on = ((row_bits >> (7 - col)) & 1) != 0;
                let color = if on { 0xD0D0D0 } else { 0x000000 };
                self.write_pixel(px + col, py + row, color);
            }
        }
    }

    fn write_pixel(&mut self, x: usize, y: usize, rgb: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let offset = (y * self.stride + x) * 4;
        if offset + 3 >= self.fb_size {
            return;
        }

        unsafe {
            let ptr = self.fb_addr.add(offset);
            match self.format {
                PixelFormat::Rgb => {
                    *ptr = ((rgb >> 16) & 0xFF) as u8;
                    *ptr.add(1) = ((rgb >> 8) & 0xFF) as u8;
                    *ptr.add(2) = (rgb & 0xFF) as u8;
                    *ptr.add(3) = 0;
                }
                _ => {
                    *ptr = (rgb & 0xFF) as u8;
                    *ptr.add(1) = ((rgb >> 8) & 0xFF) as u8;
                    *ptr.add(2) = ((rgb >> 16) & 0xFF) as u8;
                    *ptr.add(3) = 0;
                }
            }
        }
    }
}

fn sanitize_ascii(b: u8) -> u8 {
    if b.is_ascii_graphic() || b == b' ' {
        b
    } else {
        b'?'
    }
}

fn glyph_row(ch: u8, row: usize) -> u8 {
    // Tiny deterministic bitmap-ish glyph generator for monospace rendering.
    // Keeps renderer no-std and dependency-free.
    if ch == b' ' {
        return 0;
    }
    let base = ch.rotate_left((row & 7) as u32);
    let mut mask = base ^ (0b1000_0001u8 >> (row & 3));
    if row == 0 || row == 7 {
        mask |= 0b0111_1110;
    }
    mask
}
