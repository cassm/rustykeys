#[macro_use]
extern crate lazy_static;
extern crate midir;
extern crate ordinal;

use ordinal::Ordinal;
use std::sync::Mutex;
use std::io::{stdin, stdout, Write};
use std::error::Error;
use std::fmt;

use midir::{MidiInput, Ignore};

lazy_static! {
    static ref KEYS_DOWN: Mutex<Vec<u8>> = Mutex::new(vec![]);
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err.description())
    }
}

enum ChordType {
    Major,
    Minor,
    Diminished,
    MajorSeventh,
    MinorSeventh,
    DominantSeventh,
    Augmented,
}

impl fmt::Display for ChordType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
           ChordType::Major => write!(f, ""),
           ChordType::Minor => write!(f, "m"),
           ChordType::Diminished => write!(f, "dim"),
           ChordType::MajorSeventh => write!(f, "maj7"),
           ChordType::MinorSeventh => write!(f, "min7"),
           ChordType::DominantSeventh => write!(f, "dom7"),
           ChordType::Augmented => write!(f, "aug"),
       }
    }
}

struct Chord {
    root: String,
    chord_type: ChordType,
    inversion: usize,
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inversion_str = match self.inversion {
            0 => String::from(""),
            _ => format!(", {} inversion", Ordinal(self.inversion)),
        };

        write!(f, "{}{}{}", self.root, self.chord_type, inversion_str)
    }
}

fn run() -> Result<(), Box<Error>> {
    let mut input = String::new();

    let mut midi_in = MidiInput::new("midir forwarding input")?;
    midi_in.ignore(Ignore::None);

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

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |stamp, message, _| {
        process_msg(message);
    }, ())?;

    println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}

fn get_note_name(key_index: u8) -> String {
    let note_names = ["A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#"];
    let midi_start_index = 21;

    let note_index = key_index - midi_start_index;

    format!("{}{}", note_names[note_index as usize % note_names.len()], note_index / note_names.len() as u8)
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

    if KEYS_DOWN.lock().unwrap().len() > 0 {
        match identify_chord() {
            Some(i) => println!("chord: {}", i),
            None => {},
        }
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

        match positions.as_slice() {
            [0, 4, 7] => return Some(Chord{root, chord_type: ChordType::Major, inversion}),
            [0, 3, 7] => return Some(Chord{root, chord_type: ChordType::Minor, inversion}),
            [0, 3, 6] => return Some(Chord{root, chord_type: ChordType::Diminished, inversion}),
            [0, 4, 7, 11] => return Some(Chord{root, chord_type: ChordType::MajorSeventh, inversion}),
            [0, 3, 7, 10] => return Some(Chord{root, chord_type: ChordType::MinorSeventh, inversion}),
            [0, 4, 7, 10] => return Some(Chord{root, chord_type: ChordType::DominantSeventh, inversion}),
            [0, 4, 8] => return Some(Chord{root, chord_type: ChordType::Augmented, inversion}),
            _ => {},
        }

        match keys_down.pop() {
            Some(i) => keys_down.insert(0, i - 12),
            _ => {}
        };
    }

    return None
}
