use crate::model::*;
use mpd::Song;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::Duration;

pub fn make_progress_bar<'a>(ratio: f64) -> LineGauge<'a> {
    let progress_bar = LineGauge::default()
        .block(Block::bordered().title("Progress"))
        .filled_style(Style::default().bg(Color::White).fg(Color::Black))
        .ratio(ratio);
    return progress_bar;
}

pub fn make_queue<'a>(model: &Model) -> (Table<'a>, TableState) {
    let rows: Vec<Row> = model
        .queue
        .contents
        .iter()
        .map(|song| {
            Row::new(vec![
                Cell::from(song.artist.clone().unwrap_or("".to_string())),
                Cell::from(song.title.clone().unwrap_or("".to_string())),
                Cell::from(
                    song.duration
                        .unwrap_or(Duration::new(0, 0))
                        .as_secs()
                        .to_string(),
                ),
            ])
        })
        .collect();
    let table = Table::new(rows, vec![30, 30, 30])
        .highlight_style(Style::default().fg(Color::Red));

    (
        table,
        TableState::new()
            .with_offset(model.queue.offset)
            .with_selected(model.queue.selection),
    )
}

pub fn render(model: &Model, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Max(3)])
        .split(frame.size());

    let (table, mut table_state) = make_queue(model);
    frame.render_stateful_widget(table, layout[0], &mut table_state);

    let ratio: f64 = match (model.status.elapsed, model.status.duration) {
        (Some(e), Some(t)) => e.as_secs_f64() / t.as_secs_f64(),
        _ => 0 as f64,
    };

    frame.render_widget(make_progress_bar(ratio), layout[1])
}
