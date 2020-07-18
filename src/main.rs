extern crate regex;
extern crate sparkline;

use anyhow::*;
use chrono::NaiveDateTime;
use regex::Regex;
use sparkline::*;
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

    let sparkline = build_sparkline(reader, timestamp_format, terminal_width)?;
    println!("{}", sparkline);

    Ok(())
}

fn build_sparkline(
    reader: impl BufRead,
    timestamp_format: &str,
    length: usize,
) -> Result<String, anyhow::Error> {
    let timestamps: Vec<i64> = scan_for_timestamps(reader, timestamp_format)?;
    if timestamps.is_empty() {
        return Err(anyhow!("Found no lines with a matching timestamp"));
    }

    let line_counts = bin_timestamps(&timestamps, length);
    let (min, max) = (
        *line_counts.iter().min().unwrap() as f64,
        *line_counts.iter().max().unwrap() as f64,
    );
    let sparky = select_sparkline(SparkThemeName::Classic);

    let sparkline = line_counts
        .iter()
        .map(|count| sparky.spark(min, max, *count as f64).to_owned())
        .collect();
    Ok(sparkline)
}

fn scan_for_timestamps(reader: impl BufRead, format: &str) -> Result<Vec<i64>, anyhow::Error> {
    let date_finder = TimestampFinder::new(format)?;
    let timestamps = reader
        .lines()
        .filter_map(Result::ok)
        .filter_map(|line| date_finder.find_timestamp(&line))
        .collect();
    Ok(timestamps)
}

fn bin_timestamps(timestamps: &[i64], length: usize) -> Vec<usize> {
    let first_timestamp = timestamps.iter().min().unwrap();
    let last_timestamp = timestamps.iter().max().unwrap();
    let duration_seconds = last_timestamp - first_timestamp;
    let seconds_per_bucket = (duration_seconds as f64 / length as f64).ceil();
    let mut time_buckets = vec![Vec::<i64>::new(); length];
    for timestamp in timestamps {
        let bucket_index = ((timestamp - first_timestamp) as f64 / seconds_per_bucket) as usize;
        time_buckets[bucket_index].push(*timestamp);
    }

    time_buckets.iter().map(|bucket| bucket.len()).collect()
}

struct TimestampFinder<'a> {
    datetime_format: &'a str,
    regex: Regex,
}

impl<'a> TimestampFinder<'a> {
    fn new(datetime_format: &'a str) -> Result<Self, anyhow::Error> {
        let datetime_regex = Self::strftime_to_regex(datetime_format);
        let regex = Regex::new(&datetime_regex)?;

        Ok(TimestampFinder {
            datetime_format,
            regex,
        })
    }

    fn find_timestamp(&self, s: &str) -> Option<i64> {
        let regex_match = self.regex.captures(s)?.get(0)?;
        let datetime = NaiveDateTime::parse_from_str(regex_match.as_str(), self.datetime_format).ok()?;
        Some(datetime.timestamp())
    }

