use super::*;
use crate::event_handler::Result;
use crate::model::*;
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

pub fn handle_playlist(model: &mut Model, msg: Message) -> Result<()> {
    Ok(())
}
