use minifb::{Window, WindowOptions, Scale};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

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
    use super::{Display, WIDTH, HEIGHT};

    #[test]
    fn check_clear_buf() {
        let mut disp = Display{buf: [1; WIDTH * HEIGHT], window: None};
        disp.clear_buf();
        for pxl in disp.buf.iter() {
            assert_eq!(*pxl, 0);
        }
    }
}