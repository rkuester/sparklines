use tiny_skia::{
    Color, FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Rect, Stroke, Transform,
};

use crate::core::{Marker, SparkStyle, Sparkline};

pub struct PixelSparkline<'a> {
    sparkline: &'a Sparkline,
    line_color: Color,
    fill_color: Option<Color>,
    min_marker_color: Color,
    max_marker_color: Color,
    current_marker_color: Color,
    reference_line_color: Color,
    background_color: Option<Color>,
    stroke_width: f32,
    marker_radius: f32,
    bar_gap: f32,
}

impl<'a> PixelSparkline<'a> {
    pub fn new(sparkline: &'a Sparkline) -> Self {
        Self {
            sparkline,
            line_color: Color::from_rgba8(0, 128, 255, 255),
            fill_color: None,
            min_marker_color: Color::from_rgba8(64, 128, 255, 255),
            max_marker_color: Color::from_rgba8(255, 64, 64, 255),
            current_marker_color: Color::from_rgba8(255, 255, 255, 255),
            reference_line_color: Color::from_rgba8(128, 128, 128, 200),
            background_color: None,
            stroke_width: 2.0,
            marker_radius: 4.0,
            bar_gap: 1.0,
        }
    }

    pub fn line_color(mut self, color: Color) -> Self {
        self.line_color = color;
        self
    }

    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    pub fn marker_color(mut self, color: Color) -> Self {
        self.min_marker_color = color;
        self.max_marker_color = color;
        self.current_marker_color = color;
        self
    }

    pub fn min_marker_color(mut self, color: Color) -> Self {
        self.min_marker_color = color;
        self
    }

    pub fn max_marker_color(mut self, color: Color) -> Self {
        self.max_marker_color = color;
        self
    }

    pub fn current_marker_color(
        mut self,
        color: Color,
    ) -> Self {
        self.current_marker_color = color;
        self
    }

    pub fn reference_line_color(mut self, color: Color) -> Self {
        self.reference_line_color = color;
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    pub fn marker_radius(mut self, radius: f32) -> Self {
        self.marker_radius = radius;
        self
    }

    pub fn bar_gap(mut self, gap: f32) -> Self {
        self.bar_gap = gap;
        self
    }

    pub fn render(
        &self,
        pixmap: &mut Pixmap,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        // Background fill
        if let Some(bg) = self.background_color {
            if let Some(rect) = Rect::from_xywh(x, y, width, height) {
                let mut paint = Paint::default();
                paint.set_color(bg);
                pixmap.fill_rect(rect, &paint, Transform::identity(), None);
            }
        }

        let data = self.sparkline.data();
        if data.is_empty() {
            return;
        }

        match self.sparkline.spark_style() {
            SparkStyle::Line => self.render_line(pixmap, x, y, width, height),
            SparkStyle::Bar => self.render_bars(pixmap, x, y, width, height),
            SparkStyle::WinLoss => self.render_winloss(pixmap, x, y, width, height),
        }

        // Reference line
        if let Some(rv) = self.sparkline.reference_value() {
            self.render_reference_line(pixmap, x, y, width, height, rv);
        }

        // Markers
        self.render_markers(pixmap, x, y, width, height);
    }
}

// -- Rendering internals --

impl PixelSparkline<'_> {
    /// Map a data value to a y-pixel coordinate (top = max, bottom = min).
    fn value_to_y(&self, v: f64, min: f64, span: f64, y: f32, height: f32) -> f32 {
        let padding = self.stroke_width;
        let usable = height - 2.0 * padding;
        if span == 0.0 {
            y + height / 2.0
        } else {
            let normalized = (v - min) / span;
            y + padding + usable * (1.0 - normalized as f32)
        }
    }

    /// X coordinate for data point at index `i`.
    fn index_to_x(&self, i: usize, n: usize, x: f32, width: f32) -> f32 {
        if n <= 1 {
            x + width / 2.0
        } else {
            x + (i as f32 / (n - 1) as f32) * width
        }
    }

