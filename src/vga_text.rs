use core::fmt;

use volatile::Volatile;

pub const VGA_BUFFER_ADDR: usize = 0xB8000;
const VGA_BUFFER_HEIGHT: usize = 25;
const VGA_BUFFER_WIDTH: usize = 80;

const PRINTABLE_RANGE_START: u8 = 0x20;
const PRINTABLE_RANGE_END: u8 = 0x7E;

const VGA_UNKNOWN_CHAR: u8 = 0xFE;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum VgaFgColour {
    #[default]
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xA,
    LightCyan = 0xB,
    LightRed = 0xC,
    Pink = 0xD,
    Yellow = 0xE,
    White = 0xF,
}
impl From<VgaFgColour> for u8 {
    fn from(value: VgaFgColour) -> Self {
        value as u8
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum VgaBgColour {
    #[default]
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
}
impl From<VgaBgColour> for u8 {
    fn from(value: VgaBgColour) -> Self {
        value as u8
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
struct VgaAttrByte(u8);
impl VgaAttrByte {
    fn new(bg_colour: VgaBgColour, fg_colour: VgaFgColour, blink: bool) -> Self {
        Self(
            (if blink { 0b1000_0000 } else { 0b0000_0000 })
                | ((bg_colour as u8) << 4)
                | (fg_colour as u8),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct VgaChar {
    text_byte: u8,
    attr_byte: VgaAttrByte,
}

#[derive(Debug, Clone)]
#[repr(transparent)]
struct VgaBuffer {
    chars: [[Volatile<VgaChar>; VGA_BUFFER_WIDTH]; VGA_BUFFER_HEIGHT],
}

#[derive(Debug)]
pub struct Writer {
    column_position: usize,
    attr_byte: VgaAttrByte,
    // We know that the VGA text buffer is valid for the whole runtime
    buffer: &'static mut VgaBuffer,
}
impl Writer {
    pub fn write_char(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= VGA_BUFFER_WIDTH {
                    self.new_line();
                }

                let row = VGA_BUFFER_HEIGHT - 1;
                let col = self.column_position;

                // Declare volatile to ensure the compiler never optimises away the writes
                self.buffer.chars[row][col].write(VgaChar {
                    text_byte: byte,
                    attr_byte: self.attr_byte,
                });

                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                PRINTABLE_RANGE_START..=PRINTABLE_RANGE_END | b'\n' => self.write_char(byte),
                // not part of printable ASCII range
                _ => self.write_char(VGA_UNKNOWN_CHAR),
            }
        }
    }

    fn new_line(&mut self) {
        todo!()
    }
}
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn print_str(string: &str, bg: VgaBgColour, fg: VgaFgColour, blink: bool) {
    let mut writer = Writer {
        column_position: 0,
        attr_byte: VgaAttrByte::new(bg, fg, blink),
        buffer: unsafe { &mut *(VGA_BUFFER_ADDR as *mut VgaBuffer) },
    };
    writer.write_string(string);
}

/// Write a character in VGA text mode.
///
/// UNSAFE: The raw pointer (and the offset) must be valid!
pub unsafe fn putc_vga_text(
    vga_buf: *mut u8,
    offset: usize,
    char: u8,
    bg_colour: VgaBgColour,
    fg_colour: VgaFgColour,
    blink: bool,
) {
    *vga_buf.offset(offset as isize * 2) = char;
    *vga_buf.offset((offset as isize * 2) + 1) = VgaAttrByte::new(bg_colour, fg_colour, blink).0;
}
