extern crate mpd;
use mpd::error::Result;
use mpd::{Client, Song, Status};
use ratatui::widgets::*;
use std::env;

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

pub struct Album {
    pub name: String,
    pub sort_name: Option<String>,
    pub tracks: Vec<Song>,
}

pub struct Artist {
    pub name: String,
    pub sort_name: Option<String>,
    pub albums: Vec<Album>,
}

pub struct LibraryState {
    pub contents: Vec<Artist>,
}

pub struct QueueState {
    pub contents: Vec<Song>,
    pub offset: usize,
    pub selection: Option<usize>,
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
            library: LibraryState { contents: vec![] },
            queue: QueueState {
                contents: vec![],
                offset: 0,
                selection: None,
            },
            playlist: PlaylistState,
        })
    }
    pub fn update_status(&mut self) -> Result<()> {
        self.status = self.conn.status()?;
        Ok(())
    }
}
