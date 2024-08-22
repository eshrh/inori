extern crate mpd;
use mpd::error::Result;
use mpd::{Client, Song, Status};
use ratatui::widgets::*;
use std::env;
mod impl_album_song;
mod impl_artiststate;
mod impl_library;
mod impl_queue;
mod impl_searchstate;
pub mod selector_state;
use crate::model::selector_state::*;
use crate::update::build_library;

#[derive(Clone)]
pub enum Screen {
    Library,
    Queue
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
    pub track_sel_state: TableState,
}

pub enum LibActiveSelector {
    ArtistSelector,
    TrackSelector,
}

pub struct FilterCache {
    pub query: String,
    pub order: Vec<Option<usize>>,
    pub indices: Vec<Option<Vec<u32>>>,
}

pub struct Filter {
    pub active: bool,
    pub query: String,
    pub cache: FilterCache,
}

#[derive(Clone, Debug)]
pub struct InfoEntry {
    pub artist: String,
    pub artist_sort: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
}
pub struct GlobalSearchState {
    pub search: Filter,
    pub matcher: nucleo_matcher::Matcher,
    pub contents: Option<Vec<InfoEntry>>,
    pub results_state: ListState,
}

pub struct LibraryState {
    pub artist_search: Filter,
    pub global_search: GlobalSearchState,
    pub active: LibActiveSelector,
    pub contents: Vec<ArtistData>,
    pub artist_state: ListState,
    pub matcher: nucleo_matcher::Matcher,
}

pub struct QueueSelector {
    pub search: Filter,
    pub contents: Vec<Song>,
    pub state: TableState,
}

pub struct Model {
    pub state: State,
    pub status: Status,
    pub conn: Client,
    pub screen: Screen,
    pub library: LibraryState,
    pub queue: QueueSelector,
    pub currentsong: Option<Song>,
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
            screen: Screen::Library,
            library: LibraryState::new(),
            queue: QueueSelector::new(),
            currentsong: None,
        })
    }
    pub fn update_status(&mut self) -> Result<()> {
        self.status = self.conn.status()?;
        Ok(())
    }
    pub fn update_currentsong(&mut self) -> Result<()> {
        self.currentsong = self.conn.currentsong()?;
        Ok(())
    }

    pub fn update_global_search_contents(&mut self) -> Result<()> {
        let mut res = self.conn.list_groups(vec![
            "title",
            "album",
            "albumartistsort",
            "albumartist",
        ])?;
        self.library.global_search.contents = Some(
            res.iter_mut()
                .filter_map(|vec| {
                    let ie = InfoEntry::from(vec);
                    if !ie.is_redundant() {
                        Some(ie)
                    } else {
                        None
                    }
                })
                .collect::<Vec<InfoEntry>>(),
        );
        Ok(())
    }

    pub fn jump_to(&mut self, target: InfoEntry) {
        // order: albumartist albumartistsort album title
        let artist_idx = self
            .library
            .contents()
            .position(|i| i.name == target.artist);
        self.library.artist_state.set_selected(artist_idx);

        if target.album.is_none() {
            return;
        }
        // 3 -> album
        // 4 -> track
        if self.library.selected_item().is_some_and(|i| !i.fetched) {
            build_library::add_tracks(self)
                .expect("couldn't add tracks on the fly while searching");
        }
        if target.album.is_some() || target.title.is_some() {
            self.library.active = LibActiveSelector::TrackSelector;
        }
        if let Some(artist) = self.library.selected_item_mut() {
            let mut idx: Option<usize> = None;
            if let Some(track_name) = target.title {
                idx = artist.contents().iter().position(|i| match i {
                    TrackSelItem::Song(s) => {
                        *s.title.as_ref().unwrap() == track_name
                    }
                    _ => false,
                });
            } else {
                if let Some(album_name) = target.album {
                    idx = artist.contents().iter().position(|i| match i {
                        TrackSelItem::Album(a) => a.name == *album_name,
                        _ => false,
                    });
                }
            }
            artist.set_selected(idx);
        }
    }
}
