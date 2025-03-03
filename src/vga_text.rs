//! Write to the VGA buffer.

use core::{
    default::Default,
    fmt::{self, Write},
};

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts;

const VGA_BUFFER_ADDR: usize = 0xB8000;
const VGA_BUFFER_HEIGHT: usize = 25;
const VGA_BUFFER_WIDTH: usize = 80;

const PRINTABLE_RANGE_START: u8 = 0x20;
const PRINTABLE_RANGE_END: u8 = 0x7E;

const VGA_UNPRINTABLE: u8 = 0xFE;
const VGA_WHITESPACE: u8 = 0x20;
const BACKSPACE_BYTE: u8 = 0x08;

const BLINK_ATTR_MASK: u8 = 0b1000_0000;
const BG_ATTR_MASK: u8 = 0b0111_0000;
const FG_ATTR_MASK: u8 = 0b0000_1111;
const BLINK_ATTR_OFFSET: u8 = 7;
const BG_ATTR_OFFSET: u8 = 4;
const FG_ATTR_OFFSET: u8 = 0;

const TAB_WIDTH: usize = 4;

/// Prints to VGA buffer using format syntax.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_text::_print(format_args!($($arg)*)));
}

/// Prints to VGA buffer using format syntax with a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

lazy_static! {
    /// Writes to the VGA buffer.
    static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new(
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
struct VgaAttr(u8);
impl VgaAttr {
    fn new(bg: VgaBgColour, fg: VgaFgColour, blink: bool) -> Self {
        Self((if blink { 0b1000_0000 } else { 0b0000_0000 }) | ((bg as u8) << 4) | (fg as u8))
    }

    fn bg(&self) -> VgaBgColour {
        VgaBgColour::try_from((self.0 & BG_ATTR_MASK) >> BG_ATTR_OFFSET).unwrap()
    }

    fn fg(&self) -> VgaFgColour {
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
impl Default for VgaAttr {
    fn default() -> Self {
        Self::new(VgaBgColour::default(), VgaFgColour::default(), false)
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
struct Writer {
    // Current position within the last row.
    column_position: usize,
    attr: VgaAttr,
    // We know that the VGA text buffer is valid for the whole runtime
    buffer: &'static mut VgaBuffer,
}
impl Writer {
    fn new(bg: VgaBgColour, fg: VgaFgColour, blink: bool) -> Self {
        Self {
            column_position: 0,
            attr: VgaAttr::new(bg, fg, blink),
            // SAFETY: The reference points to the constant VGA_BUFFER_ADDR, so we know it's valid.
            // Rust's bounds checking ensures we can't accidentally write outside the buffer, so
            // all subsequent operations are safe.
            buffer: unsafe { &mut *(VGA_BUFFER_ADDR as *mut VgaBuffer) },
        }
    }

    fn write_char(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\t' => self.write_tab(),
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

    fn erase_char(&mut self) {
        if self.column_position == 0 {
            self.back_line();
        }

        let row = VGA_BUFFER_HEIGHT - 1;
        let col = self.column_position - 1;

        // Declare volatile to ensure the compiler never optimises away the writes
        self.buffer.chars[row][col].write(VgaChar {
            text_byte: VGA_WHITESPACE,
            attr: self.attr,
        });

        self.column_position -= 1;
    }

    fn write_tab(&mut self) {
        if (self.column_position % TAB_WIDTH) == 0 {
            // Cursor already aligned, tab ahead
            self.write_char(VGA_WHITESPACE);
        }
        // Move forward to the next tab position
        while (self.column_position % TAB_WIDTH) != 0 {
            self.write_char(VGA_WHITESPACE);
        }
    }

    fn new_line(&mut self) {
        // Shift all the lines upward
        for row in 1..VGA_BUFFER_HEIGHT {
            for col in 0..VGA_BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        self.clear_row(VGA_BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn back_line(&mut self) {
        // Shift all the lines downward
        for row in (0..(VGA_BUFFER_HEIGHT - 1)).rev() {
            for col in 0..VGA_BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row + 1][col].write(character);
            }
        }

        self.column_position = VGA_BUFFER_WIDTH;
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte, tab, or newline
                PRINTABLE_RANGE_START..=PRINTABLE_RANGE_END | b'\n' | b'\t' => {
                    self.write_char(byte)
                }
                // backspace
                BACKSPACE_BYTE => self.erase_char(),
                // not part of printable ASCII range
                _ => self.write_char(VGA_UNPRINTABLE),
            }
        }
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
impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

macro_rules! vga_get {
    [$($(#[$doc:meta])*($fn_name:ident, $out:ty, $getter:ident)),*] => {
        $(
            $(#[$doc])*
            pub fn $fn_name() -> $out {
                interrupts::without_interrupts(|| {
                    WRITER.lock().attr.$getter()
                })
            }
        )*
    };
}
vga_get![
    /// Get the current [VgaBgColour].
    (vga_bg, VgaBgColour, bg),
    /// Get the current [VgaFgColour].
    (vga_fg, VgaFgColour, fg),
    /// Check if the VGA blink bit is set.
    (vga_blink, bool, blink)
];

macro_rules! vga_set {
    [$($(#[$doc:meta])*($fn_name:ident, $in:ty, $setter:ident)),*] => {
        $(
            $(#[$doc])*
            pub fn $fn_name(val: $in) {
                interrupts::without_interrupts(|| {
                    WRITER.lock().attr.$setter(val);
                });
            }
        )*
    };
}
vga_set![
    /// Set the VGA background colour to the given [VgaBgColour].
    (set_vga_bg, VgaBgColour, set_bg),
    /// Set the VGA foreground colour to the given [VgaFgColour].
    (set_vga_fg, VgaFgColour, set_fg),
    /// Set the VGA blink bit to the given value.
    (set_vga_blink, bool, set_blink)
];

/// Set the [VgaBgColour], the [VgaFgColour], and the VGA blink value.
pub fn set_vga_attr(bg: VgaBgColour, fg: VgaFgColour, blink: bool) {
    interrupts::without_interrupts(|| {
        WRITER.lock().attr = VgaAttr::new(bg, fg, blink);
    });
}

/// Set the VGA text attribute to the default values.
pub fn set_default_vga_attr() {
    interrupts::without_interrupts(|| {
        WRITER.lock().attr = VgaAttr::default();
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const WRITELN_FAIL_MSG: &str = "writeln fail :(";

    fn clear_buffer(writer: &mut Writer) {
        for _ in 0..(VGA_BUFFER_HEIGHT + 1) {
            writeln!(writer).expect(WRITELN_FAIL_MSG);
        }
    }

    #[test_case]
    fn simple_println() {
        println!("Hello, world!");
    }

    #[test_case]
    fn many_println() {
        interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            for _ in 0..200 {
                writeln!(writer, "ping!").expect(WRITELN_FAIL_MSG);
            }
        });
    }

    #[test_case]
    fn println_appears_on_screen() {
        let s = "Hello, world!";

        interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            clear_buffer(&mut writer);
            writeln!(writer, "\n{}", s).expect(WRITELN_FAIL_MSG);
            for (i, c) in s.chars().enumerate() {
                let vga_char = writer.buffer.chars[VGA_BUFFER_HEIGHT - 2][i].read();
                assert_eq!(char::from(vga_char.text_byte), c);
            }
        });
    }

    #[test_case]
    fn print_wrap() {
        let test_char = 'O';
        let test_next_line_char = 'I';

        interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            clear_buffer(&mut writer);
            for _ in 0..(VGA_BUFFER_WIDTH) {
                write!(writer, "{}", test_char).expect(WRITELN_FAIL_MSG);
            }
            write!(writer, "{}", test_next_line_char).expect(WRITELN_FAIL_MSG);

            for i in 0..VGA_BUFFER_WIDTH {
                let vga_char = writer.buffer.chars[VGA_BUFFER_HEIGHT - 2][i].read();
                assert_eq!(char::from(vga_char.text_byte), test_char);
            }
            let vga_char = writer.buffer.chars[VGA_BUFFER_HEIGHT - 1][0].read();
            assert_eq!(char::from(vga_char.text_byte), test_next_line_char);
        });
    }

    #[test_case]
    fn print_tab() {
        interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            clear_buffer(&mut writer);
            assert_eq!(writer.column_position, 0);
            write!(writer, "\t").expect(WRITELN_FAIL_MSG);
            assert_eq!(writer.column_position, TAB_WIDTH);
        });
    }

    #[test_case]
    fn set_get_attr() {
        set_vga_bg(VgaBgColour::Red);
        assert_eq!(vga_bg(), VgaBgColour::Red);

        set_vga_fg(VgaFgColour::LightCyan);
        assert_eq!(vga_fg(), VgaFgColour::LightCyan);

        set_vga_blink(true);
        assert!(vga_blink());

        set_vga_attr(VgaBgColour::Brown, VgaFgColour::Pink, false);
        assert_eq!(vga_bg(), VgaBgColour::Brown);
        assert_eq!(vga_fg(), VgaFgColour::Pink);
        assert!(!vga_blink());
    }

    #[test_case]
    fn vga_attr_new() {
        let expected: u8 = 0b1001_1010;
        let bg: VgaBgColour = VgaBgColour::try_from(0b0001).unwrap();
        let fg: VgaFgColour = VgaFgColour::try_from(0b1010).unwrap();
        let blink: bool = true;
        let vga_attr = VgaAttr::new(bg, fg, blink);
        assert_eq!(vga_attr.0, expected);
    }

    #[test_case]
    fn vga_attr_fg() {
        let expected: u8 = 0b0000_1111;
        let fg: VgaFgColour = VgaFgColour::try_from(0b1111).unwrap();
        let bg: VgaBgColour = VgaBgColour::try_from(0b0000).unwrap();
        let mut vga_attr = VgaAttr::new(bg, fg, false);
        assert_eq!(vga_attr.0, expected);

        let expected: u8 = 0b0000_1010;
        vga_attr.set_fg(VgaFgColour::try_from(0b1010).unwrap());
        assert_eq!(vga_attr.0, expected);
    }

    #[test_case]
    fn vga_attr_bg() {
        let expected: u8 = 0b0111_0000;
        let fg: VgaFgColour = VgaFgColour::try_from(0b0000).unwrap();
        let bg: VgaBgColour = VgaBgColour::try_from(0b0111).unwrap();
        let mut vga_attr = VgaAttr::new(bg, fg, false);
        assert_eq!(vga_attr.0, expected);

        let expected: u8 = 0b0101_0000;
        vga_attr.set_bg(VgaBgColour::try_from(0b0101).unwrap());
        assert_eq!(vga_attr.0, expected);
    }

    #[test_case]
    fn vga_attr_blink() {
        let expected: u8 = 0b1000_0000;
        let mut vga_attr = VgaAttr::new(
            VgaBgColour::try_from(0).unwrap(),
            VgaFgColour::try_from(0).unwrap(),
            true,
        );
        assert_eq!(vga_attr.0, expected);

        let expected: u8 = 0b0000_0000;
        vga_attr.set_blink(false);
        assert_eq!(vga_attr.0, expected);
    }

    #[test_case]
    fn bad_vga_fg() {
        let _ = VgaFgColour::try_from(0x10).unwrap_err();
    }

    #[test_case]
    fn bad_vga_bg() {
        let _ = VgaBgColour::try_from(0x8).unwrap_err();
    }
}
