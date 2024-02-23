use std::thread;
use std::sync::{Mutex, Arc};
use std::time::Duration;


pub struct Timer {
    delay: Mutex<u8>,
}

impl Timer {
    pub fn new(for_test: bool) -> Arc<Timer> {
        let timer = Arc::new(Timer {
            delay: Mutex::new(0),
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

    fn one_iteration(delay: &Mutex<u8>) {
        let mut delay = delay.lock().unwrap();
        if *delay > 0 {
            *delay -= 1;
        }
    }

    fn thread_loop(timer: Arc<Timer>) {
        loop {
            Timer::one_iteration(&timer.delay);
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
        Timer::one_iteration(&timer.delay);
        assert_eq!(Timer::get_delay(&timer), 0x5);
        Timer::one_iteration(&timer.delay);
        assert_eq!(Timer::get_delay(&timer), 0x4);
    }
}