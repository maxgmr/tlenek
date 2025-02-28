//! Write to the VGA buffer.

use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

const VGA_BUFFER_ADDR: usize = 0xB8000;
const VGA_BUFFER_HEIGHT: usize = 25;
const VGA_BUFFER_WIDTH: usize = 80;

const PRINTABLE_RANGE_START: u8 = 0x20;
const PRINTABLE_RANGE_END: u8 = 0x7E;

const VGA_UNPRINTABLE: u8 = 0xFE;
const VGA_WHITESPACE: u8 = 0x20;

const BLINK_ATTR_MASK: u8 = 0b1000_0000;
const BG_ATTR_MASK: u8 = 0b0111_0000;
const FG_ATTR_MASK: u8 = 0b0000_1111;
const BLINK_ATTR_OFFSET: u8 = 7;
const BG_ATTR_OFFSET: u8 = 4;
const FG_ATTR_OFFSET: u8 = 0;

lazy_static! {
    /// Writes to the VGA buffer.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new(
        VgaBgColour::default(),
        VgaFgColour::default(),
        false
    ));
}

/// All the possible VGA foreground colours.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum VgaFgColour {
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
    #[default]
    White = 0xF,
}
impl From<VgaFgColour> for u8 {
    fn from(value: VgaFgColour) -> Self {
        value as u8
    }
}
impl TryFrom<u8> for VgaFgColour {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Self::Black),
            0x1 => Ok(Self::Blue),
            0x2 => Ok(Self::Green),
            0x3 => Ok(Self::Cyan),
            0x4 => Ok(Self::Red),
            0x5 => Ok(Self::Magenta),
            0x6 => Ok(Self::Brown),
            0x7 => Ok(Self::LightGray),
            0x8 => Ok(Self::DarkGray),
            0x9 => Ok(Self::LightBlue),
            0xA => Ok(Self::LightGreen),
            0xB => Ok(Self::LightCyan),
            0xC => Ok(Self::LightRed),
            0xD => Ok(Self::Pink),
            0xE => Ok(Self::Yellow),
            0xF => Ok(Self::White),
            _ => Err("Given value does not match an enum variant."),
        }
    }
}

/// All the possible VGA backgrond colours.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[allow(missing_docs)]
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
impl TryFrom<u8> for VgaBgColour {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(Self::Black),
            0x1 => Ok(Self::Blue),
            0x2 => Ok(Self::Green),
            0x3 => Ok(Self::Cyan),
            0x4 => Ok(Self::Red),
            0x5 => Ok(Self::Magenta),
            0x6 => Ok(Self::Brown),
            0x7 => Ok(Self::LightGray),
            _ => Err("Given value does not match an enum variant."),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
struct VgaAttr(u8);
impl VgaAttr {
    fn new(bg_colour: VgaBgColour, fg_colour: VgaFgColour, blink: bool) -> Self {
        Self(
            (if blink { 0b1000_0000 } else { 0b0000_0000 })
                | ((bg_colour as u8) << 4)
                | (fg_colour as u8),
        )
    }

    fn _bg(&self) -> VgaBgColour {
        VgaBgColour::try_from((self.0 & BG_ATTR_MASK) >> BG_ATTR_OFFSET).unwrap()
    }

    fn _fg(&self) -> VgaFgColour {
        VgaFgColour::try_from((self.0 & FG_ATTR_MASK) >> FG_ATTR_OFFSET).unwrap()
    }

    fn blink(&self) -> bool {
        (self.0 & BLINK_ATTR_MASK) != 0
    }

    fn set_bg(&mut self, bg: VgaBgColour) {
        self.overwrite_mask_offset(BG_ATTR_MASK, BG_ATTR_OFFSET, bg as u8);
    }

    fn set_fg(&mut self, fg: VgaFgColour) {
        self.overwrite_mask_offset(FG_ATTR_MASK, FG_ATTR_OFFSET, fg as u8);
    }

    fn set_blink(&mut self, blink: bool) {
        self.overwrite_mask_offset(
            BLINK_ATTR_MASK,
            BLINK_ATTR_OFFSET,
            if blink { 1 } else { 0 },
        );
    }

    // Clear the bits of the given mask, then write those bits with the given value
    fn overwrite_mask_offset(&mut self, mask: u8, offset: u8, value: u8) {
        *self = Self((self.0 & !mask) | (value << offset))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct VgaChar {
    text_byte: u8,
    attr: VgaAttr,
}

#[derive(Debug, Clone)]
#[repr(transparent)]
struct VgaBuffer {
    chars: [[Volatile<VgaChar>; VGA_BUFFER_WIDTH]; VGA_BUFFER_HEIGHT],
}

/// A VGA text writer.
#[derive(Debug)]
pub struct Writer {
    // Current position within the last row.
    column_position: usize,
    attr: VgaAttr,
    // We know that the VGA text buffer is valid for the whole runtime
    buffer: &'static mut VgaBuffer,
}
impl Writer {
    fn new(bg_colour: VgaBgColour, fg_colour: VgaFgColour, blink: bool) -> Self {
        Self {
            column_position: 0,
            attr: VgaAttr::new(bg_colour, fg_colour, blink),
            // SAFETY: The reference points to the constant VGA_BUFFER_ADDR, so we know it's valid.
            // Rust's bounds checking ensures we can't accidentally write outside the buffer, so
            // all subsequent operations are safe.
            buffer: unsafe { &mut *(VGA_BUFFER_ADDR as *mut VgaBuffer) },
        }
    }

    /// Set the background colour, foreground colour, and blink.
    pub fn set_attr(&mut self, bg_colour: VgaBgColour, fg_colour: VgaFgColour, blink: bool) {
        self.attr = VgaAttr::new(bg_colour, fg_colour, blink);
    }

    /// Set the background colour to the given [VgaBgColour].
    pub fn set_bg(&mut self, bg_colour: VgaBgColour) {
        self.attr.set_bg(bg_colour);
    }

    /// Set the foreground colour to the given [VgaFgColour].
    pub fn set_fg(&mut self, fg_colour: VgaFgColour) {
        self.attr.set_fg(fg_colour);
    }

    /// Enable blink.
    pub fn blink_on(&mut self) {
        self.set_blink(true);
    }

    /// Disable blink.
    pub fn blink_off(&mut self) {
        self.set_blink(false);
    }

    /// Toggle blink.
    pub fn blink_toggle(&mut self) {
        if self.attr.blink() {
            self.blink_off();
        } else {
            self.blink_on();
        }
    }

    /// Set blink.
    fn set_blink(&mut self, val: bool) {
        self.attr.set_blink(val);
    }

    fn write_char(&mut self, byte: u8) {
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
                    attr: self.attr,
                });

                self.column_position += 1;
            }
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                PRINTABLE_RANGE_START..=PRINTABLE_RANGE_END | b'\n' => self.write_char(byte),
                // not part of printable ASCII range
                _ => self.write_char(VGA_UNPRINTABLE),
            }
        }
    }

    fn new_line(&mut self) {
        // Shift all the lines upward
        for row in 1..VGA_BUFFER_HEIGHT {
            for col in 0..VGA_BUFFER_WIDTH {
                let c = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(c);
            }
        }

        self.clear_row(VGA_BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let vga_whitepsace_char = VgaChar {
            text_byte: VGA_WHITESPACE,
            attr: self.attr,
        };

        for col in 0..VGA_BUFFER_WIDTH {
            self.buffer.chars[row][col].write(vga_whitepsace_char);
        }
    }
}
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
