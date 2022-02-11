mod sparkline;
mod time_marker;
mod timestamp_finder;

use anyhow::*;
use std::io::{prelude::*, BufReader};

use crate::sparkline::sparkline;
use crate::timestamp_finder::TimestampFinder;

pub fn build_sparkline(timestamps: &[i64], length: usize) -> Result<String> {
    let timestamps_per_bucket = timestamp_frequency_distribution(timestamps, length);
    let (min, max) = (
        *timestamps_per_bucket.iter().min().unwrap() as f64,
        *timestamps_per_bucket.iter().max().unwrap() as f64,
    );
    let sparkline = timestamps_per_bucket
        .iter()
        .map(|count| sparkline(min, max, *count as f64).to_owned())
        .collect();
    Ok(sparkline)
}

pub fn scan_for_timestamps<R>(reader: R, format: &str) -> Result<Vec<i64>>
where
    R: Read,
{
    let date_finder = TimestampFinder::new(format)?;
    let timestamps = BufReader::new(reader)
        .lines()
        .filter_map(Result::ok)
        .filter_map(|line| date_finder.find_timestamp(&line))
        .collect();
    Ok(timestamps)
}

pub fn build_time_markers(
    timestamps: &[i64],
    marker_count: usize,
    terminal_width: usize,
) -> (String, String) {
    if marker_count == 0 {
        return (String::from(""), String::from(""));
    }

    let mut footer_marker_count = marker_count / 2;
    if footer_marker_count % 2 != 0 {
        footer_marker_count += 1;
    }

    let marker_timestamp_offsets: Vec<usize> = (0..marker_count)
        .map(|i| (i as f64 * timestamps.len() as f64 / marker_count as f64).ceil() as usize)
        .collect();
    let header_timestamp_offsets = marker_timestamp_offsets[footer_marker_count..].to_vec();
    let footer_timestamp_offsets = marker_timestamp_offsets[..footer_marker_count].to_vec();

    let marker_terminal_offsets = marker_offsets(marker_count, terminal_width);
    let header_terminal_offsets = marker_terminal_offsets[footer_marker_count..].to_vec();
    let footer_terminal_offsets = marker_terminal_offsets[..footer_marker_count].to_vec();

    let mut header_canvas =
        time_marker::Canvas::new(terminal_width, header_timestamp_offsets.len() + 1);
    let mut footer_canvas =
        time_marker::Canvas::new(terminal_width, footer_timestamp_offsets.len() + 1);

    header_timestamp_offsets
        .iter()
        .enumerate()
        .map(|(index, timestamp_index)| time_marker::TimeMarker {
            horizontal_offset: header_terminal_offsets[index],
            timestamp: timestamps[*timestamp_index],
            timestamp_location: time_marker::TimestampLocation::Top,
            vertical_offset: index + 1,
        })
        .for_each(|time_marker| time_marker.render(&mut header_canvas));

    footer_timestamp_offsets
        .iter()
        .enumerate()
        .map(|(index, timestamp_index)| time_marker::TimeMarker {
            horizontal_offset: footer_terminal_offsets[index],
            timestamp: timestamps[*timestamp_index],
            timestamp_location: time_marker::TimestampLocation::Bottom,
            vertical_offset: footer_timestamp_offsets.len() - index,
        })
        .for_each(|time_marker| time_marker.render(&mut footer_canvas));

    (format!("{}", header_canvas), format!("{}", footer_canvas))
}

fn marker_offsets(count: usize, terminal_width: usize) -> Vec<usize> {
    // Always show a marker at the left edge
    let mut offsets = vec![0];

    // Divide the non-edge offsets into equally-sized segments, placing a marker between them
    let skip = (terminal_width - 2) as f64 / (count - 1) as f64;
    let mut current_offset = skip;
    (0..(count - 2)).for_each(|_| {
        offsets.push(current_offset.ceil() as usize % terminal_width);
        current_offset += skip;
    });

    // Always show a marker at the right edge
    offsets.push(terminal_width - 1);

    offsets
}

fn timestamp_frequency_distribution(timestamps: &[i64], bucket_count: usize) -> Vec<usize> {
    let first_timestamp = timestamps.iter().min().unwrap();
    let last_timestamp = timestamps.iter().max().unwrap();
    let duration_seconds = last_timestamp - first_timestamp;
    let seconds_per_bucket = duration_seconds as f64 / bucket_count as f64;

    let mut timestamps_per_bucket = vec![0; bucket_count];
    for timestamp in timestamps {
        let bucket_index = usize::min(
            ((timestamp - first_timestamp) as f64 / seconds_per_bucket) as usize,
            bucket_count - 1,
        );
        timestamps_per_bucket[bucket_index] += 1;
    }

    timestamps_per_bucket
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_sparkline_() {
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
        let timestamps = scan_for_timestamps(log.as_bytes(), format).unwrap();
        let sparkline = build_sparkline(&timestamps, 80).unwrap();
        assert_eq!(
            sparkline,
            "█▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁▁█▁▁▁▁▁▁▁█"
        );
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
    fn timestamp_frequency_distribution_() {
        let timestamps = vec![1, 2, 3, 4, 5];
        let bins = timestamp_frequency_distribution(&timestamps, 5);
        assert_eq!(bins, [1, 1, 1, 1, 1]);

        let timestamps = vec![1, 2, 3, 4, 5, 6];
        let bins = timestamp_frequency_distribution(&timestamps, 3);
        assert_eq!(bins, [2, 2, 2]);
    }

    #[test]
    fn marker_offsets_() {
        assert_eq![marker_offsets(2, 2), vec![0, 1]];
        assert_eq![marker_offsets(2, 5), vec![0, 4]];
        assert_eq!(marker_offsets(5, 10), vec![0, 2, 4, 6, 9]);
        assert_eq!(marker_offsets(3, 5), vec![0, 2, 4]);
    }
}
