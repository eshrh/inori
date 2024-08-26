use crate::model::*;
use ratatui::prelude::*;
use ratatui::style::Color::*;
use ratatui::style::Style;
mod artist_select_renderer;
pub mod library_renderer;
pub mod queue_renderer;
mod search_renderer;
mod status_renderer;
mod track_select_renderer;

pub struct Theme {
    pub item_highlight_active: Style,
    pub item_highlight_inactive: Style,
    pub block_active: Style,
    pub status_artist: Style,
    pub status_album: Style,
    pub status_title: Style,
    pub artist_sort: Style,
    pub album: Style,
    pub playing: Style,
    pub paused: Style,
    pub stopped: Style,
    pub slash_span: Style,
    pub search_query_active: Style,
    pub search_query_inactive: Style,
}
impl Theme {
    pub fn new() -> Self {
        Self {
            item_highlight_active: Style::new().fg(Black).bg(White),
            item_highlight_inactive: Style::new().fg(Black).bg(DarkGray),
            block_active: Style::new().fg(Red),
            status_artist: Style::new().fg(Cyan),
            status_title: Style::new().bold(),
            status_album: Style::new().bold().italic().fg(Red),
            album: Style::new().bold().italic().fg(Red),
            playing: Style::new().fg(LightGreen),
            paused: Style::new().fg(LightRed),
            stopped: Style::new().fg(Red),
            artist_sort: Style::new().fg(DarkGray),
            slash_span: Style::new().fg(LightMagenta),
            search_query_active: Style::new().bg(White).fg(Black),
            search_query_inactive: Style::new().bg(DarkGray).fg(Black),
        }
    }
}

pub fn view(model: &mut Model, frame: &mut Frame) {
    // only &mut for ListState/TableState updating.
    // view function should be pure!

    let theme = Theme::new();
    match model.screen {
        Screen::Library => library_renderer::render(model, frame, &theme),
        Screen::Queue => queue_renderer::render(model, frame, &theme),
    }
}
