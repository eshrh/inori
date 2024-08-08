use crate::model::*;
use ratatui::prelude::*;
pub mod library_renderer;
pub mod queue_renderer;

pub fn view(model: &Model, frame: &mut Frame) {
    match model.screen {
        Screen::Library => library_renderer::render(model, frame),
        Screen::Queue => queue_renderer::render(model, frame),
        Screen::Playlist => library_renderer::render(model, frame),
    }
}
