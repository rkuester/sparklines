use std::io;

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::Block,
    Frame,
};
use sparkline::{Marker, SparkStyle, Sparkline};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(draw)?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                break;
            }
        }
    }

    ratatui::restore();
    Ok(())
}

fn draw(frame: &mut Frame) {
    let data = [1.0, 4.0, 2.0, 8.0, 5.0, 7.0, 2.0, 6.0, 3.0, 9.0, 1.0, 5.0];

    let chunks = Layout::vertical([
        Constraint::Percentage(33),
        Constraint::Percentage(33),
        Constraint::Percentage(34),
    ])
    .split(frame.area());

    // Line style
    let line_spark = Sparkline::from_values(&data)
        .style(SparkStyle::Line)
        .marker(Marker::Min)
        .marker(Marker::Max)
        .reference_line(5.0);
    let line_widget = line_spark
        .tui()
        .data_style(Style::new().fg(Color::Cyan))
        .marker_style(Style::new().fg(Color::Red))
        .reference_line_style(Style::new().fg(Color::Yellow))
        .block(Block::bordered().title("Line"));
    frame.render_widget(line_widget, chunks[0]);

    // Bar style
    let bar_spark = Sparkline::from_values(&data)
        .style(SparkStyle::Bar)
        .marker(Marker::Max);
    let bar_widget = bar_spark
        .tui()
        .data_style(Style::new().fg(Color::Green))
        .marker_style(Style::new().fg(Color::Magenta))
        .block(Block::bordered().title("Bar"));
    frame.render_widget(bar_widget, chunks[1]);

    // Win/Loss style
    let wl_data = [1.0, -2.0, 3.0, -1.0, 0.0, 4.0, -3.0, 2.0, -1.0, 5.0];
    let wl_spark = Sparkline::from_values(&wl_data).style(SparkStyle::WinLoss);
    let wl_widget = wl_spark
        .tui()
        .data_style(Style::new().fg(Color::Blue))
        .block(Block::bordered().title("Win/Loss"));
    frame.render_widget(wl_widget, chunks[2]);
}