    fn strftime_to_regex(time_format: &str) -> String {
        time_format
            .replace("%Y", r"\d{1,4}")
            .replace("%C", r"\d{1,2}")
            .replace("%y", r"\d{1,2")
            .replace("%m", r"\d{1,2}")
            .replace("%b", r"[A-Za-z]{3}")
            .replace("%B", r"[A-Za-z]{3,4,5,6,7,8,9}")
            .replace("%h", r"[A-Za-z]{3}")
            .replace("%d", r"\d{1,2}")
            .replace("%H", r"\d{1,2}")
            .replace("%M", r"\d{1,2}")
            .replace("%S", r"\d{1,2}")
            .replace("%.f", r"\d{1,}")
        // TODO: Add support for remaining characters. https://docs.rs/chrono/0.4.13/chrono/format/strftime/index.html
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_sparkline_() -> Result<(), anyhow::Error> {
        let log = "Nov 23 06:26:40 ip-10-1-1-1 haproxy[20128]: 10.1.1.10:57305 [23/Nov/2019:06:26:40.781] public myapp/i-05fa49c0e7db8c328 0/0/0/78/78 206 913/458 - - ---- 9/9/6/0/0 0/0 {bytes=0-0} {||1|bytes 0-0/499704} \"GET \
        /2518cb13a48bdf53b2f936f44e7042a3cc7baa06 HTTP/1.1\"
Nov 23 06:26:41 ip-10-1-1-1 haproxy[20128]: 10.1.1.11:51819 [23/Nov/2019:06:27:41.780] public myapp/i-059c225b48702964a 0/0/0/80/80 200 802/142190 - - ---- 8/8/5/0/0 0/0 {} {||141752|} \"GET /2043f2eb9e2691edcc0c8084d1ff\
ce8bd70bc6e7 HTTP/1.1\"
Nov 23 06:26:42 ip-10-1-1-1 haproxy[20128]: 10.1.1.12:38870 [23/Nov/2019:06:28:42.773] public myapp/i-048088fd46abe7ed0 0/0/0/77/100 200 823/512174 - - ---- 8/8/5/0/0 0/0 {} {||511736|} \"GET /eb59c0b5dad36f080f3d261c625\
7ce0e21ef1a01 HTTP/1.1\"
Nov 23 06:26:43 ip-10-1-1-1 haproxy[20128]: 10.1.1.13:35528 [23/Nov/2019:06:29:43.775] public myapp/i-05e9315b035d50f62 0/0/0/103/105 200 869/431481 - - ---- 8/8/1/0/0 0/0 {} {|||} \"GET /164672c9d75c76a8fa237c24f9cbfd22\
22554f6d HTTP/1.1\"
Nov 23 06:26:44 ip-10-1-1-1 haproxy[20128]: 10.1.1.14:48553 [23/Nov/2019:06:30:44.808] public myapp/i-0008bfe6b1c98e964 0/0/0/72/73 200 840/265518 - - ---- 7/7/5/0/0 0/0 {} {||265080|} \"GET /e3b526928196d19ab3419d433f3d\
e0ceb71e62b5 HTTP/1.1\"
Nov 23 06:26:45 ip-10-1-1-1 haproxy[20128]: 10.1.1.15:60969 [23/Nov/2019:06:31:45.727] public myapp/i-005a2bfdba4c405a8 0/0/0/146/167 200 852/304622 - - ---- 7/7/5/0/0 0/0 {} {||304184|} \"GET /52f5edb4a46276defe54ead2fa\
e3a19fb8cafdb6 HTTP/1.1\"
Nov 23 06:26:46 ip-10-1-1-1 haproxy[20128]: 10.1.1.14:48539 [23/Nov/2019:06:32:46.730] public myapp/i-03b180605be4fa176 0/0/0/171/171 200 889/124142 - - ---- 6/6/4/0/0 0/0 {} {||123704|} \"GET /ef9e0c85cc1c76d7dc777f5b19\
d7cb85478496e4 HTTP/1.1\"
Nov 23 06:26:47 ip-10-1-1-1 haproxy[20128]: 10.1.1.11:51847 [23/Nov/2019:06:33:47.886] public myapp/i-0aa566420409956d6 0/0/0/28/28 206 867/458 - - ---- 6/6/4/0/0 0/0 {bytes=0-0} {} \"GET /3c7ace8c683adcad375a4d14995734a\
c0db08bb3 HTTP/1.1\"
Nov 23 06:26:48 ip-10-1-1-1 haproxy[20128]: 10.1.1.13:35554 [23/Nov/2019:06:34:48.866] public myapp/i-07f4205f35b4774b6 0/0/0/23/49 200 816/319662 - - ---- 5/5/3/0/0 0/0 {} {||319224|} \"GET /b95db0578977cd32658fa28b386c\
0db67ab23ee7 HTTP/1.1\"
Nov 23 06:26:49 ip-10-1-1-1 haproxy[20128]: 10.1.1.12:38899 [23/Nov/2019:06:35:49.879] public myapp/i-08cb5309afd22e8c0 0/0/0/59/59 200 1000/112110 - - ---- 5/5/3/0/0 0/0 {} {||111672|} \"GET /5314ca870ed0f5e48a71adca185\
e4ff7f1d9d80f HTTP/1.1\"
";
        let format = "%d/%b/%Y:%H:%M:%S%.f";
        let sparkline = build_sparkline(log.as_bytes(), format, 80)?;
        assert_eq!(
            sparkline,
            "█▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁"
        );

        return Ok(());
    }

    #[test]
    fn timestamp_finder() {
        let format = "%d/%b/%Y:%H:%M:%S%.f";
        let date_finder = TimestampFinder::new(format).unwrap();
        let log = "Nov 23 06:26:40 ip-10-1-26-81 haproxy[20128]: 54.242.135.245:57305 [23/Nov/2019:06:26:40.781] public repackager/i-05fa49c0e7db8c328 0/0/0/78/78 206 913/458 - - ---- 9/9/6/0/0 0/0 {1.1 v1-akamaitech.net(ghost) (AkamaiGHost), 1.1 v1-akamaitech.net(ghost) (AkamaiGHost), 1.1 akamai.n|bytes=0-0} {||1|bytes 0-0/499704} \"GET /deliveries/2518cb13a48bdf53b2f936f44e7042a3cc7baa06.m3u8/seg-88-v1-a1.ts HTTP/1.1\"";
        let timestamp = date_finder.find_timestamp(log).unwrap();
        assert_eq!(timestamp, 1574490400);        
    }

    #[test]
    fn timestamp_finder_strftime_to_regex() {
        let convert_compile_match = |format: &str, match_str: &str| {
            let format_regex = TimestampFinder::strftime_to_regex(format);
            let regex = Regex::new(&format_regex).unwrap();
            assert!(regex.is_match(match_str));
        };

        convert_compile_match("%d/%b/%Y:%H:%M:%S%.f", "06/Jan/2006:13:04:05.000");
    }

    #[test]
    fn scan_for_timestamps_() {
        let log = "Nov 23 06:26:40 ip-10-1-26-81 haproxy[20128]: 54.242.135.245:57305 [23/Nov/2019:06:26:40.781] public repackager/i-05fa49c0e7db8c328 0/0/0/78/78 206 913/458 - - ---- 9/9/6/0/0 0/0 {1.1 v1-akamaitech.net(ghost) (AkamaiGHost), 1.1 v1-akamaitech.net(ghost) (AkamaiGHost), 1.1 akamai.n|bytes=0-0} {||1|bytes 0-0/499704} \"GET /deliveries/2518cb13a48bdf53b2f936f44e7042a3cc7baa06.m3u8/seg-88-v1-a1.ts HTTP/1.1\"
Nov 23 14:21:53 ip-10-1-26-81 haproxy[20128]: 54.209.125.72:58030 [23/Nov/2019:14:21:53.241] public repackager/i-0728dc03214405429 0/0/0/246/246 200 810/8324 - - ---- 17/17/12/0/0 0/0 {1.1 v1-akamaitech.net(ghost) (AkamaiGHost), 1.1 v1-akamaitech.net(ghost) (AkamaiGHost), 1.1 akamai.n|} {||7870|} \"GET /deliveries/4fb7b6ff75f8a13da4ac482e25e29790105ba075.m3u8?origin_v2=1 HTTP/1.1\"
";
        let format = "%d/%b/%Y:%H:%M:%S%.f";
        let timestamps = scan_for_timestamps(log.as_bytes(), format).unwrap();
        assert_eq!(timestamps, [1574490400, 1574518913]);
    }

    #[test]
    fn bin_timestamps_() {
        let timestamps = vec![1, 2, 3, 4, 5];
        let bins = bin_timestamps(&timestamps, 5);
        assert_eq!(bins, [1, 1, 1, 1, 1]);

        let timestamps = vec![1, 2, 3, 4, 5, 6];
        let bins = bin_timestamps(&timestamps, 3);
        assert_eq!(bins, [2, 2, 2]);
    }
}
