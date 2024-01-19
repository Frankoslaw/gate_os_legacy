use crate::api::vga::color;
use crate::api::vga::{Color, Palette};
use crate::sys;

use bit_field::BitField;
use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use core::fmt::Write;
use core::{fmt, ptr};
use font_constants::BACKUP_CHAR;
use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};

use conquer_once::spin::OnceCell;
use lazy_static::lazy_static;
use spinning_top::Spinlock;

use vte::{Params, Parser, Perform};
use x86_64::instructions::interrupts;

const LINE_SPACING: usize = 2;
const LETTER_SPACING: usize = 0;
const BORDER_PADDING: usize = 1;

pub const FG: Color = Color::White;
pub const BG: Color = Color::Black;

mod font_constants {
    use super::*;

    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;
    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);
    pub const BACKUP_CHAR: char = '�';
    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;
}
lazy_static! {
    pub static ref PARSER: Spinlock<Parser> = Spinlock::new(Parser::new());
}
pub static FB_WRITER: OnceCell<Spinlock<FrameBufferWriter>> = OnceCell::uninit();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(
            c,
            font_constants::FONT_WEIGHT,
            font_constants::CHAR_RASTER_HEIGHT,
        )
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}

pub struct FrameBufferWriter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
    color_code: ColorCode,
    palette: Palette,
}

impl FrameBufferWriter {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut logger = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
            color_code: ColorCode::new(FG, BG),
            palette: Palette::default(),
        };
        logger.clear();
        logger
    }

    fn position(&self) -> (usize, usize) {
        (self.x_pos, self.y_pos)
    }

    fn set_position(&mut self, x: usize, y: usize) {
        self.x_pos = x;
        self.y_pos = y;
    }

    fn newline(&mut self) {
        self.y_pos += font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
        self.framebuffer.fill(0);
    }

    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }

    fn disable_echo(&self) {
        sys::console::disable_echo();
    }

    fn enable_echo(&self) {
        sys::console::enable_echo();
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            0x0A => {
                self.newline();
            }
            0x0D => {
                // Carriage Return
                self.carriage_return()
            }
            0x08 => {
                // Backspace
                self.x_pos -= 1;
                self.write_char(' ');
            }
            byte => {
                let ascii_code = if is_printable(byte) {
                    byte
                } else {
                    BACKUP_CHAR as u8
                };
                self.write_char(ascii_code as char);
            }
        }
    }

    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                let new_xpos = self.x_pos + font_constants::CHAR_RASTER_WIDTH;
                if new_xpos >= self.width() {
                    self.newline();
                }
                let new_ypos =
                    self.y_pos + font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.height() {
                    self.clear();
                }
                self.write_rendered_char(get_char_raster(c));
            }
        }
    }

    fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(self.x_pos + x, self.y_pos + y, *byte);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.info.stride + x;

        let color = match intensity {
            0..=10 => self.color().1,
            _ => self.color().0,
        };

        let color = self.palette.colors[color as usize];

        let color = match intensity {
            0 => [color.0, color.1, color.2, 0],
            _ => [
                (color.0 as u16 * intensity as u16 / 255) as u8,
                (color.1 as u16 * intensity as u16 / 255) as u8,
                (color.2 as u16 * intensity as u16 / 255) as u8,
                0,
            ],
        };

        let color = match self.info.pixel_format {
            PixelFormat::Rgb => color,
            PixelFormat::Bgr => [color[2], color[1], color[0], color[3]],
            PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
            other => {
                // set a supported (but invalid) pixel format before panicking to avoid a double
                // panic; it might not be readable though
                self.info.pixel_format = PixelFormat::Rgb;
                panic!("pixel format {:?} not supported in logger", other)
            }
        };

        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }

    pub fn set_color(&mut self, foreground: Color, background: Color) {
        self.color_code = ColorCode::new(foreground, background);
    }

    pub fn color(&self) -> (Color, Color) {
        let cc = self.color_code.0;
        let fg = color::from_index(cc.get_bits(0..4) as usize);
        let bg = color::from_index(cc.get_bits(4..8) as usize);
        (fg, bg)
    }

    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
    }
}

/// See https://vt100.net/emu/dec_ansi_parser
impl Perform for FrameBufferWriter {
    fn print(&mut self, c: char) {
        self.write_char(c);
    }

    fn execute(&mut self, byte: u8) {
        self.write_byte(byte);
    }

