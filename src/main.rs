#[macro_use]
extern crate clap;

extern crate rand;
extern crate sdl2;

mod disassembler;
mod display;
mod emulator;
mod input;
mod memory;
mod opcode;
mod sound;

use clap::{App, Arg};

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use disassembler::disassemble;
use emulator::System;

fn main() {
    let matches = App::new("alvin")
        .version(crate_version!())
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("What file to load")
                .takes_value(true)
                .required(true),
        ).subcommand(clap::SubCommand::with_name("disassemble"))
        .subcommand(
            clap::SubCommand::with_name("run").arg(
                Arg::with_name("debug")
                    .long("debug")
                    .help("Continuously dump emulator state to the command line")
                    .takes_value(false)
                    .required(false),
            ),
        ).get_matches();

    let filename = matches.value_of("file").expect("file is required");
    let file = File::open(filename).expect("file not found");

    let mut reader = BufReader::new(&file);
    let buffer = reader.fill_buf().unwrap();

    match matches.subcommand_name() {
        Some("disassemble") => disassemble(buffer),
        Some("run") => {
            let mut system = System::new(buffer);

            let dump_state = matches
                .subcommand_matches("run")
                .unwrap()
                .is_present("debug");
            system.run(dump_state);
        }
        _ => println!("ERROR: command invalid or not provided"),
    }
}
