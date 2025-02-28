pub const VGA_BUF_ADDR: usize = 0xB8000;

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
