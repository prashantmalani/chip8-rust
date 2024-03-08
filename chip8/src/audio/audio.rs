use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

// Copied as is from the sdl2::audio documentation:
// https://github.com/Rust-SDL2/rust-sdl2/blob/master/examples/audio-squarewave.rs
pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct Audio {
    device: AudioDevice<SquareWave>,
}

unsafe impl Sync for Audio {}
unsafe impl Send for Audio {}

impl Audio {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let audio_subsystem = sdl_context.audio().unwrap();

        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                // initialize the audio callback
                SquareWave {
                    phase_inc: 440.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.25,
                }
            })
            .unwrap();

        return Audio {
          device: device,
        };
    }

    pub fn start(&self) {
        self.device.resume();
    }

    pub fn stop(&self) {
        self.device.pause();
    }
}