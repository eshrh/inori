use super::Theme;
use crate::model::*;
use crate::util::{format_time, song_album};
use mpd::Song;
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::Duration;

use super::status_renderer::render_status;

pub fn make_progress_bar<'a>(ratio: f64) -> LineGauge<'a> {
    let progress_bar = LineGauge::default()
        .block(Block::bordered().title("Progress"))
        .filled_style(
            Style::default()
                .fg(Color::LightYellow)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .unfilled_style(Style::default().fg(Color::Black))
        .line_set(symbols::line::THICK)
        .ratio(ratio);
    return progress_bar;
}

pub fn make_queue<'a>(model: &mut Model, theme: &Theme) -> Table<'a> {
    let rows: Vec<Row> = model
        .queue
        .contents
        .iter()
        .map(|song| {
            Row::new(vec![
                Cell::from(song.title.clone().unwrap_or("".to_string())),
                Cell::from(
                    Text::from(
                        song.artist.clone().unwrap_or("Unknown Artist".into()),
                    )
                    .left_aligned(),
                ),
                Cell::from(
                    Text::from(
                        song_album(song)
                            .cloned()
                            .unwrap_or("Unknown Album".into()),
                    )
                    .left_aligned(),
                ),
                Cell::from(
                    Text::from(format_time(
                        song.duration.unwrap_or(Duration::new(0, 0)),
                    ))
                    .left_aligned(),
                ),
            ])
            .add_modifier(
                if song
                    .place
                    .is_some_and(|s| model.status.song.is_some_and(|o| s == o))
                {
                    Modifier::ITALIC | Modifier::BOLD
                } else {
                    Modifier::empty()
                },
            )
        })
        .collect();
    let table = Table::new(
        rows,
        vec![Percentage(50), Percentage(30), Percentage(20), Min(5)],
    )
    .highlight_style(theme.item_highlight_active)
    .block(Block::bordered().title("Queue"));

    table
}

pub fn render(model: &mut Model, frame: &mut Frame, theme: &Theme) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Max(4), Min(1), Max(3)])
        .split(frame.size());

    render_status(model, frame, layout[0], theme);

    let table = make_queue(model, theme);
    frame.render_stateful_widget(table, layout[1], &mut model.queue.state);

    let ratio: f64 = match (model.status.elapsed, model.status.duration) {
        (Some(e), Some(t)) => e.as_secs_f64() / t.as_secs_f64(),
        _ => 0 as f64,
    };

    frame.render_widget(make_progress_bar(ratio), layout[2])
}
