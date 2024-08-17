use super::status_renderer::render_status;
use super::Theme;
use crate::model::selector_state::*;
use crate::model::LibActiveSelector::*;
use crate::model::*;
use crate::util::{format_progress, format_time, song_album};
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::Duration;

pub fn get_artist_list<'a>(model: &Model) -> List<'a> {
    let artists: Vec<String> = model
        .library
        .contents()
        .map(|artist| artist.name.clone())
        .collect();
    List::new(artists)
}

pub fn get_track_data<'a>(
    artist: Option<&ArtistData>,
    theme: &Theme,
    width: u16
) -> Table<'a> {
    if let Some(artist) = artist {
        let items = artist
            .contents()
            .iter()
            .map(|i| match i {
                TrackSelItem::Album(a) => Row::new(vec![
                    Text::from(format!("{} {}", a.name.clone(), &str::repeat("─", width.into()))),
                    Text::from(format_time(a.total_time())).right_aligned(),
                ])
                .style(theme.album),
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
        Table::new::<Vec<Row>, Vec<Constraint>>(items, vec![Min(10), Max(9)])
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

pub fn render_filter(
    model: &mut Model,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let t = Paragraph::new(vec![Line::from(vec![
        Span::from("Search: "),
        Span::from(&model.library.search.query)
            .style(Style::new().bg(Color::DarkGray).fg(Color::Black)),
    ])])
    .block(
        Block::new()
            .borders(Borders::all().difference(Borders::BOTTOM))
            .border_type(BorderType::Rounded)
            .padding(Padding::vertical(0)),
    );
    frame.render_widget(Clear, area);
    frame.render_widget(t, area);
}

pub fn render(model: &mut Model, frame: &mut Frame, theme: &Theme) {
    let layout = Layout::vertical(vec![Max(4), Min(1)]).split(frame.size());
    let menu_layout =
        Layout::horizontal(vec![Ratio(1, 3), Ratio(2, 3)]).split(layout[1]);
    let header_layout = Layout::horizontal(vec![Ratio(1, 1)]).split(layout[0]);
    render_artist_list(model, frame, menu_layout[0], theme);
    render_track_list(model, frame, menu_layout[1], theme);
    render_status(model, frame, header_layout[0], theme);

    if model.library.search.active {
        let area = Layout::vertical(vec![Min(1), Max(4)]).split(frame.size());
        let bottom = Layout::horizontal(vec![
            Percentage(20),
            Percentage(60),
            Percentage(20),
        ])
        .split(area[1]);
        frame.render_widget(Clear, bottom[1]);
        render_filter(model, frame, bottom[1], theme);
    }
}
