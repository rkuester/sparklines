#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SparkStyle {
    #[default]
    Line,
    Bar,
    WinLoss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    Min,
    Max,
    /// Last present value in the data.
    Current,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sparkline {
    data: Vec<Option<f64>>,
    style: SparkStyle,
    markers: Vec<Marker>,
    reference_line: Option<f64>,
}

impl Sparkline {
    /// Create from dense data (no gaps).
    pub fn from_values(values: &[f64]) -> Self {
        Self {
            data: values.iter().copied().map(Some).collect(),
            ..Default::default()
        }
    }

    /// Create from sparse data with explicit gaps.
    pub fn new(data: Vec<Option<f64>>) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }

    pub fn style(mut self, style: SparkStyle) -> Self {
        self.style = style;
        self
    }

    /// Add a marker (deduplicated by variant).
    pub fn marker(mut self, marker: Marker) -> Self {
        if !self.markers.contains(&marker) {
            self.markers.push(marker);
        }
        self
    }

    pub fn reference_line(mut self, value: f64) -> Self {
        self.reference_line = Some(value);
        self
    }

    // -- Accessors --

    pub fn data(&self) -> &[Option<f64>] {
        &self.data
    }

    pub fn spark_style(&self) -> SparkStyle {
        self.style
    }

    pub fn markers(&self) -> &[Marker] {
        &self.markers
    }

    pub fn reference_value(&self) -> Option<f64> {
        self.reference_line
    }

    /// Index of the minimum present value, or `None` if data is empty.
    pub fn min_index(&self) -> Option<usize> {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.map(|val| (i, val)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(i, _)| i)
    }

    /// Index of the maximum present value, or `None` if data is empty.
    pub fn max_index(&self) -> Option<usize> {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.map(|val| (i, val)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(i, _)| i)
    }

    /// Index of the last present value, or `None` if data is empty.
    pub fn current_index(&self) -> Option<usize> {
        self.data
            .iter()
            .rposition(|v| v.is_some())
    }

    /// Returns `(min, max)` of present values, or `None` if no data.
    pub fn data_range(&self) -> Option<(f64, f64)> {
        let mut iter = self.data.iter().filter_map(|v| *v);
        let first = iter.next()?;
        iter.fold((first, first), |(lo, hi), v| (lo.min(v), hi.max(v)))
            .into()
    }

    // -- Feature-gated bridge methods --

    #[cfg(feature = "tui")]
    pub fn tui(&self) -> crate::tui::TuiSparkline<'_> {
        crate::tui::TuiSparkline::new(self)
    }

    #[cfg(feature = "pixels")]
    pub fn pixel(&self) -> crate::pixel::PixelSparkline<'_> {
        crate::pixel::PixelSparkline::new(self)
    }
}
