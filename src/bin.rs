extern crate clap;
extern crate krapslog;
extern crate regex;
extern crate sparkline;

use anyhow::*;
use clap::{App, Arg};
use std::fs;
use std::io::{self, prelude::*, BufReader};
use terminal_size::{terminal_size, Width};

// TODO: progress https://crates.io/crates/indicatif

fn main() -> Result<()> {
    let app = App::new("krapslog")
        .about("Visualize log files using sparklines")
        .arg(
            Arg::with_name("FILTER")
                .short("f")
                .long("filter")
                .value_name("filter")
                .help("Only consider lines that contain this value")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("FILE")
                .help("Log file to analyze")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::with_name("MARKERS")
                .short("m")
                .long("markers")
                .value_name("markers")
                .help("Number of time markers to display")
                .takes_value(true)
                .required(false)
                .default_value("0"),
        );
    let arg_matches = app.get_matches();

    let timestamp_format = "%d/%b/%Y:%H:%M:%S%.f";
    let reader: Box<dyn BufRead> = match arg_matches.value_of("FILE") {
        None => {
            if atty::is(atty::Stream::Stdin) {
                eprintln!("Reading from standard input. Paste your log and then send EOF (e.g. by pressing ctrl-D).");
            }

            Box::new(BufReader::new(io::stdin()))
        }
        Some(filename) => Box::new(BufReader::new(fs::File::open(filename)?)),
    };

    let terminal_width = match terminal_size() {
        Some((Width(w), _)) => w as usize,
        _ => 80,
    };

    let timestamps: Vec<i64> =
        krapslog::scan_for_timestamps(reader, timestamp_format, arg_matches.value_of("FILTER"))?;
    if timestamps.is_empty() {
        return Err(anyhow!("Found no lines with a matching timestamp"));
    }

    let num_markers = clap::value_t!(arg_matches.value_of("MARKERS"), usize)?;
    let (header, footer) = krapslog::build_time_markers(&timestamps, num_markers, terminal_width);
    let sparkline = krapslog::build_sparkline(&timestamps, terminal_width)?;
    print!("{}", header);
    println!("{}", sparkline);
    print!("{}", footer);

    Ok(())
}
