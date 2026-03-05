use super::*;
use crate::view::layout::InoriLayout;
use crate::{event_handler::Result, view::layout::queue_layout::QueueLayout};

pub fn handle_queue(model: &mut Model, msg: Message) -> Result<Update> {
    match msg {
        Message::ToggleScreen => {
            model.screen = Screen::Library;
            Ok(Update::empty())
        }
        Message::Direction(Dirs::Vert(d)) => {
            handle_vertical(d, &mut model.queue);
            Ok(Update::empty())
        }
        Message::ScrollScreenful(v) => {
            let k = QueueLayout::new(model.frame_size, model).queue.height;
            scroll_screenful(v, k.into(), &mut model.queue);
            Ok(Update::empty())
        }
        Message::Select => {
            if let Some(s) = model.queue.selected_item() {
                if let Some(place) = s.place {
                    model.conn.switch(place.pos)?;
                }
            }
            Ok(Update::STATUS | Update::CURRENT_SONG)
        }
        Message::Direction(Dirs::Horiz(d)) => {
            if model.queue.len() >= 2 {
                if let Some(p) = model.queue.selected() {
                    let to = match d {
                        Horizontal::Left => {
                            safe_subtract(p, 1, model.queue.len())
                        }
                        Horizontal::Right => safe_add(p, 1, model.queue.len()),
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
                let len = model.queue.len();
                let next = if len == 0 {
                    None
                } else {
                    Some(safe_subtract(p, 1, len))
                };
                model.queue.set_selected(next);
                model.queue.watch_oob();
            }
            Ok(Update::STATUS | Update::QUEUE)
        }
        Message::LocalSearch(SearchMsg::Start) => {
            model.queue.songs.filter.active = true;
            model.state = State::Searching;
            if model.queue.len() != 0 {
                model.queue.set_selected(Some(0));
            }
            Ok(Update::empty())
        }
        Message::LocalSearch(SearchMsg::End) => {
            model.state = State::Running;
            Ok(Update::empty())
        }
        Message::Escape => {
            model.queue.songs.filter.active = false;
            model.queue.songs.filter.query = String::new();
            Ok(Update::empty())
        }
        _ => Ok(Update::empty()),
    }
}

pub fn handle_search(model: &mut Model, k: KeyEvent) -> Result<Update> {
    if let Some(m) = handle_search_k(
        &mut model.queue,
        k,
        &mut model.matcher,
        model.frame_size.height.into(),
    ) {
        handle_msg(model, m)
    } else {
        Ok(Update::empty())
    }
}
