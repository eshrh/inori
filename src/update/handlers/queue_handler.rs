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
            let Some(id) = model.queue.selected_id() else {
                return Ok(Update::empty());
            };
            model.conn.switch(id)?;
            Ok(Update::STATUS | Update::CURRENT_SONG)
        }
        Message::Direction(Dirs::Horiz(d)) => {
            let Some(index) = model.queue.selected() else {
                return Ok(Update::empty());
            };
            let to = match d {
                Horizontal::Left => index.checked_sub(1),
                Horizontal::Right => index.checked_add(1),
            };
            let Some((from, to)) = model.queue.id_at(index).zip(
                to.filter(|&i| i < model.queue.display_len())
                    .and_then(|i| model.queue.id_at(i)),
            ) else {
                return Ok(Update::empty());
            };
            model.conn.swap(from, to)?;
            Ok(Update::STATUS | Update::QUEUE)
        }
        Message::Delete => {
            let Some(id) = model.queue.selected_id() else {
                return Ok(Update::empty());
            };
            model.conn.delete(id)?;
            let next = model.queue.selected().and_then(|index| {
                index
                    .checked_sub(1)
                    .or_else(|| (model.queue.display_len() > 1).then_some(1))
            });
            model.queue.set_selected(next);
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
