use super::*;
use crate::view::layout::queue_layout::QueueLayout;
use crate::view::layout::InoriLayout;

pub fn handle_queue(model: &mut Model, msg: Message) -> Option<Action> {
    match msg {
        Message::ToggleScreen => {
            model.screen = Screen::Library;
            None
        }
        Message::Direction(Dirs::Vert(d)) => {
            handle_vertical(d, &mut model.queue);
            None
        }
        Message::ScrollScreenful(v) => {
            let k = QueueLayout::new(model.frame_size, model).queue.height;
            scroll_screenful(v, k.into(), &mut model.queue);
            None
        }
        Message::Select => model
            .queue
            .selected_id()
            .map(|id| Action::new(Command::Play(id))),
        Message::Direction(Dirs::Horiz(d)) => {
            let index = model.queue.selected()?;
            let to = match d {
                Horizontal::Left => index.checked_sub(1),
                Horizontal::Right => index.checked_add(1),
            };
            let (from, to) = model.queue.id_at(index).zip(
                to.filter(|&i| i < model.queue.display_len())
                    .and_then(|i| model.queue.id_at(i)),
            )?;
            Some(Action::new(Command::Swap(from, to)))
        }
        Message::Delete => {
            Some(Action::new(Command::Delete(model.queue.selected_id()?)))
        }
        Message::LocalSearch(SearchMsg::Start) => {
            model.queue.songs.filter.set_on();
            model.state = State::Searching;
            if model.queue.len() != 0 {
                model.queue.set_selected(Some(0));
            }
            None
        }
        Message::LocalSearch(SearchMsg::End) => {
            model.state = State::Running;
            None
        }
        Message::Escape => {
            model.queue.songs.filter.set_off();
            None
        }
        _ => None,
    }
}

pub fn handle_search(model: &mut Model, k: KeyEvent) -> Option<Action> {
    if let SearchEditOutcome::Message(m) = handle_search_k(
        &mut model.queue,
        k,
        &mut model.matcher,
        model.frame_size.height.into(),
    ) {
        handle_msg(model, m)
    } else {
        None
    }
}
