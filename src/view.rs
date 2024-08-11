use crate::model::*;
use ratatui::prelude::*;
pub mod library_renderer;
pub mod queue_renderer;

pub fn view(model: &mut Model, frame: &mut Frame) {
    // only &mut for ListState/TableState updating.
    // view function should be pure!
    match model.screen {
        Screen::Library => library_renderer::render(model, frame),
        Screen::Queue => queue_renderer::render(model, frame),
        Screen::Playlist => library_renderer::render(model, frame),
    }
}
