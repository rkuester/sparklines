use sparkline::{Marker, SparkStyle, Sparkline};
use tiny_skia::{Color, Pixmap};

fn main() {
    let mut pixmap = Pixmap::new(800, 600).expect("failed to create pixmap");

    let data = [1.0, 4.0, 2.0, 8.0, 5.0, 7.0, 2.0, 6.0, 3.0, 9.0, 1.0, 5.0];

    // Line sparkline with area fill (top third)
    let line = Sparkline::from_values(&data)
        .style(SparkStyle::Line)
        .marker(Marker::Min)
        .marker(Marker::Max)
        .reference_line(5.0);
    line.pixel()
        .background_color(Color::from_rgba8(30, 30, 30, 255))
        .line_color(Color::from_rgba8(0, 200, 255, 255))
        .fill_color(Color::from_rgba8(0, 200, 255, 60))
        .marker_color(Color::from_rgba8(255, 80, 80, 255))
        .reference_line_color(Color::from_rgba8(255, 255, 0, 150))
        .stroke_width(3.0)
        .marker_radius(5.0)
        .render(&mut pixmap, 20.0, 20.0, 760.0, 170.0);

    // Bar sparkline (middle third)
    let bar = Sparkline::from_values(&data)
        .style(SparkStyle::Bar)
        .marker(Marker::Max);
    bar.pixel()
        .background_color(Color::from_rgba8(30, 30, 30, 255))
        .line_color(Color::from_rgba8(80, 200, 80, 255))
        .marker_color(Color::from_rgba8(255, 0, 255, 255))
        .bar_gap(3.0)
        .marker_radius(4.0)
        .render(&mut pixmap, 20.0, 210.0, 760.0, 170.0);

    // Win/loss sparkline (bottom third)
    let wl_data = [1.0, -2.0, 3.0, -1.0, 0.0, 4.0, -3.0, 2.0, -1.0, 5.0];
    let wl = Sparkline::from_values(&wl_data).style(SparkStyle::WinLoss);
    wl.pixel()
        .background_color(Color::from_rgba8(30, 30, 30, 255))
        .line_color(Color::from_rgba8(100, 140, 255, 255))
        .bar_gap(3.0)
        .render(&mut pixmap, 20.0, 400.0, 760.0, 170.0);

    pixmap.save_png("sparkline_demo.png").expect("failed to save PNG");
    println!("Saved sparkline_demo.png");
}
