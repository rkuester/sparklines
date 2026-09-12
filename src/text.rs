use std::fmt;

use crate::core::{SparkStyle, Sparkline};

/// Block characters U+2581 through U+2588 (lower one-eighth to full block).
const BLOCKS: [char; 8] = [
    '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}',
];

const UPPER_HALF: char = '\u{2580}';
const LOWER_HALF: char = '\u{2584}';

impl fmt::Display for Sparkline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.spark_style() {
            SparkStyle::Line | SparkStyle::Bar => write_blocks(self, f),
            SparkStyle::WinLoss => write_winloss(self, f),
        }
    }
}

fn write_blocks(sparkline: &Sparkline, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let Some((min, max)) = sparkline.data_range() else {
        return Ok(());
    };
    let span = max - min;

    for val in sparkline.data() {
        match val {
            Some(v) => {
                let idx = if span == 0.0 {
                    3 // mid-height when all values are equal
                } else {
                    let normalized = (v - min) / span;
                    // Map [0.0, 1.0] to block indices [0, 7]
                    ((normalized * 7.0).round() as usize).min(7)
                };
                f.write_str(BLOCKS[idx].encode_utf8(&mut [0u8; 4]))?;
            }
            None => f.write_str(" ")?,
        }
    }
    Ok(())
}

fn write_winloss(sparkline: &Sparkline, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let threshold = sparkline.reference_value().unwrap_or(0.0);

    for val in sparkline.data() {
        match val {
            Some(v) if *v > threshold => {
                f.write_str(UPPER_HALF.encode_utf8(&mut [0u8; 4]))?;
            }
            Some(v) if *v < threshold => {
                f.write_str(LOWER_HALF.encode_utf8(&mut [0u8; 4]))?;
            }
            Some(_) => f.write_str(" ")?, // exactly at threshold
            None => f.write_str(" ")?,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascending_values_produce_ascending_blocks() {
        let s = Sparkline::from_values(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
        let text = s.to_string();
        let chars: Vec<char> = text.chars().collect();
        assert_eq!(chars.len(), 8);
        // Each character should be >= the previous one
        for pair in chars.windows(2) {
            assert!(
                pair[0] <= pair[1],
                "{:?} should be <= {:?}",
                pair[0],
                pair[1]
            );
        }
        assert_eq!(chars[0], BLOCKS[0]);
        assert_eq!(chars[7], BLOCKS[7]);
    }

    #[test]
    fn constant_values_use_mid_block() {
        let s = Sparkline::from_values(&[5.0, 5.0, 5.0]);
        let text = s.to_string();
        // All equal -> index 3 (mid-height)
        assert!(text.chars().all(|c| c == BLOCKS[3]));
    }

    #[test]
    fn gaps_render_as_spaces() {
        let s = Sparkline::new(vec![Some(1.0), None, Some(3.0)]);
        let chars: Vec<char> = s.to_string().chars().collect();
        assert_eq!(chars[1], ' ');
    }

    #[test]
    fn winloss_above_below_threshold() {
        let s = Sparkline::from_values(&[1.0, -1.0, 0.0]).style(SparkStyle::WinLoss);
        let chars: Vec<char> = s.to_string().chars().collect();
        assert_eq!(chars[0], UPPER_HALF);
        assert_eq!(chars[1], LOWER_HALF);
        assert_eq!(chars[2], ' '); // exactly at threshold
    }

    #[test]
    fn winloss_custom_threshold() {
        let s = Sparkline::from_values(&[5.0, 3.0, 4.0])
            .style(SparkStyle::WinLoss)
            .reference_line(4.0);
        let chars: Vec<char> = s.to_string().chars().collect();
        assert_eq!(chars[0], UPPER_HALF); // 5 > 4
        assert_eq!(chars[1], LOWER_HALF); // 3 < 4
        assert_eq!(chars[2], ' '); // 4 == 4
    }

    #[test]
    fn empty_data_produces_empty_string() {
        let s = Sparkline::from_values(&[]);
        assert_eq!(s.to_string(), "");
    }

    #[test]
    fn bar_style_renders_same_as_line() {
        let data = &[1.0, 4.0, 2.0, 7.0, 3.0];
        let line = Sparkline::from_values(data)
            .style(SparkStyle::Line)
            .to_string();
        let bar = Sparkline::from_values(data)
            .style(SparkStyle::Bar)
            .to_string();
        assert_eq!(line, bar);
    }
}
