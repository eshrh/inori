use super::*;
use crate::event_handler::Result;
use crate::model::*;
use ratatui::widgets::{ListState, StatefulWidget, TableState};
use selector_state::*;

pub mod library_handler;

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

pub fn handle_queue(model: &mut Model, msg: Message) -> Result<()> {
    if !model.queue.contents.is_empty() && model.queue.selected().is_none() {
        model.queue.set_selected(Some(0));
    }
    if model.queue.contents.is_empty() {
        model.queue.set_selected(None);
    }
    match msg {
        Message::Direction(Dirs::Vert(d)) => {
            handle_vertical(d, &mut model.queue)
        }
        Message::Enter => match model.queue.selected_item() {
            Some(s) => {
                model
                    .conn
                    .switch(s.place.expect("Selected song has no place").pos)?;
            }
            None => (),
        },
        Message::Direction(Dirs::Horiz(d)) => {
            if model.queue.len() >= 2 && model.queue.selected().is_some() {
                let sel = model.queue.selected().unwrap();
                let to = match d {
                    Horizontal::Left => safe_increment(sel, model.queue.len()),
                    Horizontal::Right => safe_decrement(sel, model.queue.len()),
                };
                model.conn.swap(sel as u32, to as u32)?;
                model.queue.set_selected(Some(to));
            }
        }
        Message::Delete => match model.queue.selected() {
            Some(p) => model.conn.delete(p as u32)?,
            _ => (),
        },
        _ => (),
    }
    Ok(())
}

pub fn handle_playlist(model: &mut Model, msg: Message) -> Result<()> {
    Ok(())
}
