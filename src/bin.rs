extern crate clap;
extern crate krapslog;
extern crate progress_streams;
extern crate regex;
extern crate sparkline;

use anyhow::*;
use clap::{App, Arg};
use indicatif::ProgressBar;
use progress_streams::ProgressReader;
use std::fs;
use std::io::{self, prelude::*, BufReader};
use terminal_size::{terminal_size, Width};

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
        )
        .arg(
            Arg::with_name("PROGRESS")
                .short("p")
                .long("progress")
                .value_name("progress")
                .help("Display progress while working. Requires a file.")
                .required(false)
                .takes_value(false),
        );
    let arg_matches = app.get_matches();

    let timestamp_format = "%d/%b/%Y:%H:%M:%S%.f";
    let pb = ProgressBar::new(0);
    pb.set_draw_delta(10_000_000);

    let reader: Box<dyn BufRead> = match arg_matches.value_of("FILE") {
        None => {
            if atty::is(atty::Stream::Stdin) {
                eprintln!("Reading from standard input. Paste your log and then send EOF (e.g. by pressing ctrl-D).");
            }

            Box::new(BufReader::new(io::stdin()))
        }
        Some(filename) => {
            let mut reader: Box<dyn BufRead> = Box::new(BufReader::new(fs::File::open(filename)?));

            if arg_matches.is_present("PROGRESS") {
                let file_metadata = fs::metadata(filename)?;
                pb.set_length(file_metadata.len());

                reader = Box::new(BufReader::new(ProgressReader::new(reader, |bytes_read| {
                    pb.inc(bytes_read as u64);
                })));
            }

            reader
        }
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

    pb.finish_and_clear();

    let num_markers = clap::value_t!(arg_matches.value_of("MARKERS"), usize)?;
    let (header, footer) = krapslog::build_time_markers(&timestamps, num_markers, terminal_width);
    let sparkline = krapslog::build_sparkline(&timestamps, terminal_width)?;
    print!("{}", header);
    println!("{}", sparkline);
    print!("{}", footer);

    Ok(())
}
