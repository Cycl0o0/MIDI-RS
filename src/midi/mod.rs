// src/midi/mod.rs

pub mod parser;
pub mod player;
pub mod note;

pub use parser::MidiParser;
pub use player::MidiPlayer;
pub use note::Note;