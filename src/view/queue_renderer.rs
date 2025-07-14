use super::layout::queue_layout::QueueLayout;
use super::layout::InoriLayout;
use super::search_renderer::make_search_box;
use super::Theme;
use crate::model::proto::Searchable;
use crate::model::*;
use crate::util::{format_time, song_album};
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::Duration;

use super::status_renderer::render_status;

pub fn make_queue<'a>(model: &mut Model, theme: &Theme) -> Table<'a> {
    let rows: Vec<Row> = model
        .queue
        .contents()
        .map(|song| {
            Row::new(vec![
                Cell::from(song.title.clone().unwrap_or("".to_string())),
                Cell::from(
                    Text::from(
                        song.artist.clone().unwrap_or("Unknown Artist".into()),
                    )
                    .style(theme.status_artist)
                    .left_aligned(),
                ),
                Cell::from(
                    Text::from(
                        song_album(song)
                            .cloned()
                            .unwrap_or("Unknown Album".into()),
                    )
                    .style(theme.field_album)
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
        vec![Percentage(50), Percentage(30), Percentage(20), Min(7)],
    )
    .row_highlight_style(theme.item_highlight_active)
    .block(Block::bordered().title("Queue"));

    table
}

pub fn render(model: &mut Model, frame: &mut Frame, theme: &Theme) {
    let layout = QueueLayout::new(frame.area(), model);
    render_status(model, frame, layout.header, theme);
    let table = make_queue(model, theme);

    if let Some(a) = layout.search {
        frame.render_widget(
            make_search_box(
                &model.queue.search.query,
                matches!(model.state, State::Searching),
                theme,
            ),
            a,
        );
    }

    frame.render_stateful_widget(table, layout.queue, &mut model.queue.state);

    let ratio: f64 = match (model.status.elapsed, model.status.duration) {
        (Some(e), Some(t)) => e.as_secs_f64() / t.as_secs_f64(),
        _ => 0 as f64,
    };

    frame.render_widget(
        LineGauge::default()
            .block(Block::bordered().title("Progress"))
            .filled_style(theme.progress_bar_filled)
            .unfilled_style(theme.progress_bar_unfilled)
            .line_set(symbols::line::THICK)
            .ratio(ratio),
        layout.progress,
    );
}
