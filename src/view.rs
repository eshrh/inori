use crate::config::deserialize_style;
use crate::config::ConfigError;
use crate::model::*;
use ratatui::prelude::*;
use ratatui::style::Color::*;
use ratatui::style::Style;
use std::error::Error;
use toml::Table;
use toml::Value;
mod artist_select_renderer;
pub mod layout;
pub mod library_renderer;
pub mod queue_renderer;
pub mod search_doc;
pub mod search_renderable;
mod search_renderer;
mod status_renderer;
mod track_select_renderer;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Clone, Default)]
pub struct Theme {
    pub block_active: Style,
    pub field_album: Style,
    pub field_artistsort: Style,
    pub item_highlight_active: Style,
    pub item_highlight_inactive: Style,
    pub progress_bar_filled: Style,
    pub progress_bar_unfilled: Style,
    pub search_query_active: Style,
    pub search_query_inactive: Style,
    pub slash_span: Style,
    pub status_album: Style,
    pub status_artist: Style,
    pub status_paused: Style,
    pub status_playing: Style,
    pub status_stopped: Style,
    pub status_title: Style,
}
impl Theme {
    pub fn new() -> Self {
        Self {
            block_active: Style::default().fg(Red),
            field_album: Style::default().bold().italic().fg(Red),
            field_artistsort: Style::default().fg(DarkGray),
            item_highlight_active: Style::default().fg(Black).bg(White),
            item_highlight_inactive: Style::default().fg(Black).bg(DarkGray),
            progress_bar_filled: Style::default()
                .fg(LightYellow)
                .bg(Black)
                .add_modifier(Modifier::BOLD),
            progress_bar_unfilled: Style::default().fg(Black),
            search_query_active: Style::default().bg(White).fg(Black),
            search_query_inactive: Style::default().bg(DarkGray).fg(Black),
            slash_span: Style::default().fg(LightMagenta),
            status_album: Style::default().bold().italic().fg(Red),
            status_artist: Style::default().fg(Cyan),
            status_paused: Style::default().fg(LightRed),
            status_playing: Style::default().fg(LightGreen),
            status_stopped: Style::default().fg(Red),
            status_title: Style::default().bold(),
        }
    }

    pub fn apply_theme(mut self, value: Table) -> Result<Self> {
        for (key, value) in value {
            match (key.as_str(), value) {
                ("item_highlight_active", Value::Table(t)) => {
                    self.item_highlight_active = deserialize_style(t)?;
                }
                ("item_highlight_inactive", Value::Table(t)) => {
                    self.item_highlight_inactive = deserialize_style(t)?;
                }
                ("block_active", Value::Table(t)) => {
                    self.block_active = deserialize_style(t)?;
                }
                ("status_artist", Value::Table(t)) => {
                    self.status_artist = deserialize_style(t)?;
                }
                ("status_album", Value::Table(t)) => {
                    self.status_album = deserialize_style(t)?;
                }
                ("status_title", Value::Table(t)) => {
                    self.status_title = deserialize_style(t)?;
                }
                ("artist_sort", Value::Table(t)) => {
                    self.field_artistsort = deserialize_style(t)?;
                }
                ("field_artistsort", Value::Table(t)) => {
                    self.field_artistsort = deserialize_style(t)?;
                }
                ("album", Value::Table(t)) => {
                    self.field_album = deserialize_style(t)?;
                }
                ("field_album", Value::Table(t)) => {
                    self.field_album = deserialize_style(t)?;
                }
                ("playing", Value::Table(t)) => {
                    self.status_playing = deserialize_style(t)?;
                }
                ("paused", Value::Table(t)) => {
                    self.status_paused = deserialize_style(t)?;
                }
                ("stopped", Value::Table(t)) => {
                    self.status_stopped = deserialize_style(t)?;
                }
                ("status_playing", Value::Table(t)) => {
                    self.status_playing = deserialize_style(t)?;
                }
                ("status_paused", Value::Table(t)) => {
                    self.status_paused = deserialize_style(t)?;
                }
                ("status_stopped", Value::Table(t)) => {
                    self.status_stopped = deserialize_style(t)?;
                }
                ("slash_span", Value::Table(t)) => {
                    self.slash_span = deserialize_style(t)?;
                }
                ("search_query_active", Value::Table(t)) => {
                    self.search_query_active = deserialize_style(t)?;
                }
                ("search_query_inactive", Value::Table(t)) => {
                    self.search_query_inactive = deserialize_style(t)?;
                }
                ("progress_bar_filled", Value::Table(t)) => {
                    self.progress_bar_filled = deserialize_style(t)?;
                }
                ("progress_bar_unfilled", Value::Table(t)) => {
                    self.progress_bar_unfilled = deserialize_style(t)?;
                }
                (other, _) => {
                    return Err(Box::new(ConfigError::UnknownThemeOption(
                        other.to_string(),
                    )))
                }
            }
        }
        Ok(self)
    }
}

impl TryFrom<Table> for Theme {
    type Error = Box<dyn Error>;

    fn try_from(value: Table) -> Result<Self> {
        Self::default().apply_theme(value)
    }
}

pub fn view(model: &mut Model, frame: &mut Frame) {
    // only &mut for ListState/TableState updating.
    // view function should be pure!

    let theme = model.config.theme.clone();
    match model.screen {
        Screen::Library => library_renderer::render(model, frame, &theme),
        Screen::Queue => queue_renderer::render(model, frame, &theme),
    }
}
