extern crate mpd;
use mpd::error::Result;
use mpd::{Client, Song, Status};
use ratatui::widgets::*;
use std::collections::{BTreeMap, HashMap};
use std::env;

mod impl_library;
mod impl_queue;

#[derive(Clone)]
pub enum Screen {
    Library,
    Queue,
    Playlist,
}

#[derive(PartialEq, Eq)]
pub enum State {
    Running,
    Done,
}

pub struct AlbumData {
    pub tracks: Vec<Song>,
}

pub struct ArtistData {
    pub fetched: bool,
    pub sort_names: Vec<String>,
    pub albums: Vec<String>,
    pub contents: Option<HashMap<String, AlbumData>>,
}

pub struct LibraryState {
    pub artists: Vec<String>,
    pub contents: HashMap<String, ArtistData>,
    pub artist_state: ListState,
    pub song_state: ListState,
}

pub struct QueueState {
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
    pub queue: QueueState,
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
            library: LibraryState {
                artists: Vec::new(),
                contents: HashMap::new(),
                artist_state: ListState::default(),
                song_state: ListState::default(),
            },
            queue: QueueState {
                contents: Vec::new(),
                state: TableState::default(),
            },
            playlist: PlaylistState,
        })
    }
    pub fn update_status(&mut self) -> Result<()> {
        self.status = self.conn.status()?;
        Ok(())
    }
}
