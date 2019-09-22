#[macro_use]
extern crate lazy_static;
extern crate midir;
extern crate ordinal;
extern crate timer;
extern crate chrono;
extern crate rand;

use ordinal::Ordinal;
use std::sync::Mutex;
use std::io::{stdin, stdout, Write};
use std::error::Error;
use std::fmt;
use std::time::Instant;

use midir::{MidiInput, Ignore};
use rand::{thread_rng, seq::SliceRandom};

const NOTE_NAMES: &'static [&'static str] = &["A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#"];

lazy_static! {
    static ref KEYS_DOWN: Mutex<Vec<u8>> = Mutex::new(vec![]);
    static ref LAST_KEY_PRESS: Mutex<Option<Instant>> = Mutex::new(None);
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err.description())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ChordType {
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
    fn value(&self) -> String {
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
struct Chord {
    root: String,
    chord_type: ChordType,
    inversion: usize,
    octave: Option<u8>,
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

fn run() -> Result<(), Box<dyn Error>> {
    let mut input = String::new();

    loop {
        println!("Enter command: (q)uit, (p)ractise chords");
        input.clear();
        match stdin().read_line(&mut input) {
            Ok(_) => {
                match input.to_lowercase().trim() {
                    "q" => {
                        println!("Closing connection");
                        break;
                    },
                    "p" => {
                        practice_chords_launcher();
                    },
                    _ => println!("Unknown command: {}", input.trim()),
                }
            },
            Err(error) => println!("error: {}", error),
        }
    }

    Ok(())
}

fn practice_chords_launcher() {
    let mut input = String::new();

    let mut chord_type: Option<ChordType> = None;
    let mut inversion: Option<usize> = None;

    while chord_type == None {
        println!("Enter type of chords to practice: (M)ajor, (m)inor, (d)iminished, (a)ugmented, (M)ajor (7)th, (m)inor (7)7th, (d)ominant (7)th, sus2, sus4, 7sus2, 7sus4, sus6, (q)uit");

        input.clear();
        match stdin().read_line(&mut input) {
            Ok(_) => {
                match input.trim() {
                    "q" => {
                        println!("Exiting.");
                        return;
                    },
                    "M" => {
                        chord_type = Some(ChordType::Major);
                    },
                    "m" => {
                        chord_type = Some(ChordType::Minor);
                    },
                    "d" => {
                        chord_type = Some(ChordType::Diminished);
                    },
                    "a" => {
                        chord_type = Some(ChordType::Augmented);
                    },
                    "M7" => {
                        chord_type = Some(ChordType::MajorSeventh);
                    },
                    "m7" => {
                        chord_type = Some(ChordType::MinorSeventh);
                    },
                    "d7" => {
                        chord_type = Some(ChordType::DominantSeventh);
                    },
                    "sus2" => {
                        chord_type = Some(ChordType::SusTwo);
                    },
                    "sus4" => {
                        chord_type = Some(ChordType::SusFour);
                    },
                    "7sus2" => {
                        chord_type = Some(ChordType::SevenSusTwo);
                    },
                    "7sus4" => {
                        chord_type = Some(ChordType::SevenSusFour);
                    },
                    "sus6" => {
                        chord_type = Some(ChordType::SusSix);
                    },
                    _ => println!("Unknown command: {}", input.trim()),
                }
            },
            Err(error) => println!("error: {}", error),
        }
    }

    while inversion == None {
        println!("Enter inversion: (0..2), (q)uit");

        input.clear();
        match stdin().read_line(&mut input) {
            Ok(_) => {
                match input.trim() {
                    "q" => {
                        println!("Exiting.");
                        return;
                    },
                    "0" | "1" | "2" => {
                        inversion = Some(input.trim().parse().unwrap());
                    },
                    _ => println!("Unknown command: {}", input.trim()),
                }
            },
            Err(error) => println!("error: {}", error),
        }
    }

    match chord_type {
        Some(i) => {
            match inversion {
                Some(j) => {
                    match practice_chords(i, j) {
                        Err(e) => println!("{}", e),
                        _ => {},
                    }
                },
                None => {
                    println!("inversion is undefined! this should not be possible.");
                }
            }
        },
        None => {
            println!("chord type is undefined! this should not be possible.");
        }
    }
}

fn get_in_port(midi_in: &MidiInput) -> Result<usize, Box<dyn Error>> {
    let in_port = match midi_in.port_count() {
        0 => return Err("no input port found".into()),
        1 => {
            println!("Choosing the only available input port: {}", midi_in.port_name(0).unwrap());
            0
        },
        _ => {
            println!("\nAvailable input ports:");
            for i in 0..midi_in.port_count() {
                println!("{}: {}", i, midi_in.port_name(i).unwrap())
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            input.trim().parse::<usize>()?
        }
    };

    Ok(in_port)
}

fn generate_chord_list(chord_type: ChordType, inversion: usize) -> Vec<Chord> {
    let mut rng = thread_rng();
    let mut chords: Vec<Chord> = NOTE_NAMES.iter().map(|x| Chord{root: x.to_string(), chord_type, inversion, octave: None}).collect();
    chords.shuffle(&mut rng);
    chords
}

fn practice_chords(chord_type: ChordType, inversion: usize) -> Result<(), Box<dyn Error>> {
    let mut chords = generate_chord_list(chord_type, inversion);

    println!("{:?}", chords);

    let mut midi_in = MidiInput::new("midir forwarding input")?;
    midi_in.ignore(Ignore::None);

    let in_port = match get_in_port(&midi_in) {
        Ok(i)  => i,
        Err(e) => return Err(e),
    };

    let in_port_name = midi_in.port_name(in_port)?;

    println!("\nOpening connection");

    let debounce_millis = 100;

    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |_, message, _| {
        *LAST_KEY_PRESS.lock().unwrap() = Some(Instant::now());
        process_msg(message);
    }, ())?;

    println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);

    println!("Play {}", chords[0]);

    while chords.len() > 0 {
        let last_key_press = *LAST_KEY_PRESS.lock().unwrap();

        match last_key_press {
            Some(i) => {
                if i.elapsed().as_millis() > debounce_millis {
                    *LAST_KEY_PRESS.lock().unwrap() = None;
                    if KEYS_DOWN.lock().unwrap().len() > 0 {
                        match identify_chord() {
                            Some(i) => {
                                println!("chord: {}", i);

                                if chords[0] == i {
                                    println!("Correct!");
                                    chords.remove(0);

                                    if chords.len() > 0 {
                                        println!("Play {}", chords[0]);
                                    }
                                }
                                else {
                                    println!("Try again: {}", chords[0]);
                                }
                            },
                            None => {},
                        }
                    }
                }
            },
            _ => {},
        }
    }

    Ok(())
}

fn get_octave(key_index: u8) -> Option<u8> {
    let midi_start_index = 21;

    let note_index = key_index - midi_start_index;

    Some(note_index / NOTE_NAMES.len() as u8)
}

fn get_note_name(key_index: u8) -> String {
    let midi_start_index = 21;

    let note_index = key_index - midi_start_index;

    NOTE_NAMES[note_index as usize % NOTE_NAMES.len()].to_string()
}

fn process_msg(msg: &[u8]) {
    match msg[0] {
        0x90 => {
            KEYS_DOWN.lock().unwrap().push(msg[1]);
        },
        0x80 => {
            let index = KEYS_DOWN.lock().unwrap().iter().position(|x| *x == msg[1]).unwrap();
            KEYS_DOWN.lock().unwrap().remove(index);
        },
        _ => {},
    }
}

fn identify_chord() -> Option<Chord> {
    let mut keys_down: Vec<u8> = vec!();

    for i in KEYS_DOWN.lock().unwrap().as_slice() {
        keys_down.push(*i);
    }

    keys_down.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for inversion in 0..keys_down.len() {
        let positions: Vec<u8> = keys_down.iter().map(|x| x - keys_down[0]).collect();
        let root = get_note_name(keys_down[0]);
        let octave = get_octave(keys_down[0]);

        let chord_type = match positions.as_slice() {
            [0, 4, 7] => Some(ChordType::Major),
            [0, 3, 7] => Some(ChordType::Minor),
            [0, 3, 6] => Some(ChordType::Diminished),
            [0, 4, 7, 11] => Some(ChordType::MajorSeventh),
            [0, 3, 7, 10] => Some(ChordType::MinorSeventh),
            [0, 4, 7, 10] => Some(ChordType::DominantSeventh),
            [0, 4, 8] => Some(ChordType::Augmented),
            [0, 2, 7] => Some(ChordType::SusTwo),
            [0, 5, 7] => Some(ChordType::SusFour),
            [0, 2, 7, 10] => Some(ChordType::SevenSusTwo),
            [0, 5, 7, 10] => Some(ChordType::SevenSusFour),
            [0, 4, 7, 9] => Some(ChordType::SusSix),
            _ => None,
        };

        match chord_type {
            Some(i) => return Some(Chord{root, chord_type: i, inversion, octave}),
            _ => {}
        }

        match keys_down.pop() {
            Some(i) => keys_down.insert(0, i - 12),
            _ => {}
        };
    }

    return None
}
