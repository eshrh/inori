use super::build_library;
use crate::event_handler::Result;
use crate::model::*;

pub fn update_library(model: &mut Model) -> Result<()> {
    if model.library.contents.is_empty() {
        build_library::build_library(model)?;
    }

    Ok(())
}

pub fn update_queue(model: &mut Model) -> Result<()> {
    model.queue.contents = model.conn.queue().expect("failed to get queue");
    if model.queue.selection.is_none() && model.queue.len() > 0 {
        model.queue.selection = Some(0)
    }
    Ok(())
}

pub fn update_playlist(model: &mut Model) -> Result<()> {
    Ok(())
}
