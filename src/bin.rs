extern crate krapslog;
extern crate regex;
extern crate sparkline;

use anyhow::*;
use std::env;
use std::fs;
use std::io::{self, prelude::*, BufReader};
use terminal_size::{terminal_size, Width};

// TODO: CLI args https://crates.io/crates/clap
// TODO: progress https://crates.io/crates/indicatif
// TODO: Display time markers

fn main() -> Result<()> {
    let timestamp_format = "%d/%b/%Y:%H:%M:%S%.f";
    let input = env::args().nth(1);
    let reader: Box<dyn BufRead> = match input {
        None => {
            if atty::is(atty::Stream::Stdin) {
                // TODO: formalize usage message
                println!("usage: krapslog <file.log>");
                std::process::exit(1);
            }

            Box::new(BufReader::new(io::stdin()))
        }
        Some(filename) => Box::new(BufReader::new(fs::File::open(filename)?)),
    };

    let terminal_width = match terminal_size() {
        Some((Width(w), _)) => w as usize,
        _ => 80,
    };

    let sparkline = krapslog::build_sparkline(reader, timestamp_format, terminal_width)?;
    println!("{}", sparkline);

    Ok(())
}