    fn csi_dispatch(&mut self, params: &Params, _: &[u8], _: bool, c: char) {
        match c {
            'm' => {
                let mut fg = FG;
                let mut bg = BG;
                for param in params.iter() {
                    match param[0] {
                        0 => {
                            fg = FG;
                            bg = BG;
                        }
                        30..=37 | 90..=97 => {
                            fg = color::from_ansi(param[0] as u8);
                        }
                        40..=47 | 100..=107 => {
                            bg = color::from_ansi((param[0] as u8) - 10);
                        }
                        _ => {}
                    }
                }
                self.set_color(fg, bg);
            }
            'A' => {
                // Cursor Up
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                // TODO: Don't go past edge
                self.x_pos -= n;
            }
            'B' => {
                // Cursor Down
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                // TODO: Don't go past edge
                self.x_pos += n;
            }
            'C' => {
                // Cursor Forward
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                // TODO: Don't go past edge
                self.x_pos += n;
            }
            'D' => {
                // Cursor Backward
                let mut n = 1;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                // TODO: Don't go past edge
                self.x_pos -= n;
            }
            'G' => {
                // Cursor Horizontal Absolute
                let y = self.y_pos;
                let mut x = 1;
                for param in params.iter() {
                    x = param[0] as usize; // 1-indexed value
                }
                if x > self.width() {
                    return;
                }
                self.x_pos = x - 1;
                self.y_pos = y;
            }
            'H' => {
                // Move cursor
                let mut x = 1;
                let mut y = 1;
                for (i, param) in params.iter().enumerate() {
                    match i {
                        0 => y = param[0] as usize, // 1-indexed value
                        1 => x = param[0] as usize, // 1-indexed value
                        _ => break,
                    };
                }
                if x > self.width() || y > self.height() {
                    return;
                }
                self.x_pos = x - 1;
                self.y_pos = y - 1;
            }
            'J' => {
                // Erase in Display
                let mut n = 0;
                for param in params.iter() {
                    n = param[0] as usize;
                }
                match n {
                    // TODO: 0 and 1, from cursor to begining or to end of screen
                    2 => self.clear(),
                    _ => return,
                }
                self.x_pos = 0;
                self.y_pos = 0;
            }
            'K' => {
                // Erase in Line
                todo!();
                // let (x, y) = (self.x_pos, self.y_pos);
                // let mut n = 0;
                // for param in params.iter() {
                //     n = param[0] as usize;
                // }
                // match n {
                //     0 => self.clear_row_after(x, y),
                //     1 => return, // TODO: self.clear_row_before(x, y),
                //     2 => self.clear_row_after(0, y),
                //     _ => return,
                // }
                // self.set_writer_position(x, y);
                // self.set_cursor_position(x, y);
            }
            'h' => {
                // Enable
                for param in params.iter() {
                    match param[0] {
                        12 => self.enable_echo(),
                        _ => return,
                    }
                }
            }
            'l' => {
                // Disable
                for param in params.iter() {
                    match param[0] {
                        12 => self.disable_echo(),
                        _ => return,
                    }
                }
            }
            _ => {}
        }
    }
}

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut parser = PARSER.lock();
        for byte in s.bytes() {
            parser.advance(self, byte);
        }
        let (x, y) = self.position();
        self.set_position(x, y);

        Ok(())
    }
}

#[doc(hidden)]
pub fn print_fmt(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        FB_WRITER
            .get()
            .unwrap()
            .lock()
            .write_fmt(args)
            .expect("Could not print to VGA");
    });
}

pub fn cols() -> usize {
    interrupts::without_interrupts(|| FB_WRITER.get().unwrap().lock().width())
}

pub fn rows() -> usize {
    interrupts::without_interrupts(|| FB_WRITER.get().unwrap().lock().height())
}

pub fn color() -> (Color, Color) {
    interrupts::without_interrupts(|| FB_WRITER.get().unwrap().lock().color())
}

pub fn set_color(foreground: Color, background: Color) {
    interrupts::without_interrupts(|| {
        FB_WRITER
            .get()
            .unwrap()
            .lock()
            .set_color(foreground, background)
    })
}

// ASCII Printable
// Backspace
// New Line
// Carriage Return
// Extended ASCII Printable
pub fn is_printable(c: u8) -> bool {
    matches!(c, 0x20..=0x7E | 0x08 | 0x0A | 0x0D | 0x7F..=0xFF)
}

pub fn set_palette(palette: Palette) {
    interrupts::without_interrupts(|| FB_WRITER.get().unwrap().lock().set_palette(palette))
}

pub fn init(framebuffer: &'static mut [u8], info: FrameBufferInfo) {
    FB_WRITER.get_or_init(|| Spinlock::new(FrameBufferWriter::new(framebuffer, info)));
}
