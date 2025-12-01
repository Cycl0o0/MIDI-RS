// src/midi/note.rs

use bytemuck::{Pod, Zeroable};

/// Represents a MIDI note with timing and channel information
#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub pitch: u8,
    pub velocity: u8,
    pub start_time: f32,
    pub duration: f32,
    pub channel: u8,
}

impl Note {
    /// Create a new note with the given parameters
    pub fn new(pitch: u8, velocity: u8, start_time: f32, duration: f32, channel: u8) -> Self {
        Note {
            pitch,
            velocity,
            start_time,
            duration,
            channel,
        }
    }

    /// Calculate end time of the note
    pub fn end_time(&self) -> f32 {
        self.start_time + self.duration
    }

    /// Check if the note is visible at the given playback time within a time window
    /// For vertical rendering: notes are visible from above the playhead down to below it
    pub fn is_visible(&self, current_time: f32, time_window: f32) -> bool {
        // Notes visible from 15% below current time to 85% above current time
        let window_start = current_time - time_window * 0.15;
        let window_end = current_time + time_window * 0.85;
        self.start_time <= window_end && self.end_time() >= window_start
    }

    /// Get color as [r, g, b, a] based on the channel using HSV to RGB conversion
    pub fn get_color(&self) -> [f32; 4] {
        // Use channel to create distinct colors (16 channels -> 16 different hues)
        let hue = (self.channel as f32 / 16.0) * 360.0;
        let saturation = 0.8;
        let value = 0.5 + (self.velocity as f32 / 127.0) * 0.5; // Velocity affects brightness
        
        let (r, g, b) = hsv_to_rgb(hue, saturation, value);
        [r, g, b, 1.0]
    }

    /// Get x position for screen rendering (0.0 to 1.0 normalized based on pitch)
    /// Maps the pitch to a horizontal position like a piano keyboard
    pub fn get_x_position_from_pitch(&self) -> f32 {
        // MIDI pitch values range from 0-127 (128 values total)
        self.pitch as f32 / 128.0
    }

    /// Get y position for screen rendering based on current time
    /// Notes fall from top to bottom - playhead is near the bottom
    pub fn get_y_position_from_time(&self, current_time: f32, time_window: f32) -> f32 {
        // Notes fall from top (1.0) to bottom (0.0)
        // Playhead position is at the bottom (above the piano keyboard)
        let playhead_position = 0.15; // 15% from bottom (piano area is below)
        let time_offset = self.start_time - current_time;
        // Notes above the playhead are in the future, below are in the past
        playhead_position + (time_offset / time_window) * 0.85
    }

    /// Get the height of the note on screen (duration in vertical direction)
    pub fn get_height(&self, time_window: f32) -> f32 {
        (self.duration / time_window) * 0.85
    }

    /// Get the width of the note based on pitch (for piano-style rendering)
    pub fn get_width_from_pitch(&self) -> f32 {
        // Fixed width for each key - 88 keys on a piano (A0 to C8, MIDI 21-108)
        // But we support all 128 MIDI notes
        1.0 / 128.0
    }
}

/// Convert HSV color to RGB
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;
    
    let (r, g, b) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    (r + m, g + m, b + m)
}

/// GPU-friendly instance data for rendering notes
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct NoteInstance {
    pub position: [f32; 2],  // X, Y screen position
    pub size: [f32; 2],      // Width, Height
    pub color: [f32; 4],     // RGBA color
}

impl NoteInstance {
    /// Create a NoteInstance from a Note using vertical (top-to-bottom) rendering
    #[allow(dead_code)]
    pub fn from_note(note: &Note, current_time: f32, time_window: f32, _note_height: f32) -> Self {
        let x = note.get_x_position_from_pitch();
        let y = note.get_y_position_from_time(current_time, time_window);
        let width = note.get_width_from_pitch();
        let height = note.get_height(time_window);
        let color = note.get_color();
        
        NoteInstance {
            position: [x, y],
            size: [width, height],
            color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation() {
        let note = Note::new(60, 100, 1.0, 0.5, 0);
        assert_eq!(note.pitch, 60);
        assert_eq!(note.velocity, 100);
        assert_eq!(note.start_time, 1.0);
        assert_eq!(note.duration, 0.5);
        assert_eq!(note.channel, 0);
    }

    #[test]
    fn test_end_time() {
        let note = Note::new(60, 100, 1.0, 0.5, 0);
        assert_eq!(note.end_time(), 1.5);
    }

    #[test]
    fn test_visibility() {
        let note = Note::new(60, 100, 5.0, 1.0, 0);
        // Note starts at 5.0 and ends at 6.0
        
        // With current_time=5.0 and time_window=10.0:
        // window_start = 5.0 - 1.5 = 3.5 (15% below)
        // window_end = 5.0 + 8.5 = 13.5 (85% above)
        // Note (5.0-6.0) is within (3.5-13.5)
        assert!(note.is_visible(5.0, 10.0));
        
        // With current_time=0.0 and time_window=10.0:
        // window_start = 0.0 - 1.5 = -1.5
        // window_end = 0.0 + 8.5 = 8.5
        // Note (5.0-6.0) is within (-1.5-8.5)
        assert!(note.is_visible(0.0, 10.0));
        
        // With current_time=20.0 and time_window=10.0:
        // window_start = 20.0 - 1.5 = 18.5
        // window_end = 20.0 + 8.5 = 28.5
        // Note (5.0-6.0) is NOT within (18.5-28.5)
        assert!(!note.is_visible(20.0, 10.0));
    }
}