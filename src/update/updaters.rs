use super::build_library;
use crate::event_handler::Result;
use crate::model::*;
use proto::*;

pub fn update_library(model: &mut Model) -> Result<()> {
    model.library.watch_oob();
    if model.library.contents.is_empty() {
        build_library::build_library(model)?;
    }
    if model.library.len() != 0 && model.library.selected().is_none() {
        model.library.set_selected(Some(0))
    }
    if !model.library.selected_item().is_some_and(|i| i.fetched) {
        build_library::add_tracks(model)?;
    }
    Ok(())
}

pub fn update_queue(model: &mut Model) -> Result<()> {
    if model.queue.selected().is_none()
        && !model.queue.contents_vec().is_empty()
    {
        model.queue.set_selected(Some(0));
    }
    if model.queue.contents.is_empty() {
        model.queue.set_selected(None);
    }
    Ok(())
}
