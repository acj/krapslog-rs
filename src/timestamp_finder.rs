use chrono::NaiveDateTime;
use regex::Regex;
pub struct TimestampFinder<'a> {
    datetime_format: &'a str,
    regex: Regex,
}

impl<'a> TimestampFinder<'a> {
    pub fn new(datetime_format: &'a str) -> Result<Self, anyhow::Error> {
        let datetime_regex = Self::strftime_to_regex(datetime_format);
        let regex = Regex::new(&datetime_regex)?;

        Ok(TimestampFinder {
            datetime_format,
            regex,
        })
    }

    pub fn find_timestamp(&self, s: &str) -> Option<i64> {
        let regex_match = self.regex.captures(s)?.get(0)?;
        let datetime =
            NaiveDateTime::parse_from_str(regex_match.as_str(), self.datetime_format).ok()?;
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
