extern crate mpd;
use mpd::error::Result;
use mpd::{Client, Song, Status};
use ratatui::widgets::*;
use std::env;
mod impl_artiststate;
mod impl_library;
mod impl_queue;
mod impl_searchstate;
pub mod selector_state;

#[derive(Clone)]
pub enum Screen {
    Library,
    Queue,
    Playlist,
}

pub enum State {
    Searching,
    Running,
    Done,
}

#[derive(Debug)]
pub struct AlbumData {
    pub expanded: bool,
    pub name: String,
    pub tracks: Vec<Song>,
}

pub enum TrackSelItem<'a> {
    Album(&'a AlbumData),
    Song(&'a Song),
}

pub struct ArtistData {
    pub name: String,
    pub fetched: bool,
    pub sort_names: Vec<String>,
    pub albums: Vec<AlbumData>,
    pub track_sel_state: ListState,
}

pub enum LibActiveSelector {
    ArtistSelector,
    TrackSelector,
}

pub struct Filter {
    pub active: bool,
    pub query: String,
}

pub struct LibraryState {
    pub search: Filter,
    pub active: LibActiveSelector,
    pub contents: Vec<ArtistData>,
    pub artist_state: ListState,
}

pub struct QueueSelector {
    pub search: Filter,
    pub contents: Vec<Song>,
    pub state: TableState,
}

pub struct PlaylistState;

pub struct Model {
    pub state: State,
    pub status: Status,
    pub conn: Client,
    pub screen: Screen,
    pub library: LibraryState,
    pub queue: QueueSelector,
    pub playlist: PlaylistState,
}

impl Model {
    pub fn new() -> Result<Self> {
        let mpd_url = format!(
            "{}:{}",
            env::var("MPD_HOST").unwrap_or_else(|_| "localhost".to_string()),
            env::var("MPD_PORT").unwrap_or_else(|_| "6600".to_string())
        );
        let mut conn = Client::connect(mpd_url.clone())
            .expect(&format!("Failed to connect to mpd client at {}", mpd_url));

        Ok(Model {
            state: State::Running,
            status: conn.status()?,
            conn,
            screen: Screen::Queue,
            library: LibraryState::new(),
            queue: QueueSelector::new(),
            playlist: PlaylistState,
        })
    }
    pub fn update_status(&mut self) -> Result<()> {
        self.status = self.conn.status()?;
        Ok(())
    }
}
