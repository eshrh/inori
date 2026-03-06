use super::artist_select_renderer::render_artist_list;
use super::layout;
use super::layout::InoriLayout;
use super::search_renderable::render_searchable_line;
use super::search_renderer::make_search_box;
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
    frame.render_widget(
        make_search_box(
            &model.library.global_search.entries.filter.query,
            true,
            theme,
        ),
        layout[0],
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

    if let Some(a) = layout.track_search {
        let track_query = model
            .library
            .selected_item()
            .map(|artist| artist.search.query.clone())
            .unwrap_or_default();
        frame.render_widget(
            make_search_box(
                &track_query,
                matches!(model.state, State::Searching),
                theme,
            ),
            a,
        );
    }

    render_artist_list(model, frame, layout.artist_select, theme);
    if let Some(a) = layout.artist_search {
        frame.render_widget(
            make_search_box(
                &model.library.artists.filter.query,
                matches!(model.state, State::Searching),
                theme,
            ),
            a,
        );
    }
    if let Some(a) = layout.center_popup {
        render_global_search(model, frame, a, theme);
    }
}
