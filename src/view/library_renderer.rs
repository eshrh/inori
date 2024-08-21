use super::status_renderer::render_status;
use super::Theme;
use crate::model::selector_state::*;
use crate::model::LibActiveSelector::*;
use crate::model::*;
use crate::util::{format_progress, format_time, song_album};
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::convert::TryFrom;
use std::time::Duration;
use style::Styled;
use itertools::intersperse;

pub fn get_track_data<'a>(
    artist: Option<&ArtistData>,
    theme: &Theme,
    width: u16,
) -> Table<'a> {
    if let Some(artist) = artist {
        let items = artist
            .contents()
            .iter()
            .map(|i| match i {
                TrackSelItem::Album(a) => Row::new(vec![
                    Text::from(format!(
                        " {} {}",
                        a.name.clone(),
                        &str::repeat("─", width.into())
                    )),
                    Text::from(format_time(a.total_time())).right_aligned(),
                ])
                .style(theme.album),
                TrackSelItem::Song(s) => Row::new(vec![
                    Text::from(
                        str::repeat(" ", 3)
                            + &s.title.clone().unwrap_or("Unknown Song".into()),
                    ),
                    Text::from(format_time(
                        s.duration.unwrap_or(Duration::from_secs(0)),
                    ))
                    .right_aligned(),
                ]),
            })
            .collect::<Vec<Row>>();
        Table::new::<Vec<Row>, Vec<Constraint>>(items, vec![Min(10), Max(9)])
    } else {
        return Table::new::<Vec<Row>, Vec<u16>>(vec![], vec![]);
    }
}

pub fn render_str_with_idxs<'a>(
    str: String,
    idxs: &Vec<u32>,
    len: usize,
    theme: &Theme
) -> Line<'a> {
    let spans: Vec<Span> = str
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if idxs.contains(&u32::try_from(i).unwrap()) {
                Span::from(c.to_string())
                    .style(Style::default().add_modifier(Modifier::UNDERLINED))
            } else {
                Span::from(c.to_string())
            }
            .style(if i >= len { theme.artist_sort
            } else {
                Style::default()
            })
        })
        .collect();
    Line::from(spans)
}

pub fn get_artist_list<'a>(model: &Model, theme: &Theme) -> List<'a> {
    if model.library.artist_search.active {
        let indices = &model.library.artist_search.cache.indices;
        List::new(model.library.contents().zip(indices).map(
            |(artist, idxs_o)| {
                let len = artist.name.chars().count();
                if let Some(idxs) = idxs_o {
                    render_str_with_idxs(artist.to_fuzzy_find_str(), idxs, len, theme)
                } else {
                    Line::from(vec![
                        Span::from(artist.name[0..len].to_string()),
                        Span::from(artist.sort_names.join(", ") + "]"),
                    ])
                }
            },
        ))
    } else {
        List::new(
            model
                .library
                .contents()
                .map(|artist| artist.name.clone())
                .collect::<Vec<String>>(),
        )
    }
}

pub fn render_artist_list(
    model: &mut Model,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let artist_list = get_artist_list(model, theme)
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

pub fn make_search_box<'a>(query: &'a String, active: bool) -> Paragraph<'a> {
    Paragraph::new(vec![Line::from(vec![
        Span::from("> "),
        Span::from(query).style(if active {
            Style::new().bg(Color::White).fg(Color::Black)
        } else {
            Style::new().bg(Color::DarkGray).fg(Color::Black)
        }),
    ])])
    .block(Block::bordered().border_type(BorderType::Thick))
}

pub fn render_filter(
    model: &mut Model,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    frame.render_widget(
        make_search_box(
            &model.library.artist_search.query,
            matches!(model.state, State::Searching),
        ),
        area,
    );
}

pub fn render_global_search(
    model: &mut Model,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let layout = Layout::vertical(vec![Max(3), Min(1)]).margin(1).split(area);

    frame.render_widget(Clear, area);
    frame.render_widget(Block::bordered(), area);
    frame.render_widget(
        make_search_box(&model.library.global_search.search.query, true),
        layout[0],
    );
    let list = List::new(model.library.global_search.contents().map(
        |ie| {
            let mut out = vec![Span::from(ie.artist.clone())];
            if let Some(artist_sort) = ie.artist_sort.clone() {
                out.push(Span::from("/").style(theme.slash_span));
                out.push(Span::from(artist_sort).style(theme.artist_sort));
            }
            if let Some(album) = ie.album.clone() {
                out.push(Span::from("/").style(theme.slash_span));
                out.push(Span::from(album));
            }
            if let Some(title) = ie.title.clone() {
                out.push(Span::from("/").style(theme.slash_span));
                out.push(Span::from(title));
            }
            Line::from(out)
        }
    ));
    frame.render_stateful_widget(
        list
            .block(Block::bordered())
            .highlight_style(theme.item_highlight_active),
        layout[1],
        &mut model.library.global_search.results_state,
    );
}

pub fn render(model: &mut Model, frame: &mut Frame, theme: &Theme) {
    let layout = Layout::vertical(vec![Max(4), Min(1)]).split(frame.size());
    let menu_layout =
        Layout::horizontal(vec![Ratio(1, 3), Ratio(2, 3)]).split(layout[1]);
    let header_layout = Layout::horizontal(vec![Ratio(1, 1)]).split(layout[0]);
    let left_panel =
        Layout::vertical(vec![Max(3), Min(1)]).split(menu_layout[0]);

    let center_popup_h = Layout::horizontal(vec![
        Percentage(20),
        Percentage(60),
        Percentage(20),
    ])
    .split(frame.size());
    let center_popup_v =
        Layout::vertical(vec![Percentage(20), Percentage(60), Percentage(20)])
            .split(center_popup_h[1]);
    let center_popup = center_popup_v[1];

    render_track_list(model, frame, menu_layout[1], theme);
    render_status(model, frame, header_layout[0], theme);

    if model.library.artist_search.active {
        render_filter(model, frame, left_panel[0], theme);
        render_artist_list(model, frame, left_panel[1], theme);
    } else {
        render_artist_list(model, frame, menu_layout[0], theme);
    }

    if model.library.global_search.search.active {
        render_global_search(model, frame, center_popup, theme)
    }
}
