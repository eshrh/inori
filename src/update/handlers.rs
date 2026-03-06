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

enum SearchEditOutcome {
    Message(Message),
    Continue,
}

fn apply_search_edit(
    query: &mut String,
    k: KeyEvent,
    allow_tab_toggle: bool,
) -> SearchEditOutcome {
    match k.code {
        KeyCode::Char(c) => {
            query.push(c);
            SearchEditOutcome::Continue
        }
        KeyCode::Backspace => {
            let _ = query.pop();
            SearchEditOutcome::Continue
        }
        KeyCode::Tab if allow_tab_toggle => {
            SearchEditOutcome::Message(Message::ToggleScreen)
        }
        KeyCode::Esc => {
            SearchEditOutcome::Message(Message::LocalSearch(SearchMsg::End))
        }
        KeyCode::Enter => SearchEditOutcome::Message(Message::Select),
        _ => SearchEditOutcome::Continue,
    }
}

fn delete_previous_word(query: &mut String) {
    let trimmed_len = query.trim_end().len();
    query.truncate(trimmed_len);
    if query.is_empty() {
        return;
    }

    let mut split_at = 0usize;
    for (i, c) in query.char_indices() {
        if c.is_whitespace() {
            split_at = i;
        }
    }
    query.truncate(split_at);
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
    if k.modifiers.contains(KeyModifiers::CONTROL) {
        match k.code {
            // TODO: keep track of cursor and implement AEFB
            KeyCode::Char('u') => artist.search.query.clear(),
            KeyCode::Char('w') => {
                delete_previous_word(&mut artist.search.query)
            }
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
            }
            _ => {}
        }
    } else {
        match apply_search_edit(&mut artist.search.query, k, false) {
            SearchEditOutcome::Message(m) => return Some(m),
            SearchEditOutcome::Continue => {}
        }
    }
    artist.update_search(matcher, aliases);
    None
}

pub fn handle_search_k<T>(
    s: &mut impl Searchable<T>,
    k: KeyEvent,
    matcher: &mut Matcher,
    top_k: usize,
) -> Option<Message> {
    if k.modifiers.contains(KeyModifiers::CONTROL) {
        match k.code {
            // TODO: keep track of cursor and implement AEFB
            KeyCode::Char('u') => s.filter_mut().query.clear(),
            KeyCode::Char('w') => {
                delete_previous_word(&mut s.filter_mut().query)
            }
            KeyCode::Char('n') => handle_vertical(Vertical::Down, s),
            KeyCode::Char('p') => handle_vertical(Vertical::Up, s),
            _ => {}
        }
    } else {
        match apply_search_edit(&mut s.filter_mut().query, k, true) {
            SearchEditOutcome::Message(m) => return Some(m),
            SearchEditOutcome::Continue => {}
        }
    }
    s.update_filter_cache(matcher, Some(top_k));
    s.watch_oob();
    None
}
