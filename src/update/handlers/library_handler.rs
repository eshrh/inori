use super::*;
use crate::event_handler::Result;
use crate::model::selector_state::*;
use crate::model::LibActiveSelector::*;

pub fn handle_library(model: &mut Model, msg: Message) -> Result<()> {
    match model.library.active {
        ArtistSelector => handle_library_artist(model, msg),
        TrackSelector => handle_library_track(model, msg),
    }
}

pub fn handle_library_artist(model: &mut Model, msg: Message) -> Result<()> {
    match msg {
        Message::Direction(Dirs::Vert(d)) => {
            handle_vertical(d, &mut model.library)
        }
        Message::Tab => model.library.active = TrackSelector,
        _ => (),
    }
    Ok(())
}

pub fn handle_library_track(model: &mut Model, msg: Message) -> Result<()> {
    match msg {
        Message::Direction(Dirs::Vert(d)) => {
            handle_vertical(d, model.library.selected_item_mut().unwrap())
        }
        Message::Tab => model.library.active = ArtistSelector,
        _ => (),
    }
    Ok(())
}
