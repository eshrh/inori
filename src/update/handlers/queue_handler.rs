use super::*;
use crate::event_handler::Result;

pub fn handle_queue(model: &mut Model, msg: Message) -> Result<()> {
    if !model.queue.contents.is_empty() && model.queue.selected().is_none() {
        model.queue.set_selected(Some(0));
    }
    if model.queue.contents.is_empty() {
        model.queue.set_selected(None);
    }
    match msg {
        Message::Tab => model.screen = Screen::Library,
        Message::Direction(Dirs::Vert(d)) => {
            handle_vertical(d, &mut model.queue)
        }
        Message::Enter => {
            if let Some(s) = model.queue.selected_item() {
                model
                    .conn
                    .switch(s.place.expect("Selected song has no place").pos)?;
            }
        }
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
        Message::Delete => {
            if let Some(p) = model.queue.selected() {
                model.conn.delete(p as u32)?;
                model.queue.set_selected(Some(safe_decrement(
                    model.queue.selected().unwrap(),
                    model.queue.len() - 1,
                )));
            }
        }
        _ => (),
    }
    Ok(())
}
