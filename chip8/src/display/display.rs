const WIDTH: usize = 64;
const HEIGHT: usize = 32;

// We implement the display using a linear vector of 32 bit values.
pub struct Display {
    buf: [u32; WIDTH * HEIGHT],
}

impl Display {

    pub fn new() -> Self {
        Display {
            buf: [1; WIDTH * HEIGHT],
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

    // Right now, just print everything to the console.
    // We keep the actual printing separate from the update logic so that
    // it is easier to write unit tests for the logic.
    //
    // TODO: print to an actual framebuffer.
    fn update_display(&self) {
        for i in 0..self.buf.len() {
            if i % WIDTH == 0 {
                println!();
            }
            std::print!("{}", self.buf[i])
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::{Display, WIDTH, HEIGHT};

    #[test]
    fn check_clear_buf() {
        let mut disp = Display{buf: [1; WIDTH * HEIGHT]};
        disp.clear_buf();
        for pxl in disp.buf.iter() {
            assert_eq!(*pxl, 0);
        }
    }
}