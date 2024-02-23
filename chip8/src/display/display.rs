use std::{sync::{Arc, Mutex, mpsc::{self, Sender}}, thread, time::Duration};

use show_image::{ImageView, ImageInfo, create_window, WindowProxy, event::KeyboardInput};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

const ON_PIXEL: u8 = 0xFF;
const OFF_PIXEL: u8 = 0x0;

// We implement the display using a linear vector of 32 bit values.
pub struct Display {
    buf: Mutex<[u8; WIDTH * HEIGHT]>,
    window: Option<Mutex<WindowProxy>>,
    tx: Mutex<Sender<()>>,
}

impl Display {
    pub fn new() -> Arc<Display> {
        let (tx, rx) = mpsc::channel();

        let mut disp = Arc::new(Display {
            buf: Mutex::new([1; WIDTH * HEIGHT]),
            window: Some(Mutex::new(create_window("image", Default::default()).unwrap_or_else(|e| {
                panic!("{}", e);
            }))),
            tx: Mutex::new(tx),
        });

        let tx_clone = Arc::clone(&disp).tx.lock().unwrap().clone(); // Clone Sender inside Mutex

        let disp_clone = Arc::clone(&disp); // Create a clone of the Arc

        thread::spawn(move || {
            Display::thread_loop(disp_clone, rx);
        });

        disp
    }

