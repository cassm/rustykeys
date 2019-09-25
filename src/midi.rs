use midir::{MidiInputConnection, MidiInput, Ignore};
use std::io::{stdin, stdout, Write};
use std::error::Error;
use std::time::Instant;

use crate::utils::mutex::{LAST_KEY_PRESS, KEYS_DOWN};

pub fn midi_connect() -> Result<MidiInputConnection<()>, Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midir forwarding input")?;
    midi_in.ignore(Ignore::None);

    let in_port = match get_in_port(&midi_in) {
        Ok(i)  => i,
        Err(e) => return Err(e),
    };

    let in_port_name = midi_in.port_name(in_port)?;

    println!("\nOpening connection...");

    let conn_in = midi_in.connect(in_port, "midir-read-input", move |_, message, _| {
        *LAST_KEY_PRESS.lock().unwrap() = Some(Instant::now());
        process_msg(message);
    }, ())?;

    println!("Connection open, reading input from '{}'. Press ^C to quit\n", in_port_name);

    Ok(conn_in)
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

fn process_msg(msg: &[u8]) {
    match msg[0] {
        0x90 => {
            *LAST_KEY_PRESS.lock().unwrap() = Some(Instant::now());
            KEYS_DOWN.lock().unwrap().push(msg[1]);
        },
        0x80 => {
            let index = KEYS_DOWN.lock().unwrap().iter().position(|x| *x == msg[1]).unwrap();
            KEYS_DOWN.lock().unwrap().remove(index);
        },
        _ => {},
    }
}

