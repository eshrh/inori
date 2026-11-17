use crate::model::*;
use proto::*;

pub fn update_library(model: &mut Model) {
    model.library.watch_oob();
    if model.library.len() != 0 && model.library.selected().is_none() {
        model.library.set_selected(Some(0))
    }
}

pub fn update_queue(model: &mut Model) {
    if model.queue.selected().is_none() && model.queue.display_len() != 0 {
        model.queue.set_selected(Some(0));
    }
    if model.queue.songs.items.is_empty() {
        model.queue.set_selected(None);
    }
}
