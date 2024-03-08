use std::thread;
use std::sync::{Mutex, Arc};
use std::time::Duration;

use crate::audio::audio::Audio;

pub struct Timer {
    delay: Mutex<u8>,
    sound: Mutex<u8>,
    audio: Option<Mutex<Audio>>,
}

impl Timer {
    pub fn new(for_test: bool) -> Arc<Timer> {
        let timer = Arc::new(Timer {
            delay: Mutex::new(0),
            sound: Mutex::new(0),
            audio: if !for_test {
                Some(Mutex::new(Audio::new()))
            } else {
                None
            }
        });

        if !for_test {
            let timer_clone = Arc::clone(&timer);
            thread::spawn(move || {
                Timer::thread_loop(timer_clone);
            });
        }

        return timer;
    }

    pub fn set_delay(timer: &Arc<Timer>, val: u8) {
        let mut delay = timer.delay.lock().unwrap();
        *delay = val;
    }

    pub fn get_delay(timer: &Arc<Timer>) -> u8{
        let delay = timer.delay.lock().unwrap();
        return *delay;
    }

    pub fn set_sound(timer: &Arc<Timer>, val: u8) {
        let mut sound = timer.sound.lock().unwrap();
        *sound = val;
    }

    pub fn get_sound(timer: &Arc<Timer>) -> u8{
        let sound = timer.sound.lock().unwrap();
        return *sound;
    }

    fn one_iteration(delay: &Mutex<u8>, sound: &Mutex<u8>, audio: &Option<Mutex<Audio>>) {
        let mut delay = delay.lock().unwrap();
        if *delay > 0 {
            *delay -= 1;
        }

        let mut sound = sound.lock().unwrap();
        if *sound > 0 {
            *sound -= 1;
        }

        if audio.is_some() {
            if (*sound > 0) {
                audio.as_ref().unwrap().lock().unwrap().start();
            } else {
                audio.as_ref().unwrap().lock().unwrap().stop();
            }
        }
    }

    fn thread_loop(timer: Arc<Timer>) {
        loop {
            Timer::one_iteration(&timer.delay, &timer.sound, &timer.audio);
            thread::sleep(Duration::from_micros(16666));
        }
    }
}


mod tests {
    use super::Timer;

    #[test]
    // Since we can't run the timer thread and meaningfully verify the code in a unit
    // test, create a version of the Timer which doens't have a thread running
    // and fake the passage of time by manually calling one_iteration().
    fn check_iterations() {
        let timer = Timer::new(true);
        Timer::set_delay(&timer, 0x6);
        Timer::one_iteration(&timer.delay, &timer.sound, &None);
        assert_eq!(Timer::get_delay(&timer), 0x5);
        Timer::one_iteration(&timer.delay, &timer.sound, &None);
        assert_eq!(Timer::get_delay(&timer), 0x4);
    }
}