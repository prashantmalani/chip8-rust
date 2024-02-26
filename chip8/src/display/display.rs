use std::{sync::{Arc, Mutex}, thread, time::Duration, collections::HashMap};

use show_image::{ImageView, ImageInfo, create_window, WindowProxy, event::ElementState};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

const ON_PIXEL: u8 = 0xFF;
const OFF_PIXEL: u8 = 0x0;

const THREAD_LOOP_SLEEP_US: u64 = 16666;

// We implement the display using a linear vector of 32 bit values.
pub struct Display {
    buf: Mutex<[u8; WIDTH * HEIGHT]>,
    window: Option<Mutex<WindowProxy>>,
    // Maintain state whether the key is currently pressed or not.
    keys_state: Mutex<HashMap<u8, bool>>,
}

impl Display {
    pub fn new(for_test: bool) -> Arc<Display> {
        let disp = Arc::new(Display {
            buf: Mutex::new([OFF_PIXEL; WIDTH * HEIGHT]),
            window: if !for_test {
                    Some(Mutex::new(create_window("image", Default::default())
                                    .unwrap_or_else(|e| {
                    panic!("{}", e);})))
                } else {
                    None
                },
            keys_state: Mutex::new(HashMap::new()),
        });

        let disp_clone = Arc::clone(&disp); // Create a clone of the Arc

        if !for_test {
            thread::spawn(move || {
                Display::thread_loop(disp_clone);
            });
        }

        disp
    }

    fn scancode_to_key(scancode: u32) -> Result<u8, String> {
        match scancode {
            2 => return Ok(0x1),
            3 => return Ok(0x2),
            4 => return Ok(0x3),
            5 => return Ok(0xC),
            16 => return Ok(0x4),
            17 => return Ok(0x5),
            18 => return Ok(0x6),
            19 => return Ok(0xD),
            30 => return Ok(0x7),
            31 => return Ok(0x8),
            32 => return Ok(0x9),
            33 => return Ok(0xE),
            44 => return Ok(0xA),
            45 => return Ok(0x0),
            46 => return Ok(0xB),
            47 => return Ok(0xF),
            _ => return Err(format!("Invalid keypress: {}", scancode)),
        }
    }

    fn set_key_state(disp: &Arc<Display>, scan_code: u32, state: ElementState) -> Result<i32, String> {
        let key_code = Display::scancode_to_key(scan_code)?;

        let mut keys_state = disp.keys_state.lock().unwrap();
        match state {
            ElementState::Pressed => { keys_state.insert(key_code, true); },
            ElementState::Released => { keys_state.insert(key_code, false); },
        }

        return Ok(0);
    }

    pub fn get_key_state(disp: &Arc<Display>, key: u8) -> Result<bool, String> {
        if key > 0xF {
            return Err(format!("Invalid key provided: {}", key));
        } else {
            let keys_state = disp.keys_state.lock().unwrap();
            match keys_state.get(&key) {
                Some(val) => return Ok(*val),
                None => return Ok(false),
            }
        }
    }

    fn handle_window_events(disp: &Arc<Display>, window: &mut WindowProxy) {
        for event in window.event_channel() {
            match event.recv() {
                Ok(wevent) => {
                    match wevent {
                        show_image::event::WindowEvent::KeyboardInput(kb_input) => {
                            match Display::set_key_state(disp, kb_input.input.scan_code, kb_input.input.state) {
                                Err(e) => eprintln!("Set key state failed: {}", e),
                                _ => {},
                            }
                        },
                        show_image::event::WindowEvent::CloseRequested(_) => std::process::exit(0),
                        _ => {},
                    }
                }
                Err(e) => println!("Error receiving window event: {}", e),
            }
        }
    }

    fn thread_loop(disp: Arc<Display>) {
        loop {
            if let Some(window_mutex) = &disp.window {
                if let Ok(mut window_lock) = window_mutex.lock() {
                    let window = &mut *window_lock;
                    if let Err(err) = window.set_image("image", ImageView::new(
                        ImageInfo::mono8(WIDTH as u32, HEIGHT as u32),
                        &*disp.buf.lock().unwrap(),
                    )) {
                        eprintln!("Failed to set image: {}", err);
                    }

                    Display::handle_window_events(&disp, window);
                }
            }

            thread::sleep(Duration::from_micros(THREAD_LOOP_SLEEP_US));
        }
    }

    pub fn clear(disp: &Arc<Display>) {
        Display::clear_buf(&disp.buf);
    }

    fn clear_buf(buf:&Mutex<[u8; WIDTH * HEIGHT]>) {
        let mut buf_unlocked = buf.lock().unwrap();
        for pxl in buf_unlocked.iter_mut() {
            *pxl = 0;
        }
    }

    pub fn draw(disp: &Arc<Display>, x: u8, y: u8, sprite: &Vec<u8>) -> u8 {
        let vf = Display::update_buf_sprite(&disp.buf, x, y, sprite);

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

            for x_ind in 0..8 {
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
    use show_image::event::ElementState;

    use super::{Display, WIDTH, HEIGHT, ON_PIXEL, OFF_PIXEL};

    #[test]
    fn check_clear_buf() {
        let disp_arc = Display::new(true);
        Display::clear_buf(&disp_arc.buf);
        for pxl in disp_arc.buf.lock().unwrap().iter() {
            assert_eq!(*pxl, 0);
        }
    }

    #[test]
    fn update_buf_sprite_normal() {
        let disp_arc = Display::new(true);
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
        let disp_arc = Display::new(true);
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
        let disp_arc = Display::new(true);
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
        for (j, _) in sprite.iter().enumerate() {
            let cur_y = y as usize + j;
            for _ in 0..8 {
                let buf_ind: usize = (WIDTH * cur_y) + (x + 1) as usize;
                assert_eq!(disp_arc.buf.lock().unwrap()[buf_ind], OFF_PIXEL)
            }
        }
    }

    #[test]
    fn key_state() {
        let disp_arc = Display::new(true);

        // Press a key.
        assert!(Display::set_key_state(&disp_arc, 2, ElementState::Pressed).is_ok());
        assert_eq!(Display::get_key_state(&disp_arc, 1).unwrap(), true);

        // Gets pressed again.
        assert!(Display::set_key_state(&disp_arc, 2, ElementState::Pressed).is_ok());
        assert_eq!(Display::get_key_state(&disp_arc, 1).unwrap(), true);

        // Release the key.
        assert!(Display::set_key_state(&disp_arc, 2, ElementState::Released).is_ok());
        assert_eq!(Display::get_key_state(&disp_arc, 1).unwrap(), false);

        // Press a invalid key
        assert!(Display::set_key_state(&disp_arc, 93, ElementState::Pressed).is_err());

        // Press two keys consecutively without releasing them, then make sure the first
        // one is still reporting as pressed.
        assert!(Display::set_key_state(&disp_arc, 2, ElementState::Pressed).is_ok());
        assert!(Display::set_key_state(&disp_arc, 3, ElementState::Pressed).is_ok());
        assert_eq!(Display::get_key_state(&disp_arc, 2).unwrap(), true);
    }
}