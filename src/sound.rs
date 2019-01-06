use sdl2::{
    self,
    audio::{AudioCallback, AudioDevice, AudioSpecDesired},
};

pub struct Sound {
    device: AudioDevice<SquareWave>
}

impl Sound {
    pub fn new(sdl_context: &sdl2::Sdl) -> Sound {
        let audio = sdl_context.audio().unwrap();

        let spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let device = audio.open_playback(None, &spec, |spec| {
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        }).unwrap();

        Sound {
            device
        }
    }

    pub fn play(&self) {
        self.device.resume();
    }

    pub fn stop(&self) {
        self.device.pause();
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = match self.phase {
                0.0...0.5 => self.volume,
                _ => -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
