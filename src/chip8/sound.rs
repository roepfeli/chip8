use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use sdl2::audio::{AudioCallback, AudioSpecDesired};

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
    sound_timer: Arc<AtomicU8>,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.sound_timer.load(Ordering::Relaxed) <= 0 {
                -self.volume
            } else if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct Sound {
    audio_device: sdl2::audio::AudioDevice<SquareWave>,
}

impl Sound {
    pub fn init(sdl_context: &sdl2::Sdl, sound_timer: Arc<AtomicU8>) -> Sound {
        let audio_subsystem = sdl_context
            .audio()
            .expect("ERROR: Could not initialize the audio-subsystem. Exiting...");
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };

        let device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.12,
                sound_timer: sound_timer,
            })
            .expect("ERROR: Could not create SDl2-AudioDevice. Exiting...");
        Sound {
            audio_device: device,
        }
    }

    pub fn start_sound_system(&self) {
        self.audio_device.resume();
    }

    pub fn stop_sound_system(&self) {
        self.audio_device.pause();
    }
}
