use anyhow::{anyhow, Result};
use clap::{Arg, Command};
use rayon::prelude::*;
use std::fs;
use terminal_size::{terminal_size, Width};

use file_chunker::FileChunker;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let num_cores = num_cpus::get_physical();
    let num_cores_for_display = format!("{}", num_cores);
    let cmd = Command::new("krapslog")
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
        )
        .arg(
            Arg::new("CONCURRENCY")
                .short('c')
                .long("concurrency")
                .help("Number of threads to use when processing large files (defaults to number of CPU cores)")
                .takes_value(true)
                .required(false)
                .default_value(&num_cores_for_display),
        );
    let arg_matches = cmd.get_matches();

    let timestamp_format = arg_matches.value_of("FORMAT").unwrap();
    let timestamps = match arg_matches.value_of("FILE") {
        None => {
            if atty::is(atty::Stream::Stdin) {
                eprintln!("Reading from standard input. Paste your log and then send EOF (e.g. by pressing ctrl-D).");
            }

            krapslog::scan_for_timestamps(std::io::stdin(), timestamp_format)
        }
        Some(filename) => {
            let file = fs::File::open(filename)?;
            let chunker = FileChunker::new(&file)?;
            let mut count = arg_matches.value_of_t("CONCURRENCY").unwrap();
            if file.metadata()?.len() < 10 * 1024 * 1024 {
                count = 1;
            }

            Ok(chunker
                .chunks(count, Some('\n'))?
                .into_par_iter()
                .map(|chunk| krapslog::scan_for_timestamps(chunk, timestamp_format))
                .filter_map(Result::ok)
                .collect::<Vec<Vec<i64>>>()
                .concat())
        }
    }?;

    if timestamps.is_empty() {
        return Err(anyhow!("Found no lines with a matching timestamp"));
    }

    let terminal_width = match terminal_size() {
        Some((Width(w), _)) => w as usize,
        _ => 80,
    };

    let num_markers = arg_matches.value_of_t("MARKERS")?;
    let (header, footer) = krapslog::build_time_markers(&timestamps, num_markers, terminal_width);
    let sparkline = krapslog::build_sparkline(&timestamps, terminal_width)?;
    print!("{}", header);
    println!("{}", sparkline);
    print!("{}", footer);

    Ok(())
}
