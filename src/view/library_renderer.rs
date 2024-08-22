use super::artist_select_renderer::render_artist_list;
use super::search_renderer::make_search_box;
use super::status_renderer::render_status;
use super::track_select_renderer::render_track_list;
use super::Theme;
use crate::model::proto::*;
use crate::model::*;
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

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
            theme,
        ),
        area,
    );
}

pub fn render_artist_sort<'a>(text: String, style: Style) -> Span<'a> {
    Span::from(format!(" {}{}{}", "[", text, "]")).style(style)
}

pub fn render_global_search(
    model: &mut Model,
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
) {
    let layout = Layout::vertical(vec![Max(3), Min(1)])
        .horizontal_margin(2)
        .vertical_margin(1)
        .split(area);

    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::bordered().border_type(BorderType::Rounded),
        area,
    );
    frame.render_widget(
        make_search_box(&model.library.global_search.search.query, true, theme),
        layout[0],
    );
    let list = List::new(model.library.global_search.contents().map(|ie| {
        let mut out = vec![Span::from(ie.artist.clone())];
        if let Some(artist_sort) = ie.artist_sort.clone() {
            if artist_sort != ie.artist {
                out.push(render_artist_sort(artist_sort, theme.artist_sort));
            }
        }
        if let Some(album) = ie.album.clone() {
            out.push(Span::from("/").style(theme.slash_span));
            out.push(Span::from(album).style(theme.album));
        }
        if let Some(title) = ie.title.clone() {
            out.push(Span::from("/").style(theme.slash_span));
            out.push(Span::from(title));
        }
        Line::from(out)
    }));
    frame.render_stateful_widget(
        list.block(Block::bordered())
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