    fn thread_loop(disp: Arc<Display>, rx: mpsc::Receiver<()>) {
        loop {
            if let Some(window_mutex) = &disp.window {
                if let Ok(mut window_lock) = window_mutex.lock() {
                    if let window = &mut *window_lock {
                        if let Err(err) = window.set_image("image", ImageView::new(
                            ImageInfo::mono8(WIDTH as u32, HEIGHT as u32),
                            &*disp.buf.lock().unwrap(),
                        )) {
                            eprintln!("Failed to set image: {}", err);
                        }

                        for event in window.event_channel() {
                            match event.recv() {
                                Ok(wevent) => {
                                    match wevent {
                                        show_image::event::WindowEvent::KeyboardInput(kb_input) => {
                                            println!("Received KB event: {}, state: {:?}", kb_input.input.scan_code, kb_input.input.state);
                                        },
                                        _ => {},
                                    }
                                }
                                Err(e) => println!("Error receiving window event: {}", e),
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_micros(16666));
        }
    }

    pub fn clear(disp: &Arc<Display>) {
        Display::clear_buf(&disp.buf);
        let tx = disp.tx.lock().unwrap();
    }

    fn clear_buf(buf:&Mutex<[u8; WIDTH * HEIGHT]>) {
        let mut buf_unlocked = buf.lock().unwrap();
        for mut pxl in buf_unlocked.iter_mut() {
            *pxl = 0;
        }
    }

    pub fn draw(disp: &Arc<Display>, x: u8, y: u8, sprite: &Vec<u8>) -> u8 {
        let vf = Display::update_buf_sprite(&disp.buf, x, y, sprite);
        let tx = disp.tx.lock().unwrap();

        return vf;
    }

    // Performs the draw of the sprite, and returns
    // what the eventual value of F register should be.
    fn update_buf_sprite(buf: &Mutex<[u8; WIDTH * HEIGHT]>, x: u8, y:u8, sprite: &Vec<u8>) -> u8 {
        let mut vf: u8 = 0;
        let mut buf_unlocked = buf.lock().unwrap();
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
                if buf_unlocked[buf_ind] == ON_PIXEL {
                    buf_unlocked[buf_ind] = OFF_PIXEL;
                    vf = 1;
                } else {
                    buf_unlocked[buf_ind] = ON_PIXEL;
                }
            }
        }

        return vf;
     }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex, mpsc};

    use super::{Display, WIDTH, HEIGHT, ON_PIXEL, OFF_PIXEL};

    #[test]
    fn check_clear_buf() {
        let (tx, rx) = mpsc::channel();
        let mut disp = Display{buf: Mutex::new([1; WIDTH * HEIGHT]), window: None, tx: Mutex::new(tx)};
        let mut disp_arc = Arc::new(disp);
        Display::clear_buf(&disp_arc.buf);
        for pxl in disp_arc.buf.lock().unwrap().iter() {
            assert_eq!(*pxl, 0);
        }
    }

    #[test]
    fn update_buf_sprite_normal() {
        let (tx, rx) = mpsc::channel();
        let mut disp = Display{buf: Mutex::new([OFF_PIXEL; WIDTH * HEIGHT]), window: None, tx: Mutex::new(tx)};
        let mut disp_arc = Arc::new(disp);
        // Use a sprite for the letter "F"
        let sprite = vec![0xF0, 0x80, 0xF0, 0x80, 0x80];

        let x = 32;
        let y = 16;
        let vf = Display::update_buf_sprite(&disp_arc.buf, x, y, &sprite);
        assert_eq!(vf, 0);

        // Check the buffer pixel values are equal to the sprite.
        // This is the normal case so we don't care about overflow.
        for (j, byte) in sprite.iter().enumerate() {
            let cur_y = y as usize + j;
            for i in 0..8 {
                let bit = (byte >> (7 - i)) & 1;
                let buf_ind: usize = (WIDTH * cur_y) + (x + i) as usize;
                if bit == 1 {
                    assert_eq!(disp_arc.buf.lock().unwrap()[buf_ind], ON_PIXEL);
                } else {
                    assert_eq!(disp_arc.buf.lock().unwrap()[buf_ind], OFF_PIXEL);
                }
            }
        }
    }

    #[test]
    // Test the sprite doesn't wrap around.
    fn update_buf_edge() {
        let (tx, rx) = mpsc::channel();
        let mut disp = Display{buf: Mutex::new([OFF_PIXEL; WIDTH * HEIGHT]), window: None, tx: Mutex::new(tx)};
        let mut disp_arc = Arc::new(disp);
        // Use a sprite for the letter "F"
        let sprite = vec![0xF0, 0x80, 0xF0, 0x80, 0x80];

        let x = 60;
        let y = 29;
        let vf = Display::update_buf_sprite(&disp_arc.buf, x, y, &sprite);
        assert_eq!(vf, 0);

        // First check that the edge *is* filled
        for (j, byte) in sprite[..(HEIGHT-y as usize)].iter().enumerate() {
            let cur_y = y as usize + j;
            for i in 0..(WIDTH-x as usize) {
                let bit = (byte >> (7 - i)) & 1;
                let buf_ind: usize = (WIDTH * cur_y) + (x as usize + i);
                if bit == 1 {
                    assert_eq!(disp_arc.buf.lock().unwrap()[buf_ind], ON_PIXEL);
                } else {
                    assert_eq!(disp_arc.buf.lock().unwrap()[buf_ind], OFF_PIXEL);
                }
            }
        }

        // Now check that the rest i.e the wrapped around part isn't drawn.
        for (j, _) in sprite[(HEIGHT-y as usize)..].iter().enumerate() {
            let cur_y = y as usize + j;
            for i in (WIDTH-x as usize)..8 {
                let buf_ind: usize = (WIDTH * cur_y) + (x as usize + i);
                assert_eq!(disp_arc.buf.lock().unwrap()[buf_ind], OFF_PIXEL);
            }
        }

    }

    #[test]
    // Case where already on pixels are switched off by the sprite.
    fn update_buf_sprite_vf_check() {
        let (tx, rx) = mpsc::channel();
        let mut disp = Display{buf: Mutex::new([1; WIDTH * HEIGHT]), window: None, tx: Mutex::new(tx)};
        let mut disp_arc = Arc::new(disp);
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
                disp_arc.buf.lock().unwrap()[buf_ind] = if bit == 1 { ON_PIXEL } else { OFF_PIXEL };
            }
        }

        let vf = Display::update_buf_sprite(&disp_arc.buf, x, y, &sprite);
        assert_eq!(vf, 1);

        // All the pixels should be switched off.
        for (j, byte) in sprite.iter().enumerate() {
            let cur_y = y as usize + j;
            for i in 0..8 {
                let buf_ind: usize = (WIDTH * cur_y) + (x + 1) as usize;
                assert_eq!(disp_arc.buf.lock().unwrap()[buf_ind], OFF_PIXEL)
            }
        }
    }
}