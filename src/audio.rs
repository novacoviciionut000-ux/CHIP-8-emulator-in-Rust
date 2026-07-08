use rodio::{source::SineWave, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const TONE_FREQUENCY_HZ: f32 = 440.0;
const BEEP_VOLUME: f32 = 0.2;
const MUTE_VOLUME: f32 = 0.0;

pub struct AudioHandler {
    _stream: MixerDeviceSink,
    player: Player,
    should_beep: Arc<AtomicBool>,
}

impl AudioHandler {
    pub fn new(should_beep: Arc<AtomicBool>) -> Self {
        let stream = DeviceSinkBuilder::open_default_sink()
            .expect("Could not initialize audio device");
        let player = Player::connect_new(stream.mixer());
        let source = SineWave::new(TONE_FREQUENCY_HZ).repeat_infinite();
        player.append(source);
        player.set_volume(MUTE_VOLUME);

        Self {
            _stream: stream,
            player,
            should_beep,
        }
    }

    /// Reads the flag the emulation thread publishes at the same moment
    /// it decrements sound_timer, instead of re-reading sound_timer from
    /// a separately-locked Cpu snapshot on the render thread.
    pub fn update(&self) {
        
        let volume = if self.should_beep.load(Ordering::Relaxed) {
            BEEP_VOLUME
        } else {
            MUTE_VOLUME
        };
        self.player.set_volume(volume);
    }
}