use super::*;
use crate::event_handler::Result;
use crate::model::*;
use ratatui::widgets::{ListState, StatefulWidget, TableState};
use selector_state::Selector;

pub fn handle_vertical<T>(msg: Vertical, selector: &mut impl Selector<T>) {
    match selector.selected() {
        None => {}
        Some(sel) => selector.set_selected(match msg {
            Vertical::Up => Some(safe_decrement(sel, selector.len())),
            Vertical::Down => Some(safe_increment(sel, selector.len())),
        }),
    }
}

pub fn handle_library(model: &mut Model, msg: Message) -> Result<()> {
    match msg {
        Message::Direction(Dirs::Vert(d)) => {
            handle_vertical(d, &mut model.library)
        }
        _ => {}
    }
    Ok(())
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
            let sel = model.queue.selected();
            let len = model.queue.len();
            if sel.is_some() {
                model.queue.set_selected(match d {
                    Vertical::Up => Some(safe_decrement(sel.unwrap(), len)),
                    Vertical::Down => Some(safe_increment(sel.unwrap(), len)),
                })
            }
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
