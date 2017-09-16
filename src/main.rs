#[macro_use]
extern crate clap;

mod opcode;
mod emulator;

use clap::{App, Arg};

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use opcode::Opcode;
use emulator::System;

fn main() {
    let matches = App::new("alvin")
        .version(crate_version!())
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("What file to load")
            .takes_value(true)
            .required(true)
        ).get_matches();

    let filename = matches.value_of("file").expect("file is required");
    let file = File::open(filename).expect("file not found");

    let mut reader = BufReader::new(&file);
    let buffer = reader.fill_buf().unwrap();

    let opcodes: Vec<Option<Opcode>> = buffer.chunks(2).map(|chunk| {
        let first = chunk[0];
        let second: u8 = if chunk.len() == 2 {
            chunk[1]
        } else {
            0
        };

        Opcode::from(first, second)
    }).collect();

    let mut system = System::new();
    for opcode in opcodes {
        if let Some(opcode) = opcode {
            println!("Processing: {:?}", opcode);
            &system.process_opcode(opcode);
        } else {
            println!("Unrecognized opcode");
        }

        &system.tick(60);
    }

    println!("{:#?}", system);
}