    /// Data range expanded to include the reference line.
    ///
    /// When the reference value falls outside the data range, the
    /// range is stretched so the reference line sits at 80% from
    /// the bottom (20% headroom above or below).
    fn effective_range(&self) -> Option<(f64, f64)> {
        let (mut lo, mut hi) = self.sparkline.data_range()?;
        if let Some(rv) = self.sparkline.reference_value() {
            if rv > hi {
                // Expand top so rv lands at 80% from bottom.
                hi = lo + (rv - lo) / 0.8;
            } else if rv < lo {
                // Expand bottom so rv lands at 20% from bottom.
                lo = hi - (hi - rv) / 0.8;
            }
        }
        Some((lo, hi))
    }

    fn render_line(
        &self,
        pixmap: &mut Pixmap,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let data = self.sparkline.data();
        let Some((min, max)) = self.effective_range() else {
            return;
        };
        let span = max - min;
        let n = data.len();

        // Build polyline path, breaking at gaps
        let mut pb = PathBuilder::new();
        let mut in_segment = false;

        // Also build fill path if fill_color is set
        let mut fill_pb = self.fill_color.map(|_| PathBuilder::new());
        let mut fill_segment_start_x = 0.0f32;

        for (i, val) in data.iter().enumerate() {
            let px = self.index_to_x(i, n, x, width);
            match val {
                Some(v) => {
                    let py = self.value_to_y(*v, min, span, y, height);
                    if in_segment {
                        pb.line_to(px, py);
                        if let Some(ref mut fpb) = fill_pb {
                            fpb.line_to(px, py);
                        }
                    } else {
                        pb.move_to(px, py);
                        if let Some(ref mut fpb) = fill_pb {
                            fpb.move_to(px, y + height);
                            fpb.line_to(px, py);
                            fill_segment_start_x = px;
                        }
                        in_segment = true;
                    }
                }
                None => {
                    if in_segment {
                        // Close fill segment at the previous point
                        if let Some(ref mut fpb) = fill_pb {
                            let prev_x = self.index_to_x(i - 1, n, x, width);
                            fpb.line_to(prev_x, y + height);
                            fpb.line_to(fill_segment_start_x, y + height);
                            fpb.close();
                        }
                    }
                    in_segment = false;
                }
            }
        }

        // Close final fill segment
        if in_segment {
            if let Some(ref mut fpb) = fill_pb {
                if let Some(last_i) = data.iter().rposition(|v| v.is_some()) {
                    let last_x = self.index_to_x(last_i, n, x, width);
                    fpb.line_to(last_x, y + height);
                    fpb.line_to(fill_segment_start_x, y + height);
                    fpb.close();
                }
            }
        }

        // Draw fill
        if let (Some(fill_color), Some(fpb)) = (self.fill_color, fill_pb) {
            if let Some(path) = fpb.finish() {
                let mut paint = Paint::default();
                paint.set_color(fill_color);
                paint.anti_alias = true;
                pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
            }
        }

        // Draw stroke
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(self.line_color);
            paint.anti_alias = true;

            let mut stroke = Stroke::default();
            stroke.width = self.stroke_width;
            stroke.line_cap = LineCap::Round;
            stroke.line_join = LineJoin::Round;

            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    fn render_bars(
        &self,
        pixmap: &mut Pixmap,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let data = self.sparkline.data();
        let Some((min, max)) = self.effective_range() else {
            return;
        };
        let span = max - min;
        let n = data.len();
        let bar_width = ((width / n as f32) - self.bar_gap).max(1.0);
        let step = width / n as f32;

        let mut paint = Paint::default();
        paint.set_color(self.line_color);
        paint.anti_alias = true;

        let baseline_y = y + height;

        for (i, val) in data.iter().enumerate() {
            let Some(v) = val else { continue };
            let top_y = self.value_to_y(*v, min, span, y, height);
            let bar_x = x + i as f32 * step + self.bar_gap / 2.0;
            let bar_h = (baseline_y - top_y).max(1.0);

            if let Some(rect) = Rect::from_xywh(bar_x, top_y, bar_width, bar_h) {
                pixmap.fill_rect(rect, &paint, Transform::identity(), None);
            }
        }
    }

    fn render_winloss(
        &self,
        pixmap: &mut Pixmap,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let data = self.sparkline.data();
        let threshold = self.sparkline.reference_value().unwrap_or(0.0);
        let n = data.len();
        let bar_width = ((width / n as f32) - self.bar_gap).max(1.0);
        let step = width / n as f32;
        let mid_y = y + height / 2.0;
        let half_h = height / 2.0;

        let mut paint = Paint::default();
        paint.set_color(self.line_color);
        paint.anti_alias = true;

        for (i, val) in data.iter().enumerate() {
            let Some(v) = val else { continue };
            let bar_x = x + i as f32 * step + self.bar_gap / 2.0;

            let rect = if *v > threshold {
                // Win: bar in upper half
                Rect::from_xywh(bar_x, mid_y - half_h, bar_width, half_h)
            } else if *v < threshold {
                // Loss: bar in lower half
                Rect::from_xywh(bar_x, mid_y, bar_width, half_h)
            } else {
                continue;
            };

            if let Some(rect) = rect {
                pixmap.fill_rect(rect, &paint, Transform::identity(), None);
            }
        }
    }

    fn render_reference_line(
        &self,
        pixmap: &mut Pixmap,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        value: f64,
    ) {
        let Some((min, max)) = self.effective_range() else {
            return;
        };
        let span = max - min;
        let ry = self.value_to_y(value, min, span, y, height);

        let mut pb = PathBuilder::new();
        pb.move_to(x, ry);
        pb.line_to(x + width, ry);

        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(self.reference_line_color);
            paint.anti_alias = true;

            let mut stroke = Stroke::default();
            stroke.width = 1.0;

            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    fn render_markers(
        &self,
        pixmap: &mut Pixmap,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let Some((min, max)) = self.effective_range() else {
            return;
        };
        let span = max - min;
        let data = self.sparkline.data();
        let n = data.len();

        let mut paint = Paint::default();
        paint.anti_alias = true;

        for marker in self.sparkline.markers() {
            let (idx, color) = match marker {
                Marker::Min => (
                    self.sparkline.min_index(),
                    self.min_marker_color,
                ),
                Marker::Max => (
                    self.sparkline.max_index(),
                    self.max_marker_color,
                ),
                Marker::Current => (
                    self.sparkline.current_index(),
                    self.current_marker_color,
                ),
            };
            paint.set_color(color);

            let Some(idx) = idx else { continue };
            let Some(v) = data[idx] else { continue };

            let cx = self.index_to_x(idx, n, x, width);
            let cy = self.value_to_y(v, min, span, y, height);

            if let Some(circle) = circle_path(cx, cy, self.marker_radius) {
                pixmap.fill_path(
                    &circle,
                    &paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        }
    }
}

/// Approximate a circle with four cubic bezier arcs.
fn circle_path(cx: f32, cy: f32, r: f32) -> Option<Path> {
    // kappa = 4 * (sqrt(2) - 1) / 3 -- standard bezier circle approximation
    const KAPPA: f32 = 0.552_284_75;
    let k = r * KAPPA;

    let mut pb = PathBuilder::new();
    pb.move_to(cx + r, cy);
    pb.cubic_to(cx + r, cy + k, cx + k, cy + r, cx, cy + r);
    pb.cubic_to(cx - k, cy + r, cx - r, cy + k, cx - r, cy);
    pb.cubic_to(cx - r, cy - k, cx - k, cy - r, cx, cy - r);
    pb.cubic_to(cx + k, cy - r, cx + r, cy - k, cx + r, cy);
    pb.close();
    pb.finish()
}
