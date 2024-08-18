use crate::model::*;
use ratatui::prelude::*;
use ratatui::style::Color::*;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::*;
pub mod library_renderer;
pub mod queue_renderer;
mod status_renderer;

pub struct Theme {
    pub item_highlight_active: Style,
    pub item_highlight_inactive: Style,
    pub block_active: Style,
    pub status_artist: Style,
    pub status_album: Style,
    pub status_title: Style,
    pub album: Style,
    pub playing: Style,
    pub paused: Style,
    pub stopped: Style,
}

pub fn view(model: &mut Model, frame: &mut Frame) {
    // only &mut for ListState/TableState updating.
    // view function should be pure!

    let theme = Theme {
        item_highlight_active: Style::new().fg(Black).bg(White),
        item_highlight_inactive: Style::new().fg(Black).bg(DarkGray),
        block_active: Style::new().fg(Red),
        status_artist: Style::new().fg(Green),
        status_album: Style::new().fg(Cyan).italic(),
        status_title: Style::new().bold(),
        album: Style::new().bold().italic().fg(Red),
        playing: Style::new().fg(LightGreen),
        paused: Style::new().fg(LightRed),
        stopped: Style::new().fg(Red),
    };
    match model.screen {
        Screen::Library => library_renderer::render(model, frame, &theme),
        Screen::Queue => queue_renderer::render(model, frame, &theme),
        Screen::Playlist => library_renderer::render(model, frame, &theme),
    }
}
