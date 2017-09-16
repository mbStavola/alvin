#[macro_use]
extern crate clap;

mod opcode;
mod emulator;

use clap::{App, Arg};

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

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

    let mut system = System::new(buffer);
    system.run();
}
