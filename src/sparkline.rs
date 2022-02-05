// Adapted from https://github.com/ferrouswheel/rust-sparkline

const SPARKS: &[&str] = &["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

pub fn sparkline(min: f64, max: f64, num: f64) -> &'static str {
    let proportion = (num - min) / (max - min);
    let index = std::cmp::min(
        (proportion * SPARKS.len() as f64) as usize,
        SPARKS.len() - 1,
    );
    SPARKS[index]
}

#[test]
fn test_sparkline() {
    let (min, max) = (0.0, 10.0);
    let values = vec![0.0, 2.0, 3.0, 2.0, 6.0, 9.0, 10.0];
    let s: Vec<_> = values.iter().map(|v| sparkline(min, max, *v)).collect();
    assert_eq!(s, &["▁", "▂", "▃", "▂", "▅", "█", "█"]);
}
