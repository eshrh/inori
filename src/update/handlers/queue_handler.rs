use super::*;
use crate::event_handler::Result;

pub fn handle_queue(model: &mut Model, msg: Message) -> Result<Update> {
    match msg {
        Message::Tab => {
            model.screen = Screen::Library;
            Ok(Update::empty())
        }
        Message::Direction(Dirs::Vert(d)) => {
            handle_vertical(d, &mut model.queue);
            Ok(Update::empty())
        }
        Message::Enter => {
            if let Some(s) = model.queue.selected_item() {
                model
                    .conn
                    .switch(s.place.expect("Selected song has no place").pos)?;
            }
            Ok(Update::STATUS | Update::CURRENT_SONG)
        }
        Message::Direction(Dirs::Horiz(d)) => {
            if model.queue.len() >= 2 {
                if let Some(p) = model.queue.selected() {
                    let to = match d {
                        Horizontal::Left => {
                            safe_increment(p, model.queue.len())
                        }
                        Horizontal::Right => {
                            safe_decrement(p, model.queue.len())
                        }
                    };
                    model.conn.swap(p as u32, to as u32)?;
                    model.queue.set_selected(Some(to));
                    model.queue.watch_oob();
                }
            }
            Ok(Update::STATUS | Update::QUEUE)
        }
        Message::Delete => {
            if let Some(p) = model.queue.selected() {
                model.conn.delete(p as u32)?;
                model.queue.set_selected(Some(safe_decrement(
                    p,
                    model.queue.len() - 1,
                )));
                model.queue.watch_oob();
            }
            Ok(Update::STATUS | Update::QUEUE)
        }
        _ => Ok(Update::empty()),
    }
}
