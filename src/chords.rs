use std::error::Error;
use std::time;
use std::thread;
use rand::{thread_rng, seq::SliceRandom};
use dialoguer::{theme::ColorfulTheme, Select, Confirmation};
use termion::color;

use crate::midi::midi_connect;
use crate::utils::{
    music::{get_note_name, get_octave},
    mutex::{KEYS_DOWN, LAST_KEY_PRESS},
    types::{Hand, ChordType, Chord},
    constants::{DEBOUNCE_MILLIS, NOTE_NAMES},
};

pub fn practice_chords_launcher() -> Result<(), Box<dyn Error>> {
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

    match midi_connect() {
        Err(e) => Err(e),
        Ok(conn_in) => {
            let result = practice_chords(chord_type, inversion, hand);
            conn_in.close();
            result
        }
    }
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

