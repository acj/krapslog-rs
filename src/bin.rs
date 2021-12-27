extern crate clap;
extern crate krapslog;
extern crate progress_streams;
extern crate regex;
extern crate sparkline;

use anyhow::{anyhow, Result};
use clap::{value_t, App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use progress_streams::ProgressReader;
use std::fs;
use std::io::{self, prelude::*};
use terminal_size::{terminal_size, Width};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let app = App::new("krapslog")
        .version(VERSION)
        .about("Visualize log files using sparklines")
        .arg(
            Arg::with_name("FILE")
                .help("Log file to analyze")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::with_name("FORMAT")
                .short("F")
                .long("format")
                .help("Timestamp format to match")
                .takes_value(true)
                .required(false)
                .default_value("%d/%b/%Y:%H:%M:%S%.f"),
        )
        .arg(
            Arg::with_name("MARKERS")
                .short("m")
                .long("markers")
                .help("Number of time markers to display")
                .takes_value(true)
                .required(false)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("PROGRESS")
                .short("p")
                .long("progress")
                .help("Display progress while working. Requires a file.")
                .required(false)
                .takes_value(false),
        );
    let arg_matches = app.get_matches();

    let timestamp_format = arg_matches.value_of("FORMAT").unwrap();
    let pb = ProgressBar::new(0);
    pb.set_draw_delta(10_000_000);
    pb.set_style(
        ProgressStyle::default_bar().template("[{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"),
    );

    let reader: Box<dyn Read> = match arg_matches.value_of("FILE") {
        None => {
            if atty::is(atty::Stream::Stdin) {
                eprintln!("Reading from standard input. Paste your log and then send EOF (e.g. by pressing ctrl-D).");
            }

            Box::new(io::stdin())
        }
        Some(filename) => {
            let file = fs::File::open(filename)?;

            if arg_matches.is_present("PROGRESS") {
                let file_metadata = fs::metadata(filename)?;
                pb.set_length(file_metadata.len());

                Box::new(ProgressReader::new(file, |bytes_read| {
                    pb.inc(bytes_read as u64);
                }))
            } else {
                Box::new(file)
            }
        }
    };

    let terminal_width = match terminal_size() {
        Some((Width(w), _)) => w as usize,
        _ => 80,
    };

    let timestamps: Vec<i64> = krapslog::scan_for_timestamps(reader, timestamp_format)?;
    if timestamps.is_empty() {
        return Err(anyhow!("Found no lines with a matching timestamp"));
    }

    pb.finish_and_clear();

    let num_markers = value_t!(arg_matches.value_of("MARKERS"), usize)?;
    let (header, footer) = krapslog::build_time_markers(&timestamps, num_markers, terminal_width);
    let sparkline = krapslog::build_sparkline(&timestamps, terminal_width)?;
    print!("{}", header);
    println!("{}", sparkline);
    print!("{}", footer);

    Ok(())
}
