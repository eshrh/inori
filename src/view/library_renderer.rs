use super::artist_select_renderer::render_artist_list;
use super::layout;
use super::layout::InoriLayout;
use super::search_renderer::make_search_box;
use super::status_renderer::render_status;
use super::track_select_renderer::render_track_list;
use super::Theme;
use crate::model::proto::*;
use crate::model::*;
use crate::view::search_doc::{
    has_match_in_span, has_match_in_spans, AliasSpan,
};
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;
use ratatui::widgets::*;

fn alias_spans(
    alias: &str,
    style: Style,
    search_span: AliasSpan,
    idx: &[u32],
) -> Vec<Span<'static>> {
    let mut out = Vec::with_capacity(alias.chars().count() + 3);
    out.push(Span::from(" "));
    out.push(Span::from("[").style(style));
    for (k, c) in alias.chars().enumerate() {
        let mut span = Span::from(c.to_string()).style(style);
        if u32::try_from(search_span.start + k)
            .ok()
            .is_some_and(|j| idx.contains(&j))
        {
            span.style = span.style.add_modifier(Modifier::UNDERLINED);
        }
        out.push(span);
    }
    out.push(Span::from("]").style(style));
    out
}

pub fn render_search_item<'a>(
    ie: &InfoEntry,
    idx: &[u32],
    theme: &Theme,
) -> Line<'a> {
    let base = ie.to_display_string();
    let search_doc = ie.search_doc();
    let mut out: Vec<Span> =
        base.chars().map(|c| Span::from(c.to_string())).collect();
    let mut album_end_idx: Option<usize> = None;

    let mut cur = ie.artist.chars().count();
    if let Some(artist_sort) = &ie.artist_sort {
        if *artist_sort != ie.artist {
            let len = artist_sort.chars().count();
            cur += 1; // for spc
            for item in out.iter_mut().take(cur + len + 2).skip(cur) {
                item.style = theme.field_artistsort;
            }
            cur += len + 2;
        }
    }
    if let Some(album) = &ie.album {
        let len = album.chars().count();
        out[cur].style = theme.slash_span;
        cur += 1;
        for item in out.iter_mut().skip(cur).take(len) {
            item.style = theme.field_album;
        }
        cur += len;
        album_end_idx = Some(cur);
    }
    if let Some(_title) = &ie.title {
        out[cur].style = theme.slash_span;
    }

    let show_album_alias = search_doc
        .album_alias
        .is_some_and(|segment| has_match_in_span(idx, segment));
    let show_title_alias = search_doc
        .title_alias
        .is_some_and(|segment| has_match_in_span(idx, segment));
    let show_variant_marker =
        has_match_in_spans(idx, &search_doc.variant_spans);

    for (i, item) in out.iter_mut().take(search_doc.base_len).enumerate() {
        let matches_base =
            u32::try_from(i).ok().is_some_and(|j| idx.contains(&j));
        if matches_base {
            item.style = item.style.add_modifier(Modifier::UNDERLINED);
        }
    }

    if show_album_alias {
        if let (Some(alias), Some(search_span)) =
            (ie.album_alias.as_ref(), search_doc.album_alias)
        {
            let spans = alias_spans(alias, theme.field_album, search_span, idx);
            if let Some(insert_at) = album_end_idx {
                out.splice(insert_at..insert_at, spans);
            } else {
                out.extend(spans);
            }
        }
    }
    if show_title_alias {
        if let (Some(alias), Some(search_span)) =
            (ie.title_alias.as_ref(), search_doc.title_alias)
        {
            out.extend(alias_spans(
                alias,
                theme.field_artistsort,
                search_span,
                idx,
            ));
        }
    }

    if show_variant_marker {
        let mut marker =
            vec![Span::from("["), Span::from("VAR"), Span::from("] ")];
        for span in &mut marker {
            span.style = span.style.add_modifier(Modifier::UNDERLINED);
        }
        out.splice(0..0, marker);
    }

    Line::from(out)
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
            Some(render_search_item(ie, idxs, theme))
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
