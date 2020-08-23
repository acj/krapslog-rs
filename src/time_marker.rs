use chrono::NaiveDateTime;
use std::fmt;

pub enum TimestampLocation {
    Top,
    Bottom,
}

pub struct Canvas {
    buffer: Vec<String>,
    height: usize,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        let buffer = vec![String::from(" ").repeat(width); height];
        Canvas { buffer, height }
    }

    fn update_row<F>(&mut self, offset: usize, f: F)
    where
        F: Fn(&String) -> String,
    {
        self.buffer[offset] = f(&self.buffer[offset]);
    }
}

impl fmt::Display for Canvas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.buffer.iter() {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

pub struct TimeMarker {
    pub horizontal_offset: usize,
    pub timestamp_location: TimestampLocation,
    pub timestamp: i64,
    pub vertical_offset: usize,
}

impl TimeMarker {
    pub fn render(&self, canvas: &mut Canvas) {
        let time = NaiveDateTime::from_timestamp(self.timestamp, 0).to_string();

        let (stem_rows, timestamp_row, timestamp_horizontal_offset) = match &self.timestamp_location
        {
            TimestampLocation::Top => (
                (canvas.height - self.vertical_offset)..canvas.height,
                (canvas.height - 1) - self.vertical_offset,
                self.horizontal_offset - time.len() + 1,
            ),
            TimestampLocation::Bottom => (
                0..self.vertical_offset,
                self.vertical_offset,
                self.horizontal_offset,
            ),
        };

        stem_rows.for_each(|i| {
            canvas.update_row(i, |row| {
                let mut s = row.to_owned();
                s.replace_range(self.horizontal_offset..(self.horizontal_offset + 1), "|");
                s
            });
        });

        canvas.update_row(timestamp_row, |row| {
            let mut s = row.to_owned();
            s.replace_range(
                timestamp_horizontal_offset..(timestamp_horizontal_offset + time.len()),
                &time,
            );
            s
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timemarker_render_top_single_stem() {
        let mut canvas = Canvas::new(25, 5);
        let time_marker = TimeMarker {
            horizontal_offset: 20,
            timestamp_location: TimestampLocation::Top,
            timestamp: 0,
            vertical_offset: 1,
        };

        time_marker.render(&mut canvas);

        let rendered = format!("\n{}", canvas);
        assert_eq!(rendered, "
                         
                         
                         
  1970-01-01 00:00:00    
                    |    
");
    }

    #[test]
    fn timemarker_render_bottom_single_stem() {
        let mut canvas = Canvas::new(25, 5);
        let time_marker = TimeMarker {
            horizontal_offset: 0,
            timestamp_location: TimestampLocation::Bottom,
            timestamp: 0,
            vertical_offset: 1,
        };

        time_marker.render(&mut canvas);
        let rendered = format!("\n{}", canvas);
        assert_eq!(rendered, "
|                        
1970-01-01 00:00:00      
                         
                         
                         
");
    }

    #[test]
    fn timemarker_render_top_three_stems() {
        let mut canvas = Canvas::new(80, 5);
        let time_marker = TimeMarker {
            horizontal_offset: 19,
            timestamp_location: TimestampLocation::Top,
            timestamp: 0,
            vertical_offset: 1,
        };
        let time_marker2 = TimeMarker {
            horizontal_offset: 40,
            timestamp_location: TimestampLocation::Top,
            timestamp: 1000,
            vertical_offset: 2,
        };
        let time_marker3 = TimeMarker {
            horizontal_offset: 60,
            timestamp_location: TimestampLocation::Top,
            timestamp: 2000,
            vertical_offset: 3,
        };

        [time_marker, time_marker2, time_marker3]
            .iter()
            .for_each(|marker| {
                marker.render(&mut canvas);
            });

        let rendered = format!("\n{}", canvas);
        assert_eq!(rendered, "
                                                                                
                                          1970-01-01 00:33:20                   
                      1970-01-01 00:16:40                   |                   
 1970-01-01 00:00:00                    |                   |                   
                   |                    |                   |                   
");
    }

    #[test]
    fn timemarker_render_bottom_three_stems() {
        let mut canvas = Canvas::new(80, 5);
        let time_marker = TimeMarker {
            horizontal_offset: 19,
            timestamp_location: TimestampLocation::Bottom,
            timestamp: 0,
            vertical_offset: 3,
        };
        let time_marker2 = TimeMarker {
            horizontal_offset: 40,
            timestamp_location: TimestampLocation::Bottom,
            timestamp: 1000,
            vertical_offset: 2,
        };
        let time_marker3 = TimeMarker {
            horizontal_offset: 60,
            timestamp_location: TimestampLocation::Bottom,
            timestamp: 2000,
            vertical_offset: 1,
        };

        [time_marker, time_marker2, time_marker3]
            .iter()
            .for_each(|marker| {
                marker.render(&mut canvas);
            });

        let rendered = format!("\n{}", canvas);
        assert_eq!(rendered, "
                   |                    |                   |                   
                   |                    |                   1970-01-01 00:33:20 
                   |                    1970-01-01 00:16:40                     
                   1970-01-01 00:00:00                                          
                                                                                
");
    }
}
