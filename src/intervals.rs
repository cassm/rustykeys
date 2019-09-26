use rand::{thread_rng, Rng, seq::SliceRandom};
use std::{thread, time};
use std::error::Error;
use dialoguer::{theme::ColorfulTheme, Select, Confirmation};
use termion::color;
use pitch_calc::{
    Letter,
    LetterOctave,
};
use portaudio as pa;
use crate::utils::constants::NOTE_NAMES;
use crate::synth::Synth;

const INTERVALS: &'static [&'static str] = &[
    "0) Unison",
    "1) Minor Second",
    "2) Major Second",
    "3) Minor Third",
    "4) Major Third",
    "5) Perfect Fourth",
    "6) Tritone",
    "7) Perfect Fifth",
    "8) Minor Sixth",
    "9) Major Sixth",
    "10) Minor Seventh",
    "11) Major Seventh",
    "12) Perfect Octave"];

pub fn practice_intervals_launcher() -> Result<(), Box<dyn Error>> {
    let random_root = Confirmation::new()
        .with_text("Would you like to use random starting pitches?")
        .interact()
        .unwrap();

    practice_listening(random_root)
}

fn practice_listening(random_root: bool) -> Result<(), Box<dyn Error>> {
    let mut rng = thread_rng();

    let mut intervals: Vec<usize> = Vec::new();
    for i in 0..INTERVALS.len() {
        intervals.push(i);
    }
    intervals.shuffle(&mut rng);

    for interval in intervals {
        let root_index = match random_root {
            true => rng.gen_range(0, NOTE_NAMES.len()),
            false => 0,
        };

        let note_names = [
            String::from(NOTE_NAMES[root_index][0]),
            String::from(NOTE_NAMES[(root_index+interval) % NOTE_NAMES.len()][0])
        ];

        let octaves = [
            3,
            3 + ((root_index + interval) / NOTE_NAMES.len())
        ];

        let notes = [
            LetterOctave(get_letter(&note_names[0]), octaves[0] as i32),
            LetterOctave(get_letter(&note_names[1]), octaves[1] as i32)
        ];

        let pa = pa::PortAudio::new()?;
        let syn = Synth{pa};

        for note in notes.iter() {
            syn.play_note(note.hz() as f64, 1000, false)?;
            thread::sleep(time::Duration::from_millis(1050));
        }

        loop {
            let interval_selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("What interval was this?")
                .items(INTERVALS)
                .interact()
                .unwrap();

            if interval_selection == interval {
                println!("{}Correct!{}", color::Fg(color::Green), color::Fg(color::Reset));
                break;
            }
            else if interval_selection > interval {
                println!("{}Less than that!{}", color::Fg(color::Red), color::Fg(color::Reset));
            }
            else {
                println!("{}More than that!{}", color::Fg(color::Red), color::Fg(color::Reset));
            }
        }
    }

    Ok(())
}

fn get_letter(note: &String) -> Letter {
    match &note[..] {
        "Ab" => Letter::Ab,
        "A" => Letter::A,
        "A#" => Letter::Ash,
        "Bb" => Letter::Bb,
        "B" => Letter::B,
        "C" => Letter::C,
        "C#" => Letter::Csh,
        "Db" => Letter::Db,
        "D" => Letter::D,
        "D#" => Letter::Dsh,
        "Eb" => Letter::Eb,
        "E" => Letter::E,
        "F" => Letter::F,
        "F#" => Letter::Fsh,
        "Gb" => Letter::Gb,
        "G" => Letter::G,
        "G#" => Letter::Gsh,
        &_ => Letter::C,
    }
}
