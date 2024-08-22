use super::*;
use crate::model::*;
use event::KeyModifiers;
use selector_state::*;

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

pub fn handle_search_k<T>(
    s: &mut impl Searchable<T>,
    k: KeyEvent,
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
            _ => {}
        }
    }
    s.update_filter_cache();
    s.watch_oob();
    None
}
