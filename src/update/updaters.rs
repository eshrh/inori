use super::build_library;
use crate::event_handler::Result;
use crate::model::*;
use selector_state::*;
use std::borrow::Borrow;

pub fn update_library(model: &mut Model) -> Result<()> {
    model.library.watch_oob();
    if model.library.contents.is_empty() {
        build_library::build_library(model)?;
    }
    if !model.library.selected_item().is_some_and(|i| i.fetched) {
        build_library::add_tracks(model)?;
    }
    Ok(())
}

pub fn update_queue(model: &mut Model) -> Result<()> {
    model.queue.contents = model.conn.queue().expect("failed to get queue");
    if model.queue.selected().is_none()
        && !model.queue.contents_vec().is_empty()
    {
        model.queue.set_selected(Some(0));
    }
    Ok(())
}

pub fn update_playlist(model: &mut Model) -> Result<()> {
    Ok(())
}
