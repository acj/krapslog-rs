extern crate regex;
extern crate sparkline;

use chrono::NaiveDateTime;
use regex::Regex;
use sparkline::*;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use terminal_size::{terminal_size, Width};

fn main() -> io::Result<()> {
    let file = File::open("delivery-haproxy-alb-20191123.log")?;
    let reader = BufReader::new(file);

    let timestamp_regex = Regex::new(r"\d\d/[A-Za-z]{3}/\d{4}:\d\d:\d\d:\d\d\.\d\d\d").unwrap();
    let timestamp_format = "%d/%b/%Y:%H:%M:%S%.f";

    let mut timestamps = Vec::<i64>::new();
    let mut parse_error_count: usize = 0;

    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(re_match) = timestamp_regex.captures(&line) {
                if let Some(timestamp_text) = re_match.get(0) {
                    match NaiveDateTime::parse_from_str(timestamp_text.as_str(), &timestamp_format)
                    {
                        Ok(datetime) => timestamps.push(datetime.timestamp()),
                        _ => parse_error_count += 1,
                    }
                }
            }
        }
    }

    if parse_error_count > 0 {
        println!("Lines with missing or invalid date: {}", parse_error_count);
    }

    let mut terminal_width: usize = 80;
    if let Some((Width(w), _)) = terminal_size() {
        terminal_width = w as usize;
    }

    let first_timestamp = timestamps.iter().min().unwrap();
    let last_timestamp = timestamps.iter().max().unwrap();
    let duration_seconds = last_timestamp - first_timestamp;
    let seconds_per_bucket = (duration_seconds as f64 / terminal_width as f64).ceil() as i64;
    let mut time_buckets = vec![Vec::<i64>::new(); terminal_width];
    for timestamp in &timestamps {
        let bucket_index: usize =
            ((timestamp - first_timestamp) / seconds_per_bucket) as usize;
        time_buckets[bucket_index].push(*timestamp);
    }

    let bucket_counts: Vec<usize> = time_buckets.iter().map(|bucket| bucket.len()).collect();
    let (min, max) = (
        *bucket_counts.iter().min().unwrap() as f64,
        *bucket_counts.iter().max().unwrap() as f64,
    );
    let sparky = select_sparkline(SparkThemeName::Classic);
    for num in bucket_counts.iter() {
        let s: &String = sparky.spark(min, max, *num as f64);
        print!("{}", s);
    }

    Ok(())
}
