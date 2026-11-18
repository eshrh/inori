use super::artist_select_renderer::render_artist_list;
use super::layout;
use super::layout::InoriLayout;
use super::search_renderable::render_searchable_line;
use super::search_renderer::render_search_box;
use super::status_renderer::render_status;
use super::track_select_renderer::render_track_list;
use super::Theme;
use crate::model::proto::*;
use crate::model::*;
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

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
    render_search_box(
        &mut model.library.global_search.entries.filter,
        matches!(model.state, State::Searching),
        frame,
        layout[0],
        theme,
    );
    let list = List::new(
        (0..model.library.global_search.display_len()).filter_map(|i| {
            let ie = model.library.global_search.display_get(i)?;
            let idxs = model
                .library
                .global_search
                .entries
                .filter
                .cache
                .indices
                .get(i)?;
            Some(render_searchable_line(ie, Some(idxs), theme))
        }),
    );
    frame.render_stateful_widget(
        list.block(Block::bordered())
            .highlight_style(theme.item_highlight_active),
        layout[1],
        &mut model.library.global_search.results_state,
    );
}

pub fn render(model: &mut Model, frame: &mut Frame, theme: &Theme) {
    let layout =
        layout::library_layout::LibraryLayout::new(frame.area(), model);

    render_status(model, frame, layout.header, theme);
    render_track_list(model, frame, layout.track_select, theme);

    let searching = matches!(model.state, State::Searching)
        && !model.library.global_search.entries.filter.active;
    if let Some(a) = layout.track_search {
        let active = searching
            && matches!(model.library.active, LibActiveSelector::TrackSelector);
        if let Some(artist) = model.library.selected_item_mut() {
            render_search_box(&mut artist.search, active, frame, a, theme);
        }
    }

    render_artist_list(model, frame, layout.artist_select, theme);
    if let Some(a) = layout.artist_search {
        let active = searching
            && matches!(
                model.library.active,
                LibActiveSelector::ArtistSelector
            );
        render_search_box(
            &mut model.library.artists.filter,
            active,
            frame,
            a,
            theme,
        );
    }
    if let Some(a) = layout.center_popup {
        render_global_search(model, frame, a, theme);
    }
}
