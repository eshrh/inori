use super::*;
use crate::model::*;
use event::KeyModifiers;
use nucleo_matcher::Matcher;
use proto::*;

pub mod library_handler;
pub mod queue_handler;

pub fn handle_vertical(msg: Vertical, selector: &mut impl Selector) {
    match selector.selected() {
        None => {
            if selector.len() != 0 {
                selector.set_selected(Some(0));
            }
        }
        Some(sel) => selector.set_selected(match msg {
            Vertical::Up => Some(safe_decrement(sel, selector.len())),
            Vertical::Down => Some(safe_increment(sel, selector.len())),
        }),
    }
}

// TODO: Figure out a way to eliminate code duplication here
pub fn handle_search_k_tracksel(
    artist: &mut ArtistData,
    k: KeyEvent,
    matcher: &mut Matcher,
) -> Option<Message> {
    if k.modifiers.contains(KeyModifiers::CONTROL) {
        match k.code {
            // TODO: keep track of cursor and implement AEFB
            KeyCode::Char('u') => artist.search.query.clear(),
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
        match k.code {
            KeyCode::Char(c) => artist.search.query.push(c),
            KeyCode::Backspace => {
                let _ = artist.search.query.pop();
            }
            KeyCode::Esc => {
                return Some(Message::LocalSearch(SearchMsg::End));
            }
            KeyCode::Enter => return Some(Message::Enter),
            _ => {}
        }
    }
    artist.update_search(matcher);
    None
}

pub fn handle_search_k<T>(
    s: &mut impl Searchable<T>,
    k: KeyEvent,
    matcher: &mut Matcher,
) -> Option<Message> {
    if k.modifiers.contains(KeyModifiers::CONTROL) {
        match k.code {
            // TODO: keep track of cursor and implement AEFB
            KeyCode::Char('u') => s.filter_mut().query.clear(),
            KeyCode::Char('n') => handle_vertical(Vertical::Down, s),
            KeyCode::Char('p') => handle_vertical(Vertical::Up, s),
            _ => {}
        }
    } else {
        match k.code {
            KeyCode::Char(c) => {
                s.filter_mut().query.push(c);
            }
            KeyCode::Backspace => {
                let _ = s.filter_mut().query.pop();
            }
            KeyCode::Esc => {
                return Some(Message::LocalSearch(SearchMsg::End));
            }
            KeyCode::Enter => return Some(Message::Enter),
            _ => {}
        }
    }
    s.update_filter_cache(matcher);
    s.watch_oob();
    None
}
