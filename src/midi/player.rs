// src/midi/player.rs

pub struct MidiPlayer {
    current_time: f32,
    is_playing: bool,
    playback_speed: f32,
}

impl MidiPlayer {
    pub fn new() -> Self {
        MidiPlayer {
            current_time: 0.0,
            is_playing: false,
            playback_speed: 1.0,
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn toggle_playback(&mut self) {
        self.is_playing = !self.is_playing;
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn get_current_time(&self) -> f32 {
        self.current_time
    }

    pub fn get_playback_speed(&self) -> f32 {
        self.playback_speed
    }

    pub fn set_playback_speed(&mut self, speed: f32) {
        self.playback_speed = speed.clamp(0.5, 2.0);
    }

    pub fn increase_speed(&mut self) {
        self.set_playback_speed(self.playback_speed + 0.1);
    }

    pub fn decrease_speed(&mut self) {
        self.set_playback_speed(self.playback_speed - 0.1);
    }

    pub fn seek(&mut self, time: f32) {
        self.current_time = time.max(0.0);
    }

    pub fn reset(&mut self) {
        self.current_time = 0.0;
        self.is_playing = false;
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.is_playing {
            self.current_time += delta_time * self.playback_speed;
        }
    }
}

impl Default for MidiPlayer {
    fn default() -> Self {
        Self::new()
    }
}