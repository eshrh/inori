use super::Theme;
use crate::model::Model;
use crate::util::*;
use mpd::State::*;
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;
use ratatui::style::Styled;
use ratatui::widgets::*;

pub fn format_status<'a>(state: bool, on: &'a str, off: &'a str) -> &'a str {
    if state {
        on
    } else {
        off
    }
}

pub fn render_status(
    model: &mut Model,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let w = Table::new::<Vec<Row>, Vec<Constraint>>(
        vec![
            Row::new(vec![
                Cell::from(match model.status.state {
                    Play | Pause => format_progress(&model.status),
                    Stop => String::new(),
                }),
                Cell::from(
                    match &model.currentsong {
                        Some(song) => Line::from(
                            song.title
                                .clone()
                                .unwrap_or("<TITLE NOT FOUND>".into()),
                        ),
                        None => Line::from("祈"),
                    }
                    .centered()
                    .set_style(theme.status_title),
                ),
                Cell::from(Line::from("⎡r z s c⎤").right_aligned()),
            ]),
            Row::new(vec![
                Cell::from(match model.status.state {
                    Play => Text::from("[playing]").style(theme.status_playing),
                    Pause => Text::from("[paused]").style(theme.status_paused),
                    Stop => Text::from("[stopped]").style(theme.status_stopped),
                }),
                Cell::from(
                    match &model.currentsong {
                        Some(song) => Line::from(vec![
                            Span::from(
                                song.artist
                                    .clone()
                                    .unwrap_or("<ARTIST NOT FOUND>".into()),
                            )
                            .style(theme.status_artist),
                            Span::from(format!(
                                " ({})",
                                song_album(song)
                                    .cloned()
                                    .unwrap_or("<ALBUM NOT FOUND>".into())
                            ))
                            .style(theme.status_album),
                        ]),
                        None => "いのり".into(),
                    }
                    .centered(),
                ),
                Cell::from(
                    Line::from(format!(
                        "⎣{} {} {} {}⎦",
                        format_status(
                            model.status.repeat,
                            &model.config.status_indicator_enabled,
                            &model.config.status_indicator_disabled
                        ),
                        format_status(
                            model.status.random,
                            &model.config.status_indicator_enabled,
                            &model.config.status_indicator_disabled
                        ),
                        format_status(
                            model.status.single,
                            &model.config.status_indicator_enabled,
                            &model.config.status_indicator_disabled
                        ),
                        format_status(
                            model.status.consume,
                            &model.config.status_indicator_enabled,
                            &model.config.status_indicator_disabled
                        )
                    ))
                    .right_aligned(),
                ),
            ]),
        ],
        vec![Max(20), Min(10), Max(20)],
    )
    .block(Block::bordered().border_type(BorderType::Rounded));
    frame.render_widget(w, area);
}
