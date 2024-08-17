use super::Theme;
use crate::model::selector_state::*;
use crate::model::LibActiveSelector::*;
use crate::model::*;
use crate::util::{format_progress, format_time, song_album};
use mpd::State::*;
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::Duration;
use style::Styled;

pub fn get_artist_list<'a>(model: &Model) -> List<'a> {
    let artists: Vec<String> = model
        .library
        .contents()
        .map(|artist| artist.name.clone())
        .collect();
    List::new(artists)
}

pub fn get_track_data<'a>(artist: Option<&ArtistData>, theme: &Theme) -> Table<'a> {
    if let Some(artist) = artist {
        let items = artist
            .contents()
            .iter()
            .map(|i| match i {
                TrackSelItem::Album(a) => Row::new(vec![
                    Text::from(a.name.clone()),
                    Text::from(format_time(a.total_time())).right_aligned(),
                ]) .style(theme.album),
                TrackSelItem::Song(s) => Row::new(vec![
                    Text::from(
                        "    ".to_string()
                            + &s.title.clone().unwrap_or("Unknown Song".into()),
                    ),
                    Text::from(format_time(
                        s.duration.unwrap_or(Duration::from_secs(0)),
                    ))
                    .right_aligned(),
                ]),
            })
            .collect::<Vec<Row>>();
        Table::new::<Vec<Row>, Vec<Constraint>>(items, vec![Min(10), Max(10)])
    } else {
        return Table::new::<Vec<Row>, Vec<u16>>(vec![], vec![]);
    }
}

pub fn render_artist_list(
    model: &mut Model,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let artist_list = get_artist_list(model)
        .block(
            match model.library.active {
                ArtistSelector => {
                    Block::bordered().border_style(theme.block_active)
                }
                TrackSelector => Block::bordered(),
            }
            .title("Artists"),
        )
        .highlight_style(match model.library.active {
            ArtistSelector => theme.item_highlight_active,
            TrackSelector => theme.item_highlight_inactive,
        });

    frame.render_stateful_widget(
        artist_list,
        area,
        &mut model.library.artist_state,
    );
}

pub fn render_track_list(
    model: &mut Model,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let list = get_track_data(model.library.selected_item(), theme)
        .block(
            match model.library.active {
                ArtistSelector => Block::bordered(),
                TrackSelector => {
                    Block::bordered().border_style(theme.block_active)
                }
            }
            .title("Tracks"),
        )
        .highlight_style(match model.library.active {
            ArtistSelector => theme.item_highlight_inactive,
            TrackSelector => theme.item_highlight_active,
        })
        .highlight_symbol("->")
        .highlight_spacing(HighlightSpacing::Always);

    match model.library.selected_item_mut() {
        Some(artist) => frame.render_stateful_widget(
            list,
            area,
            &mut artist.track_sel_state,
        ),
        None => frame.render_widget(list, area),
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
                Cell::from("r z s c"),
            ]),
            Row::new(vec![
                Cell::from(match model.status.state {
                    Play => "[playing]",
                    Pause => "[paused]",
                    Stop => "[stopped]",
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
                Cell::from(format!(
                    "{} {} {} {}",
                    format_status(model.status.repeat),
                    format_status(model.status.random),
                    format_status(model.status.single),
                    format_status(model.status.consume)
                )),
            ]),
        ],
        vec![Max(10), Min(10), Max(10)],
    )
    .block(Block::bordered());
    frame.render_widget(w, area);
}

pub fn format_status(state: bool) -> String {
    if state {
        "#".to_string()
    } else {
        "_".to_string()
    }
}

pub fn render(model: &mut Model, frame: &mut Frame, theme: &Theme) {
    let layout = Layout::vertical(vec![Max(4), Min(1)]).split(frame.size());
    let menu_layout =
        Layout::horizontal(vec![Ratio(1, 3), Ratio(2, 3)]).split(layout[1]);
    let header_layout = Layout::horizontal(vec![Ratio(1, 1)]).split(layout[0]);
    render_artist_list(model, frame, menu_layout[0], theme);
    render_track_list(model, frame, menu_layout[1], theme);

    render_status(model, frame, header_layout[0], theme);
}
