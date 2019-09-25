// TODO implement grading and spaced repetition
// TODO chord progression practice
// TODO sight reading practice
// TODO riff practice
// TODO chord-to-chord practice
// TODO split into separate files oh my god

#[macro_use]
extern crate lazy_static;
extern crate midir;
extern crate ordinal;
extern crate timer;
extern crate chrono;
extern crate rand;
extern crate dialoguer;
extern crate termion;
extern crate ndarray;

mod utils;
mod midi;
mod chords;
mod scales;

use std::error::Error;
use dialoguer::{theme::ColorfulTheme, Select};

use chords::practice_chords_launcher;
use scales::practice_scales_launcher;

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err.description())
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
