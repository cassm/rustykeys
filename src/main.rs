#[macro_use]
extern crate lazy_static;
extern crate midir;
extern crate ordinal;
extern crate timer;
extern crate chrono;
extern crate rand;
extern crate dialoguer;
extern crate termion;

use ordinal::Ordinal;
use std::sync::Mutex;
use std::io::{stdin, stdout, Write};
use std::error::Error;
use std::fmt;
use std::time::{self, Instant};
use std::thread;

use midir::{MidiInput, Ignore};
use rand::{thread_rng, seq::SliceRandom};
use dialoguer::{theme::ColorfulTheme, Select, Confirmation};
use termion::color;


const NOTE_NAMES: &'static [&'static [&'static str]] = &[&["A"], &["A#", "Bb"], &["B"], &["C"], &["C#", "Db"], &["D"], &["D#", "Eb"], &["E"], &["F"], &["F#", "Gb"], &["G"], &["G#", "Ab"]];
const MAJOR_SCALE_INTERVALS: &'static [usize] = &[2, 2, 1, 2, 2, 2, 1];
const DEBOUNCE_MILLIS: u64 = 100;


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


enum Mode {
        Ionian = 0,
        Dorian = 1,
        Phrygian = 2,
        Lydian = 3,
        Mixolydian = 4,
        Aeolian = 5,
        Locrian = 6,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Hand {
    Left,
    Right,
    Both,
}

impl Hand {
    fn value(&self) -> Vec<u8> {
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
    let options = &[
        "Practice chords",
        "Practice scales",
    ];

    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick a root note")
        .items(options)
        .interact()
        .unwrap()
    {
        0 => practice_chords_launcher(),
        1 => practice_scales_launcher(),
        _ => Ok(()),
    }
}

fn practice_chords_launcher() -> Result<(), Box<dyn Error>> {
    let chord_type_selections = &[
        ("Major", ChordType::Major),
        ("Minor", ChordType::Minor),
        ("Diminished", ChordType::Diminished),
        ("Major Seventh", ChordType::MajorSeventh),
        ("Minor Seventh", ChordType::MinorSeventh),
        ("Dominant Seventh", ChordType::DominantSeventh),
        ("Augmented", ChordType::Augmented),
        ("sus2", ChordType::SusTwo),
        ("sus4", ChordType::SusFour),
        ("7sus2", ChordType::SevenSusTwo),
        ("7sus4", ChordType::SevenSusFour),
        ("sus6", ChordType::SusSix )];

    let chord_variants: Vec<&str>  = chord_type_selections.iter().map(|x| x.0).collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick a chord variant")
        .items(chord_variants.as_slice())
        .interact()
        .unwrap();

    let chord_type = chord_type_selections[selection].1;

    let inversion_selections = [0, 1, 2];

    let inversion = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick an inversion")
        .items(&inversion_selections)
        .interact()
        .unwrap();

    let hand_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Which hand?")
        .items(&["left", "right", "both"])
        .interact()
        .unwrap();

    let hand = match hand_selection {
        0 => Hand::Left,
        1 => Hand::Right,
        _ => Hand::Both,
    };

    practice_chords(chord_type, inversion, hand)
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

fn generate_chord_list(chord_type: ChordType, inversion: usize, hand: Hand) -> Vec<(Chord, Hand)> {
    let mut rng = thread_rng();
    let mut chords: Vec<(Chord, Hand)> = vec!();

    if hand == Hand::Left || hand == Hand::Both {
        let chords_to_add: Vec<(Chord, Hand)> = NOTE_NAMES.iter().map(|x| (Chord{root: x[0].to_string(), chord_type, inversion, octave: None}, Hand::Left)).collect();
        chords.extend(chords_to_add);

    }
    if hand == Hand::Right || hand == Hand::Both {
        let chords_to_add: Vec<(Chord, Hand)> = NOTE_NAMES.iter().map(|x| (Chord{root: x[0].to_string(), chord_type, inversion, octave: None}, Hand::Right)).collect();
        chords.extend(chords_to_add);
    }

    chords.shuffle(&mut rng);
    chords
}

fn practice_chords(chord_type: ChordType, inversion: usize, hand: Hand) -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midir forwarding input")?;
    midi_in.ignore(Ignore::None);

    let in_port = match get_in_port(&midi_in) {
        Ok(i)  => i,
        Err(e) => return Err(e),
    };

    let in_port_name = midi_in.port_name(in_port)?;

    println!("\nOpening connection...");

    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |_, message, _| {
        *LAST_KEY_PRESS.lock().unwrap() = Some(Instant::now());
        process_msg(message);
    }, ())?;

    println!("Connection open, reading input from '{}'. Press ^C to quit\n", in_port_name);

    let mut replay = true;

    while replay {
        let mut chords = generate_chord_list(chord_type, inversion, hand);


        println!("Play {}, {}", chords[0].0, chords[0].1);

        while chords.len() > 0 {
            let last_key_press = *LAST_KEY_PRESS.lock().unwrap();

            match last_key_press {
                Some(i) => {
                    if i.elapsed().as_millis() > DEBOUNCE_MILLIS.into() {
                        *LAST_KEY_PRESS.lock().unwrap() = None;
                        if KEYS_DOWN.lock().unwrap().len() > 0 {
                            match identify_chord() {
                                Some(i) => {
                                    let octave_match = match i.octave {
                                        Some(j) => chords[0].1.value().contains(&j),
                                        None => true,
                                    };

                                    if chords[0].0 == i && octave_match {
                                        println!("{}Correct!{}", color::Fg(color::Green), color::Fg(color::Reset));
                                        chords.remove(0);
                                        thread::sleep(time::Duration::from_millis(DEBOUNCE_MILLIS));

                                        if chords.len() > 0 {
                                            println!("Play {}, {}", chords[0].0, chords[0].1);
                                        }
                                    }
                                    else {
                                        println!("{}Try again: {}, {}{}", color::Fg(color::Red), chords[0].0, chords[0].1, color::Fg(color::Reset));
                                        thread::sleep(time::Duration::from_millis(DEBOUNCE_MILLIS));
                                    }
                                },
                                None => {
                                    println!("{}unrecognised chord\nTry again: {}, {}{}", color::Fg(color::Red), chords[0].0, chords[0].1, color::Fg(color::Reset));
                                    thread::sleep(time::Duration::from_millis(DEBOUNCE_MILLIS));
                                }
                            }
                        }
                    }
                },
                _ => {},
            }
        }

        replay = Confirmation::new()
            .with_text("Would you like to practice again?")
            .interact()
            .unwrap();
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

    NOTE_NAMES[note_index as usize % NOTE_NAMES.len()][0].to_string()
}

fn note_matches(key_index: u8, note: &str) -> bool {
    let midi_start_index = 21;

    let note_index = key_index - midi_start_index;

    return NOTE_NAMES[note_index as usize % NOTE_NAMES.len()].contains(&note);
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

fn practice_scales_launcher() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midir forwarding input")?;
    midi_in.ignore(Ignore::None);

    let in_port = match get_in_port(&midi_in) {
        Ok(i)  => i,
        Err(e) => return Err(e),
    };

    let in_port_name = midi_in.port_name(in_port)?;

    println!("\nOpening connection...");

    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |_, message, _| {
        *LAST_KEY_PRESS.lock().unwrap() = Some(Instant::now());
        process_msg(message);
    }, ())?;

    println!("Connection open, reading input from '{}'. Press ^C to quit\n", in_port_name);

    let mut replay = true;

    while replay {
        let mut scale = generate_scale();

        while scale.len() > 0 {
            let last_key_press = *LAST_KEY_PRESS.lock().unwrap();

            match last_key_press {
                Some(_) => {
                    *LAST_KEY_PRESS.lock().unwrap() = None;

                    if let Some(i) = KEYS_DOWN.lock().unwrap().last() {
                        if note_matches(*i, &scale[0]) {
                            // delete incorrect input
                            print!("{}{}{} ", color::Fg(color::Green), scale[0], color::Fg(color::Reset));
                            stdout().flush()?;
                            scale.remove(0);
                            thread::sleep(time::Duration::from_millis(DEBOUNCE_MILLIS));
                        }
                        else {
                            print!("{}{}{} ", color::Fg(color::Red), get_note_name(*i), color::Fg(color::Reset));
                            stdout().flush()?;
                            thread::sleep(time::Duration::from_millis(DEBOUNCE_MILLIS));
                        }

                    }
                }
                _ => {},
            }
        }
        println!("");

        replay = Confirmation::new()
            .with_text("Would you like to practice again?")
            .interact()
            .unwrap();
    }
    Ok(())
}

fn generate_scale() -> Vec<String> {
    let modes: &'static [&'static str] = &[
        "I   - Ionian (major scale)",
        "II  - Dorian (a minor scale with a sharp 6th, gives it a bit of a jazzy/upbeat vibe)",
        "III - Phrygian (very common in metal or spanish classical. Really brooding, depressing mode)",
        "IV  - Lydian (Kind of spacey and dreamy, like a major scale that is tripping)",
        "V   - Mixolydian (bluesy)",
        "VI  - Aeolian (minor scale, omnipresent in music)",
        "VII - Locrian (rarely used)",
     ];

    let note_names: Vec<&str> = NOTE_NAMES.iter().map(|x| x[0]).collect();

    let root_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick a root note")
        .items(note_names.as_slice())
        .interact()
        .unwrap();

    let mode_offset = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick a scale type")
        .items(modes)
        .interact()
        .unwrap();

    let mut scale: Vec<String> = vec!();
    let mut offset: usize = 0;

    for i in 0..MAJOR_SCALE_INTERVALS.len() + 1 {
        let note_index = (root_index + offset) % NOTE_NAMES.len();

        // use the correct sharp or flat
        if let Some(j) = scale.last() {
            if NOTE_NAMES[note_index].len() > 1 && NOTE_NAMES[note_index][0][..1] == j[..1] {
                scale.push(NOTE_NAMES[note_index][1].to_string());
            }
            else {
                scale.push(NOTE_NAMES[note_index][0].to_string());
            }
        }
        else {
            scale.push(NOTE_NAMES[note_index][0].to_string());
        }

        let interval_index = (i + mode_offset) % MAJOR_SCALE_INTERVALS.len();
        offset += MAJOR_SCALE_INTERVALS[interval_index];
    }

    scale
}
