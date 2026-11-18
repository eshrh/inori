use super::*;
use crate::model::title_alias::AliasMaps;
use crate::model::*;
use crate::util::safe_add;
use crate::util::safe_subtract;
use event::KeyModifiers;
use nucleo_matcher::Matcher;
use proto::*;

pub mod library_handler;
pub mod queue_handler;

pub(super) enum SearchEditOutcome {
    Message(Message),
    Changed,
    Navigated,
    Continue,
}

fn apply_search_edit(
    filter: &mut Filter,
    k: KeyEvent,
    allow_tab_toggle: bool,
) -> SearchEditOutcome {
    let message = match k.code {
        KeyCode::Tab if allow_tab_toggle => Some(Message::ToggleScreen),
        KeyCode::Esc => Some(Message::LocalSearch(SearchMsg::End)),
        KeyCode::Enter => Some(Message::Select),
        _ => None,
    };
    if let Some(message) = message {
        SearchEditOutcome::Message(message)
    } else if filter.edit_query(k) {
        SearchEditOutcome::Changed
    } else {
        SearchEditOutcome::Continue
    }
}

pub fn handle_vertical(msg: Vertical, selector: &mut impl Selector) {
    match selector.selected() {
        None => {
            if selector.len() != 0 {
                selector.set_selected(Some(0));
            }
        }
        Some(sel) => selector.set_selected(match msg {
            Vertical::Up => Some(safe_subtract(sel, 1, selector.len())),
            Vertical::Down => Some(safe_add(sel, 1, selector.len())),
            Vertical::Top => Some(0),
            Vertical::Bottom => {
                Some(safe_subtract(selector.len(), 1, selector.len()))
            }
        }),
    }
}

pub fn scroll_screenful(
    dir: Vertical,
    height: usize,
    selector: &mut impl Selector,
) {
    let len = selector.len();
    match dir {
        Vertical::Up => {
            selector.set_selected(Some(selector.offset()));
            selector.set_offset(safe_subtract(selector.offset(), height, len));
        }
        Vertical::Down => {
            let mut next = safe_add(selector.offset(), height, len);
            if next > 0 && next < safe_subtract(len, 1, len) {
                next = safe_subtract(next, 3, len);
                selector.set_offset(next);
            }
            selector.set_selected(Some(next));
        }
        _ => {}
    }
}

pub fn handle_search_k_tracksel(
    artist: &mut ArtistData,
    k: KeyEvent,
    matcher: &mut Matcher,
    aliases: &AliasMaps,
) -> Option<Message> {
    if k.modifiers == KeyModifiers::CONTROL {
        match k.code {
            KeyCode::Char('n') => {
                if let Some(Some(r)) = artist.selected_item().map(|i| i.rank) {
                    let idx = artist
                        .contents()
                        .iter()
                        .position(|i| i.rank == Some(r + 1));
                    if idx.is_some() {
                        artist.set_selected(idx)
                    }
                }
                return None;
            }
            KeyCode::Char('p') => {
                if let Some(Some(r)) = artist.selected_item().map(|i| i.rank) {
                    if r > 0 {
                        artist.set_selected(
                            artist
                                .contents()
                                .iter()
                                .position(|i| i.rank == Some(r - 1)),
                        );
                    }
                }
                return None;
            }
            _ => {}
        }
    }
    match apply_search_edit(&mut artist.search, k, false) {
        SearchEditOutcome::Message(message) => Some(message),
        SearchEditOutcome::Changed => {
            artist.update_search_query(matcher, aliases);
            None
        }
        _ => None,
    }
}

pub fn handle_search_k<T>(
    s: &mut impl Searchable<T>,
    k: KeyEvent,
    matcher: &mut Matcher,
    top_k: usize,
) -> SearchEditOutcome {
    if k.modifiers == KeyModifiers::CONTROL {
        match k.code {
            KeyCode::Char('n') => {
                handle_vertical(Vertical::Down, s);
                return SearchEditOutcome::Navigated;
            }
            KeyCode::Char('p') => {
                handle_vertical(Vertical::Up, s);
                return SearchEditOutcome::Navigated;
            }
            _ => {}
        }
    }
    let outcome = apply_search_edit(s.filter_mut(), k, true);
    if matches!(outcome, SearchEditOutcome::Changed) {
        s.update_filter_query(matcher, Some(top_k));
        s.watch_oob();
    }
    outcome
}
