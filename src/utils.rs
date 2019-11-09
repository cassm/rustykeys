pub mod music {
    use super::constants::{NOTE_NAMES, MIDI_START_INDEX};

    pub fn get_octave(key_index: u8) -> Option<u8> {
        let note_index = key_index - MIDI_START_INDEX;

        Some(note_index / NOTE_NAMES.len() as u8)
    }

    pub fn get_note_name(key_index: u8) -> String {
        let note_index = key_index - MIDI_START_INDEX;

        NOTE_NAMES[note_index as usize % NOTE_NAMES.len()][0].to_string()
    }

    pub fn note_matches(key_index: u8, note: &str) -> bool {
        let note_index = key_index - MIDI_START_INDEX;

        return NOTE_NAMES[note_index as usize % NOTE_NAMES.len()].contains(&note);
    }
}

pub mod mutex {
    use std::sync::Mutex;
    use std::time::Instant;

    lazy_static! {
        pub static ref KEYS_DOWN: Mutex<Vec<u8>> = Mutex::new(vec![]);
        pub static ref LAST_KEY_PRESS: Mutex<Option<Instant>> = Mutex::new(None);
    }
}

pub mod constants {
    pub const DEBOUNCE_MILLIS: u64 = 100;
    pub const MIDI_START_INDEX: u8 = 24;


    pub const NOTE_NAMES: &'static [&'static [&'static str]] = &[
        &["C"],
        &["C#", "Db"],
        &["D"],
        &["D#", "Eb"],
        &["E"],
        &["F"],
        &["F#", "Gb"],
        &["G"],
        &["G#", "Ab"],
        &["A"],
        &["A#", "Bb"],
        &["B"]];

    pub const MAJOR_SCALE_INTERVALS: &'static [usize] = &[2, 2, 1, 2, 2, 2, 1];
}

pub mod types {
    use std::fmt;
    use ordinal::Ordinal;

    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum Hand {
        Left,
        Right,
        Both,
    }

    impl Hand {
        pub fn value(&self) -> Vec<u8> {
           match *self {
               Hand::Left => vec![0, 1, 2],
               Hand::Right => vec![3, 4, 5],
               Hand::Both => vec![0, 1, 2, 3, 4, 5],
           }
        }
    }

    impl fmt::Display for Hand {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match &self {
                Hand::Left => write!(f, "Left Hand"),
                Hand::Right => write!(f, "Right Hand"),
                Hand::Both => write!(f, "Both Hands"),
            }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum ChordType {
        Major,
        Minor,
        Diminished,
        MajorSeventh,
        MinorSeventh,
        DominantSeventh,
        Augmented,
        SusTwo,
        SusFour,
        SevenSusTwo,
        SevenSusFour,
        SusSix,
    }

    impl ChordType {
        pub fn value(&self) -> String {
           match *self {
               ChordType::Major => "".to_string(),
               ChordType::Minor => "m".to_string(),
               ChordType::Diminished => "dim".to_string(),
               ChordType::MajorSeventh => "maj7".to_string(),
               ChordType::MinorSeventh => "min7".to_string(),
               ChordType::DominantSeventh => "dom7".to_string(),
               ChordType::Augmented => "aug".to_string(),
               ChordType::SusTwo => "sus2".to_string(),
               ChordType::SusFour => "sus4".to_string(),
               ChordType::SevenSusTwo => "7sus2".to_string(),
               ChordType::SevenSusFour => "7sus4".to_string(),
               ChordType::SusSix => "sus6".to_string(),
            }
        }
    }

    #[derive(Debug)]
    pub struct Chord {
        pub root: String,
        pub chord_type: ChordType,
        pub inversion: usize,
        pub octave: Option<u8>,
    }

    impl fmt::Display for Chord {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let inversion_str = match self.inversion {
                0 => String::from(""),
                _ => format!(", {} inversion", Ordinal(self.inversion)),
            };

            let octave_str = match self.octave {
                Some(i) => format!("({})", i),
                _ => String::from("")
            };

            write!(f, "{}{}{}{}", self.root, self.chord_type.value(), inversion_str, octave_str)
        }
    }

    impl PartialEq for Chord {
        fn eq(&self, other: &Self) -> bool {
            if self.root != other.root
                || self.chord_type != other.chord_type
                || self.inversion != other.inversion
                || (self.octave != None && other.octave != None && self.octave != other.octave) {
                return false;
            }

            return true;
        }
    }
}
