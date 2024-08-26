use super::artist_select_renderer::render_str_with_idxs;
use super::Theme;
use crate::model::proto::*;
use crate::model::LibActiveSelector::*;
use crate::model::*;
use crate::util::format_time;
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::Duration;

fn itemref_to_row<'a>(
    artist: &ArtistData,
    item: &TrackSelItem,
    width: u16,
    theme: &Theme,
) -> Row<'a> {
    let idxs = item.rank.and_then(|r| artist.search.cache.indices.get(r));
    let row = match item.item {
        ItemRef::Album(a) => {
            let mut album_line = vec![Span::from(" ")];
            if let Some(idxs) = idxs {
                album_line.extend(render_str_with_idxs(
                    a.name.clone(),
                    idxs,
                    a.name.chars().count(),
                    theme,
                ))
            } else {
                album_line.push(Span::from(a.name.clone()))
            }
            album_line.push(Span::from(str::repeat("─", width.into())));
            Row::new(vec![
                Line::from(album_line),
                Line::from(format_time(a.total_time())).right_aligned(),
            ])
            .style(theme.album)
        }
        ItemRef::Song(s) => {
            let mut track_line = vec![Span::from(str::repeat(" ", 3))];
            if let Some(title) = s.title.clone() {
                if let Some(idxs) = idxs {
                    track_line.extend(render_str_with_idxs(
                        title.clone(),
                        idxs,
                        title.chars().count(),
                        theme,
                    ))
                } else {
                    track_line.push(Span::from(title))
                }
            } else {
                track_line.push(Span::from("Unknown Song"))
            }

            Row::new(vec![
                Line::from(track_line),
                Line::from(vec![Span::from(format_time(
                    s.duration.unwrap_or(Duration::from_secs(0)),
                ))])
                .right_aligned(),
            ])
        }
    };
    if idxs.is_some() {
        row.style(Style::new().bg(Color::DarkGray))
    } else {
        row
    }
}

fn get_track_data<'a>(
    artist: Option<&ArtistData>,
    theme: &Theme,
    width: u16,
) -> Table<'a> {
    if let Some(artist) = artist {
        let items = artist
            .contents()
            .iter()
            .map(|i| itemref_to_row(artist, i, width, theme))
            .collect::<Vec<Row>>();
        Table::new::<Vec<Row>, Vec<Constraint>>(items, vec![Min(10), Max(9)])
    } else {
        return Table::new::<Vec<Row>, Vec<u16>>(vec![], vec![]);
    }
}

pub fn render_track_list(
    model: &mut Model,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let list = get_track_data(model.library.selected_item(), theme, area.width)
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
