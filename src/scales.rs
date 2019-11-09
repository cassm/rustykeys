use std::error::Error;
use rand::{thread_rng, seq::SliceRandom};
use dialoguer::{theme::ColorfulTheme, Select, Confirmation};
use termion::color;
use std::io::{stdout, Write};
use ndarray::Array2;
use num_traits::FromPrimitive;

use crate::midi::midi_connect;
use crate::utils::{
    music::{note_matches, get_note_name},
    mutex::{KEYS_DOWN, LAST_KEY_PRESS},
    constants::{DEBOUNCE_MILLIS, NOTE_NAMES},
    types::Mode,
};

pub fn practice_scales_launcher() -> Result<(), Box<dyn Error>> {
    let modes: &'static [&'static str] = &[
        "Blues",
        "Major",
        "Minor",
     ];

    let mode_selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick a scale type")
        .items(modes)
        .interact()
        .unwrap();

    if let Some(mode) = FromPrimitive::from_usize(mode_selection) {
        match midi_connect() {
            Err(e) => Err(e),
            Ok(conn_in) => {
                let result = practice_scales(mode);
                conn_in.close();
                result
            }
        }
    } else {
        Err("Mode selection failed".into())
    }
}

fn practice_scales(mode: Mode) -> Result<(), Box<dyn Error>> {
    let mut replay = true;

    while replay {
        let scales = generate_scales(mode);

        for scale in scales.outer_iter() {
            print!("{}: ", scale[0]);
            stdout().flush()?;

            for note in scale.iter() {
                loop {
                    let last_key_press = *LAST_KEY_PRESS.lock().unwrap();

                    match last_key_press {
                        Some(i) => {
                            if i.elapsed().as_millis() > DEBOUNCE_MILLIS.into() {
                                *LAST_KEY_PRESS.lock().unwrap() = None;

                                if let Some(i) = KEYS_DOWN.lock().unwrap().last() {
                                    if note_matches(*i, &note) {
                                        // delete incorrect input
                                        print!("{}{}{} ", color::Fg(color::Green), note, color::Fg(color::Reset));
                                        stdout().flush()?;
                                        break;
                                    }
                                    else {
                                        print!("{}{}{} ", color::Fg(color::Red), get_note_name(*i), color::Fg(color::Reset));
                                        stdout().flush()?;
                                    }
                                }
                            }
                        }
                        _ => {},
                    }
                }
            }

            println!("");
        }

        replay = Confirmation::new()
            .with_text("Would you like to practice again?")
            .interact()
            .unwrap();
    }

    Ok(())
}

fn generate_scales(mode: Mode) -> Array2::<String> {
    let mut rng = thread_rng();
    let mut root_indices: Vec<usize> = (0..NOTE_NAMES.len()).collect();
    root_indices.shuffle(&mut rng);
    let intervals = mode.value();

    println!("Generating {} scales with intervals {:?}", mode, mode.value());

    let mut scales = Array2::<String>::default((NOTE_NAMES.len(), intervals.len()*2 + 1));

    for (i, root_index) in root_indices.iter().enumerate() {
        let mut offset: usize = 0;

        for j in 0..intervals.len() + 1 {
            let note_index = (root_index + offset) % NOTE_NAMES.len();

            // use the correct sharp or flat
            if j > 0 && NOTE_NAMES[note_index].len() > 1 && NOTE_NAMES[note_index][0][..1] == scales[[i, j-1]][..1] {
                scales[[i, j]] = NOTE_NAMES[note_index][1].to_string();
                scales[[i, intervals.len()*2 - j]] = NOTE_NAMES[note_index][1].to_string();
            }
            else {
                scales[[i, j]] = NOTE_NAMES[note_index][0].to_string();
                scales[[i, intervals.len()*2 - j]] = NOTE_NAMES[note_index][0].to_string();
            }

            offset += intervals[j % intervals.len()];
        }
    }

    scales
}
