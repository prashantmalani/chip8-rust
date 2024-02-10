use minifb::{Window, WindowOptions, Scale};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

const ON_PIXEL: u32 = 0xFFFFFF;
const OFF_PIXEL: u32 = 0x0;

// We implement the display using a linear vector of 32 bit values.
pub struct Display {
    buf: [u32; WIDTH * HEIGHT],
    window: Option<Window>,
}

impl Display {
    pub fn new() -> Self {
        let mut woptions  = WindowOptions::default();
        woptions.scale = Scale::X8;
        Display {
            buf: [1; WIDTH * HEIGHT],
            window: Some(Window::new(
                "Test",
                WIDTH,
                HEIGHT,
                woptions,
            ).unwrap_or_else(|e| {
                panic!("{}", e);
            })),
        }
    }

    pub fn clear(&mut self) {
        self.clear_buf();
        self.update_display();
    }

    fn clear_buf(&mut self) {
        for pxl in self.buf.iter_mut() {
            *pxl = 0;
        }
    }

    pub fn draw(&mut self, x: u8, y: u8, sprite: &Vec<u8>) -> u8 {
        let vf = self.update_buf_sprite(x, y, sprite);
        self.update_display();

        return vf;
    }

    // Performs the draw of the sprite, and returns
    // what the eventual value of F register should be.
    fn update_buf_sprite(&mut self, x: u8, y:u8, sprite: &Vec<u8>) -> u8 {
        let mut vf: u8 = 0;
        for (i, cur_byte) in sprite.iter().enumerate() {
            // Stop if you've reach the vertical edge.
            let cur_y = y + (i as u8);
            if cur_y == (HEIGHT as u8) {
                break;
            }

            for x_ind in {0..8} {
                let cur_x = x + x_ind;
                // Stop if we've reached the edge.
                if cur_x == (WIDTH as u8) {
                    break;
                }

                let bit = (cur_byte >> (7 - x_ind)) & 1;
                if bit == 0 {
                    continue;
                }

                let buf_ind: usize = (WIDTH * cur_y as usize) + cur_x as usize;
                if self.buf[buf_ind] == ON_PIXEL {
                    self.buf[buf_ind] = OFF_PIXEL;
                    vf = 1;
                } else {
                    self.buf[buf_ind] = ON_PIXEL;
                }
            }
        }

        return vf;
     }

    // Right now, just print everything to the console.
    // We keep the actual printing separate from the update logic so that
    // it is easier to write unit tests for the logic.
    //
    // TODO: print to an actual framebuffer.
    fn update_display(&mut self) {
        if let Some(window) = &mut self.window {
            window.update_with_buffer(&self.buf, WIDTH, HEIGHT)
                .expect("Window update failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Display, WIDTH, HEIGHT, ON_PIXEL, OFF_PIXEL};

    #[test]
    fn check_clear_buf() {
        let mut disp = Display{buf: [1; WIDTH * HEIGHT], window: None};
        disp.clear_buf();
        for pxl in disp.buf.iter() {
            assert_eq!(*pxl, 0);
        }
    }

    #[test]
    fn update_buf_sprite_normal() {
        let mut disp = Display{buf: [OFF_PIXEL; WIDTH * HEIGHT], window: None};
        // Use a sprite for the letter "F"
        let sprite = vec![0xF0, 0x80, 0xF0, 0x80, 0x80];

        let x = 32;
        let y = 16;
        let vf = disp.update_buf_sprite(x, y, &sprite);
        assert_eq!(vf, 0);

        // Check the buffer pixel values are equal to the sprite.
        // This is the normal case so we don't care about overflow.
        for (j, byte) in sprite.iter().enumerate() {
            let cur_y = y as usize + j;
            for i in 0..8 {
                let bit = (byte >> (7 - i)) & 1;
                let buf_ind: usize = (WIDTH * cur_y) + (x + i) as usize;
                if bit == 1 {
                    assert_eq!(disp.buf[buf_ind], ON_PIXEL);
                } else {
                    assert_eq!(disp.buf[buf_ind], OFF_PIXEL);
                }
            }
        }
    }

    #[test]
    // Test the sprite doesn't wrap around.
    fn update_buf_edge() {
        let mut disp = Display{buf: [OFF_PIXEL; WIDTH * HEIGHT], window: None};
        // Use a sprite for the letter "F"
        let sprite = vec![0xF0, 0x80, 0xF0, 0x80, 0x80];

        let x = 60;
        let y = 29;
        let vf = disp.update_buf_sprite(x, y, &sprite);
        assert_eq!(vf, 0);

        // First check that the edge *is* filled
        for (j, byte) in sprite[..(HEIGHT-y as usize)].iter().enumerate() {
            let cur_y = y as usize + j;
            for i in 0..(WIDTH-x as usize) {
                let bit = (byte >> (7 - i)) & 1;
                let buf_ind: usize = (WIDTH * cur_y) + (x as usize + i);
                if bit == 1 {
                    assert_eq!(disp.buf[buf_ind], ON_PIXEL);
                } else {
                    assert_eq!(disp.buf[buf_ind], OFF_PIXEL);
                }
            }
        }

        // Now check that the rest i.e the wrapped around part isn't drawn.
        for (j, _) in sprite[(HEIGHT-y as usize)..].iter().enumerate() {
            let cur_y = y as usize + j;
            for i in (WIDTH-x as usize)..8 {
                let buf_ind: usize = (WIDTH * cur_y) + (x as usize + i);
                assert_eq!(disp.buf[buf_ind], OFF_PIXEL);
            }
        }

    }

    #[test]
    // Case where already on pixels are switched off by the sprite.
    fn update_buf_sprite_vf_check() {
        let mut disp = Display{buf: [OFF_PIXEL; WIDTH * HEIGHT], window: None};
        // Use a sprite for the letter "F"
        let sprite = vec![0xF0, 0x80, 0xF0, 0x80, 0x80];

        let x = 32;
        let y = 16;

        // Set the display buffer as if the sprite has already been drawn.
        for (j, byte) in sprite.iter().enumerate() {
            let cur_y = y as usize + j;
            for i in 0..8 {
                let bit = (byte >> (7 - i)) & 1;
                let buf_ind: usize = (WIDTH * cur_y) + (x + i) as usize;
                disp.buf[buf_ind] = if bit == 1 { ON_PIXEL } else { OFF_PIXEL };
            }
        }

        let vf = disp.update_buf_sprite(x, y, &sprite);
        assert_eq!(vf, 1);

        // All the pixels should be switched off.
        for (j, byte) in sprite.iter().enumerate() {
            let cur_y = y as usize + j;
            for i in 0..8 {
                let buf_ind: usize = (WIDTH * cur_y) + (x + 1) as usize;
                assert_eq!(disp.buf[buf_ind], OFF_PIXEL)
            }
        }
    }
}