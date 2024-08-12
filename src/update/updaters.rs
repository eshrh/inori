use super::build_library;
use crate::event_handler::Result;
use crate::model::*;
use selector_state::Selector;
use std::borrow::Borrow;

pub fn update_library(model: &mut Model) -> Result<()> {
    if model.library.contents.is_empty() {
        build_library::build_library(model)?;
    }
    if !model.library.contents.is_empty() && model.library.selected().is_none()
    {
        model.library.set_selected(Some(5));
    }
    if model.library.selected().is_some() && !model.library.selected_item().unwrap().fetched {
            build_library::add_tracks(model)?;
    }
    Ok(())
}

pub fn update_queue(model: &mut Model) -> Result<()> {
    model.queue.contents = model.conn.queue().expect("failed to get queue");
    if model.queue.selected().is_none() && model.queue.len() > 0 {
        model.queue.set_selected(Some(0));
    }
    Ok(())
}

pub fn update_playlist(model: &mut Model) -> Result<()> {
    Ok(())
}
