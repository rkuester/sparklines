use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::style::Style;
use ratatui_core::widgets::Widget;
use ratatui_widgets::block::Block;

use crate::core::{Marker, SparkStyle, Sparkline};

/// Block characters U+2581 through U+2588 (lower one-eighth to full block).
const BLOCKS: [char; 8] = [
    '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}',
    '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}',
];

/// Horizontal line character for reference line rendering.
const HLINE: char = '\u{2500}';

pub struct TuiSparkline<'a> {
    sparkline: &'a Sparkline,
    style: Style,
    data_style: Style,
    marker_style: Style,
    reference_line_style: Style,
    absent_value_style: Style,
    block: Option<Block<'a>>,
}

impl<'a> TuiSparkline<'a> {
    pub fn new(sparkline: &'a Sparkline) -> Self {
        Self {
            sparkline,
            style: Style::default(),
            data_style: Style::default(),
            marker_style: Style::default(),
            reference_line_style: Style::default(),
            absent_value_style: Style::default(),
            block: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn data_style(mut self, style: Style) -> Self {
        self.data_style = style;
        self
    }

    pub fn marker_style(mut self, style: Style) -> Self {
        self.marker_style = style;
        self
    }

    pub fn reference_line_style(mut self, style: Style) -> Self {
        self.reference_line_style = style;
        self
    }

    pub fn absent_value_style(mut self, style: Style) -> Self {
        self.absent_value_style = style;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for TuiSparkline<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);

        let inner = if let Some(block) = self.block.take() {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if inner.is_empty() {
            return;
        }

        let data = self.sparkline.data();
        if data.is_empty() {
            return;
        }

        let height = inner.height as f64;

        match self.sparkline.spark_style() {
            SparkStyle::Line | SparkStyle::Bar => {
                self.render_bars(data, &inner, height, buf);
            }
            SparkStyle::WinLoss => {
                self.render_winloss(data, &inner, height, buf);
            }
        }
    }
}

// -- Rendering internals --

impl TuiSparkline<'_> {
    fn render_bars(
        &self,
        data: &[Option<f64>],
        inner: &Rect,
        height: f64,
        buf: &mut Buffer,
    ) {
        let Some((min, max)) = self.sparkline.data_range() else {
            return;
        };
        let span = max - min;
        let min_idx = self.sparkline.min_index();
        let max_idx = self.sparkline.max_index();
        let has_min_marker = self.sparkline.markers().contains(&Marker::Min);
        let has_max_marker = self.sparkline.markers().contains(&Marker::Max);

        // Reference line y-position (row from top, fractional)
        let ref_row = self.sparkline.reference_value().and_then(|rv| {
            if span == 0.0 {
                return None;
            }
            let normalized = (rv - min) / span;
            // Invert: row 0 is top, row (height-1) is bottom
            let row = (height - 1.0) * (1.0 - normalized);
            Some(row.round() as u16)
        });

        for (i, val) in data.iter().enumerate() {
            let x = inner.x + i as u16;
            if x >= inner.right() {
                break;
            }

            match val {
                Some(v) => {
                    let bar_eighths = if span == 0.0 {
                        // All values equal -- half-height bar
                        (height * 4.0) as u16
                    } else {
                        let normalized = (v - min) / span;
                        // At least 1 eighth for the minimum value
                        ((normalized * height * 8.0).round() as u16).max(1)
                    };

                    let full_rows = (bar_eighths / 8) as u16;
                    let remainder = (bar_eighths % 8) as usize;

                    // Fill from bottom up
                    for row in 0..full_rows {
                        let y = inner.bottom() - 1 - row;
                        if y >= inner.top() {
                            if let Some(cell) = buf.cell_mut((x, y)) {
                                cell.set_symbol(
                                    BLOCKS[7].encode_utf8(&mut [0u8; 4]),
                                );
                                cell.set_style(self.data_style);
                            }
                        }
                    }

                    // Partial block on top
                    if remainder > 0 {
                        let y = inner.bottom() - 1 - full_rows;
                        if y >= inner.top() {
                            if let Some(cell) = buf.cell_mut((x, y)) {
                                cell.set_symbol(
                                    BLOCKS[remainder - 1]
                                        .encode_utf8(&mut [0u8; 4]),
                                );
                                cell.set_style(self.data_style);
                            }
                        }
                    }

                    // Overlay marker style on this column
                    let is_marker = (has_min_marker && min_idx == Some(i))
                        || (has_max_marker && max_idx == Some(i));
                    if is_marker {
                        for y in inner.top()..inner.bottom() {
                            if let Some(cell) = buf.cell_mut((x, y)) {
                                cell.set_style(self.marker_style);
                            }
                        }
                    }
                }
                None => {
                    for y in inner.top()..inner.bottom() {
                        if let Some(cell) = buf.cell_mut((x, y)) {
                            cell.set_style(self.absent_value_style);
                        }
                    }
                }
            }
        }

        // Reference line
        if let Some(row) = ref_row {
            let y = inner.y + row;
            if y >= inner.top() && y < inner.bottom() {
                let end_x = (inner.x + data.len() as u16).min(inner.right());
                for x in inner.x..end_x {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol(HLINE.encode_utf8(&mut [0u8; 4]));
                        cell.set_style(self.reference_line_style);
                    }
                }
            }
        }
    }

    fn render_winloss(
        &self,
        data: &[Option<f64>],
        inner: &Rect,
        height: f64,
        buf: &mut Buffer,
    ) {
        let threshold = self.sparkline.reference_value().unwrap_or(0.0);
        let midpoint = inner.y + (height / 2.0).floor() as u16;
        let half = (height / 2.0).floor() as u16;

        for (i, val) in data.iter().enumerate() {
            let x = inner.x + i as u16;
            if x >= inner.right() {
                break;
            }

            match val {
                Some(v) if *v > threshold => {
                    // Fill upper half (midpoint-half .. midpoint)
                    let top = midpoint.saturating_sub(half);
                    for y in top..midpoint {
                        if y >= inner.top() && y < inner.bottom() {
                            if let Some(cell) = buf.cell_mut((x, y)) {
                                cell.set_symbol(
                                    BLOCKS[7].encode_utf8(&mut [0u8; 4]),
                                );
                                cell.set_style(self.data_style);
                            }
                        }
                    }
                }
                Some(v) if *v < threshold => {
                    // Fill lower half (midpoint .. midpoint+half)
                    let bot = (midpoint + half).min(inner.bottom());
                    for y in midpoint..bot {
                        if y >= inner.top() && y < inner.bottom() {
                            if let Some(cell) = buf.cell_mut((x, y)) {
                                cell.set_symbol(
                                    BLOCKS[7].encode_utf8(&mut [0u8; 4]),
                                );
                                cell.set_style(self.data_style);
                            }
                        }
                    }
                }
                _ => {
                    // At threshold or absent -- leave blank
                    for y in inner.top()..inner.bottom() {
                        if let Some(cell) = buf.cell_mut((x, y)) {
                            cell.set_style(self.absent_value_style);
                        }
                    }
                }
            }
        }
    }
}
