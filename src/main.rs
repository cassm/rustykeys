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

fn main() -> Result<(), Box<dyn Error>> {
    let options = &[
        "Practice chords",
        "Practice scales",
    ];

    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What would you like to do?")
        .items(options)
        .interact()
        .unwrap()
    {
        0 => practice_chords_launcher(),
        1 => practice_scales_launcher(),
        _ => Ok(()),
    }
}
