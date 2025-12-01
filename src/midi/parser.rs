use midly::{self, MidiMessage, Smf};
use std::fs::File;
use std::io::{self, BufReader};

pub struct MidiParser;

impl MidiParser {
    pub fn new() -> Self {
        MidiParser
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<Note>, io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let smf = Smf::load(reader)?;
        let mut notes = Vec::new();

        for track in smf.tracks {
            for event in track {
                if let MidiMessage::NoteOn { key, .. } = event.message {
                    notes.push(Note::new(key));
                }
            }
        }

        Ok(notes)
    }
}

pub struct Note {
    // Define the Note structure here.
}

impl Note {
    pub fn new(key: u8) -> Self {
        Note {
            // Initialize Note properties here.
        }
    }
}