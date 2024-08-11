use super::*;
use crate::event_handler::Result;
use crate::model::*;

pub fn handle_library(model: &mut Model, msg: Message) -> Result<()> {
    Ok(())
}

pub fn handle_queue(model: &mut Model, msg: Message) -> Result<()> {
    if !model.queue.contents.is_empty() && model.queue.selection.is_none() {
        model.queue.selection = Some(0);
    }
    if model.queue.contents.is_empty() {
        model.queue.selection = None;
    }
    match msg {
        Message::Direction(Dirs::Vert(d)) => {
            let sel = model.queue.selection;
            let len = model.queue.len();
            if sel.is_some() {
                model.queue.selection = match d {
                    Vertical::Up => Some(safe_decrement(sel.unwrap(), len)),
                    Vertical::Down => Some(safe_increment(sel.unwrap(), len)),
                }
            }
        }
        Message::Enter => match model.queue.get_sel() {
            Some(s) => {
                model
                    .conn
                    .switch(s.place.expect("Selected song has no place").pos)?;
            }
            None => (),
        },
        Message::Direction(Dirs::Horiz(d)) => {
            if model.queue.len() >= 2 && model.queue.selection.is_some() {
                let sel = model.queue.selection.unwrap();
                let to = match d {
                    Horizontal::Left => safe_increment(sel, model.queue.len()),
                    Horizontal::Right => safe_decrement(sel, model.queue.len()),
                };
                model.conn.swap(sel as u32, to as u32)?;
                model.queue.selection = Some(to);
            }
        },
        Message::Delete => {
            match model.queue.selection {
                Some(p) => model.conn.delete(p as u32)?,
                _ => ()
            }
        }
        _ => (),
    }
    Ok(())
}

pub fn handle_playlist(model: &mut Model, msg: Message) -> Result<()> {
    Ok(())
}
