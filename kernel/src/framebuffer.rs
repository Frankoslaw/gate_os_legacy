use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

#[allow(dead_code)]
pub struct Display<'a> {
    pub fb: limine::framebuffer::Framebuffer<'a>,
    pub x_pos: usize,
    pub y_pos: usize,
}

impl OriginDimensions for Display<'_> {
    fn size(&self) -> Size {
        Size::new(self.fb.width() as u32, self.fb.height() as u32)
    }
}

impl DrawTarget for Display<'_> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x < 0 || coord.x > self.fb.width() as i32 {
                break;
            }

            if coord.y < 0 || coord.x > self.fb.height() as i32 {
                break;
            }

            // Calculate the index in the framebuffer.
            let pixel_offset: u32 = coord.y as u32 * self.fb.pitch() as u32
                + coord.x as u32 * (self.fb.bpp() as u32 / 8);

            unsafe {
                *(self.fb.addr().add(pixel_offset as usize) as *mut u32) = (color.r() as u32)
                    << self.fb.red_mask_shift()
                    | (color.g() as u32) << self.fb.blue_mask_shift()
                    | (color.b() as u32) << self.fb.green_mask_shift();
            }
        }

        Ok(())
    }
}

unsafe impl Send for Display<'_> {}
