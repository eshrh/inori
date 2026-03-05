use crate::model::proto::Searchable;
use crate::model::*;
use crate::view::layout::InoriLayout;
use ratatui::prelude::Constraint::*;
use ratatui::prelude::*;

#[derive(Default)]
pub struct LibraryLayout {
    pub artist_select: Rect,
    pub artist_search: Option<Rect>,
    pub track_select: Rect,
    pub track_search: Option<Rect>,
    pub header: Rect,
    pub center_popup: Option<Rect>,
}

impl InoriLayout for LibraryLayout {
    fn new(frame_rect: Rect, model: &Model) -> Self {
        let mut new = LibraryLayout::default();
        let layout = Layout::vertical(vec![Max(4), Min(1)]).split(frame_rect);
        let menu_layout =
            Layout::horizontal(vec![Ratio(1, 3), Ratio(2, 3)]).split(layout[1]);

        new.header = Layout::horizontal(vec![Ratio(1, 1)]).split(layout[0])[0];

        let left_panel =
            Layout::vertical(vec![Max(3), Min(1)]).split(menu_layout[0]);

        let right_panel =
            Layout::vertical(vec![Max(3), Min(1)]).split(menu_layout[1]);

        if model.library.artists.filter.active {
            new.artist_select = left_panel[1];
            new.artist_search = Some(left_panel[0]);
        } else {
            new.artist_select = menu_layout[0];
        }

        if model
            .library
            .selected_item()
            .is_some_and(|a| a.search.active)
        {
            new.track_select = right_panel[1];
            new.track_search = Some(right_panel[0]);
        } else {
            new.track_select = menu_layout[1];
        }

        let center_popup_h = Layout::horizontal(vec![
            Percentage(20),
            Percentage(60),
            Percentage(20),
        ])
        .split(frame_rect);

        let center_popup_v = Layout::vertical(vec![
            Percentage(20),
            Percentage(60),
            Percentage(20),
        ])
        .split(center_popup_h[1]);

        if model.library.global_search.entries.filter.active {
            new.center_popup = Some(center_popup_v[1]);
        }

        new
    }
}
