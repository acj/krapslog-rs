use anyhow::{anyhow, Result};
use clap::{App, Arg};
use std::fs;
use std::io::{self, prelude::*};
use terminal_size::{terminal_size, Width};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let app = App::new("krapslog")
        .version(VERSION)
        .about("Visualize log files using sparklines")
        .arg(
            Arg::new("FILE")
                .help("Log file to analyze")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::new("FORMAT")
                .short('F')
                .long("format")
                .help("Timestamp format to match")
                .takes_value(true)
                .required(false)
                .default_value("%d/%b/%Y:%H:%M:%S%.f"),
        )
        .arg(
            Arg::new("MARKERS")
                .short('m')
                .long("markers")
                .help("Number of time markers to display")
                .takes_value(true)
                .required(false)
                .default_value("0"),
        );
    let arg_matches = app.get_matches();

    let timestamp_format = arg_matches.value_of("FORMAT").unwrap();

    let reader: Box<dyn Read> = match arg_matches.value_of("FILE") {
        None => {
            if atty::is(atty::Stream::Stdin) {
                eprintln!("Reading from standard input. Paste your log and then send EOF (e.g. by pressing ctrl-D).");
            }

            Box::new(io::stdin())
        }
        Some(filename) => {
            let file = fs::File::open(filename)?;

            Box::new(file)
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

    let num_markers = arg_matches.value_of_t("MARKERS")?;
    let (header, footer) = krapslog::build_time_markers(&timestamps, num_markers, terminal_width);
    let sparkline = krapslog::build_sparkline(&timestamps, terminal_width)?;
    print!("{}", header);
    println!("{}", sparkline);
    print!("{}", footer);

    Ok(())
}
