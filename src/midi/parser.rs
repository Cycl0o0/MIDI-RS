// src/midi/parser.rs

use crate::midi::note::Note;
use midly::{MidiMessage, Smf, TrackEventKind};
use std::collections::HashMap;
use std::fs;
use std::io;

/// Error types for MIDI parsing
#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    MidiError(String),
    InvalidFile(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(e) => write!(f, "IO error: {}", e),
            ParseError::MidiError(e) => write!(f, "MIDI error: {}", e),
            ParseError::InvalidFile(e) => write!(f, "Invalid file: {}", e),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> Self {
        ParseError::IoError(error)
    }
}

impl From<midly::Error> for ParseError {
    fn from(error: midly::Error) -> Self {
        ParseError::MidiError(error.to_string())
    }
}

/// MIDI file parser with support for Black MIDI files
pub struct MidiParser {
    /// Minimum note duration in seconds (to filter out zero-length notes)
    min_note_duration: f32,
}

impl MidiParser {
    pub fn new() -> Self {
        MidiParser {
            min_note_duration: 0.001, // 1ms minimum
        }
    }

    /// Set minimum note duration filter
    pub fn with_min_duration(mut self, duration: f32) -> Self {
        self.min_note_duration = duration;
        self
    }

    /// Parse a MIDI file and return all notes with proper timing
    pub fn parse_file(&self, path: &str) -> Result<Vec<Note>, ParseError> {
        log::info!("Parsing MIDI file: {}", path);
        
        let data = fs::read(path)?;
        self.parse_bytes(&data)
    }

    /// Parse MIDI data from bytes
    pub fn parse_bytes(&self, data: &[u8]) -> Result<Vec<Note>, ParseError> {
        let smf = Smf::parse(data)?;
        
        // Get ticks per beat from the header
        let ticks_per_beat = match smf.header.timing {
            midly::Timing::Metrical(tpb) => tpb.as_int() as f64,
            midly::Timing::Timecode(fps, subframe) => {
                // For SMPTE timing, approximate
                (fps.as_f32() * subframe as f32) as f64
            }
        };

        log::info!("Ticks per beat: {}", ticks_per_beat);
        log::info!("Number of tracks: {}", smf.tracks.len());

        let mut all_notes = Vec::new();
        
        // Default tempo: 120 BPM = 500,000 microseconds per beat
        let default_tempo = 500_000.0;

        for (track_idx, track) in smf.tracks.iter().enumerate() {
            let mut notes = self.parse_track(track, ticks_per_beat, default_tempo)?;
            log::debug!("Track {} has {} notes", track_idx, notes.len());
            all_notes.append(&mut notes);
        }

        // Sort notes by start time for efficient rendering
        all_notes.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());

        log::info!("Total notes parsed: {}", all_notes.len());
        Ok(all_notes)
    }

    /// Parse a single MIDI track
    fn parse_track(
        &self,
        track: &[midly::TrackEvent],
        ticks_per_beat: f64,
        default_tempo: f64,
    ) -> Result<Vec<Note>, ParseError> {
        let mut notes = Vec::new();
        
        // Track active notes: (pitch, channel) -> (start_tick, velocity)
        let mut active_notes: HashMap<(u8, u8), (u64, u8)> = HashMap::new();
        
        // First pass: collect tempo changes
        let mut tempo_map: Vec<(u64, f64)> = vec![(0, default_tempo)];
        let mut tick = 0u64;
        
        for event in track {
            tick += event.delta.as_int() as u64;
            if let TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo)) = event.kind {
                tempo_map.push((tick, tempo.as_int() as f64));
            }
        }

        // Helper function to convert ticks to seconds using tempo map
        let ticks_to_seconds = |tick: u64| -> f32 {
            let mut seconds = 0.0;
            let mut last_tick = 0u64;
            let mut last_tempo = default_tempo;
            
            for &(tempo_tick, tempo) in &tempo_map {
                if tempo_tick > tick {
                    break;
                }
                // Add time from last_tick to tempo_tick at last_tempo
                let delta_ticks = tempo_tick - last_tick;
                seconds += (delta_ticks as f64 / ticks_per_beat) * (last_tempo / 1_000_000.0);
                last_tick = tempo_tick;
                last_tempo = tempo;
            }
            
            // Add remaining time from last_tick to tick at last_tempo
            let delta_ticks = tick - last_tick;
            seconds += (delta_ticks as f64 / ticks_per_beat) * (last_tempo / 1_000_000.0);
            
            seconds as f32
        };

        // Second pass: process note events
        let mut current_tick: u64 = 0;
        
        for event in track {
            current_tick += event.delta.as_int() as u64;
            
            match event.kind {
                TrackEventKind::Meta(midly::MetaMessage::Tempo(_)) => {
                    // Tempo changes are already processed in the tempo map
                }
                TrackEventKind::Midi { channel, message } => {
                    let channel = channel.as_int();
                    
                    match message {
                        MidiMessage::NoteOn { key, vel } => {
                            let pitch = key.as_int();
                            let velocity = vel.as_int();
                            
                            if velocity > 0 {
                                // Note on
                                active_notes.insert((pitch, channel), (current_tick, velocity));
                            } else {
                                // Note off (velocity 0)
                                if let Some((start_tick, vel)) = active_notes.remove(&(pitch, channel)) {
                                    let start_time = ticks_to_seconds(start_tick);
                                    let end_time = ticks_to_seconds(current_tick);
                                    let duration = end_time - start_time;
                                    
                                    if duration >= self.min_note_duration {
                                        notes.push(Note::new(pitch, vel, start_time, duration, channel));
                                    }
                                }
                            }
                        }
                        MidiMessage::NoteOff { key, .. } => {
                            let pitch = key.as_int();
                            
                            if let Some((start_tick, vel)) = active_notes.remove(&(pitch, channel)) {
                                let start_time = ticks_to_seconds(start_tick);
                                let end_time = ticks_to_seconds(current_tick);
                                let duration = end_time - start_time;
                                
                                if duration >= self.min_note_duration {
                                    notes.push(Note::new(pitch, vel, start_time, duration, channel));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Handle notes that were never closed (give them a default duration)
        for ((pitch, channel), (start_tick, velocity)) in active_notes {
            let start_time = ticks_to_seconds(start_tick);
            let duration = 0.1; // Default 100ms for unclosed notes
            notes.push(Note::new(pitch, velocity, start_time, duration, channel));
        }

        Ok(notes)
    }

    /// Get the duration of the MIDI file in seconds
    pub fn get_duration(notes: &[Note]) -> f32 {
        notes
            .iter()
            .map(|n| n.end_time())
            .fold(0.0f32, |a, b| a.max(b))
    }
}

impl Default for MidiParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = MidiParser::new();
        assert_eq!(parser.min_note_duration, 0.001);
    }

    #[test]
    fn test_parser_with_duration() {
        let parser = MidiParser::new().with_min_duration(0.01);
        assert_eq!(parser.min_note_duration, 0.01);
    }
}