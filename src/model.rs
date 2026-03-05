extern crate mpd;
use mpd::client::StreamTypes;
use std::error::Error;
//use mpd::error::Result;
use mpd::idle::IdleClient;
use mpd::{Client, Song, Status, Subsystem};
use nucleo_matcher::{Matcher, Utf32String};
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::widgets::*;
mod impl_album_song;
mod impl_artiststate;
mod impl_library;
mod impl_queue;
mod impl_searchstate;
pub mod proto;
mod search_utils;
use crate::config::Config;
use crate::model::proto::*;
use crate::update::build_library;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Clone, Debug)]
pub enum Screen {
    Library,
    Queue,
}

impl Screen {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "library" | "Library" => Some(Screen::Library),
            "queue" | "Queue" => Some(Screen::Queue),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
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

#[derive(Debug)]
pub enum ItemRef<'a> {
    Album(&'a AlbumData),
    Song(&'a Song),
}

#[derive(Debug)]
pub struct TrackSelItem<'a> {
    pub item: ItemRef<'a>,
    pub rank: Option<usize>,
}

pub struct ArtistData {
    pub name: String,
    pub fetched: bool,
    pub sort_names: Vec<String>,
    pub albums: Vec<AlbumData>,
    pub track_sel_state: TableState,
    pub search: Filter,
}

pub enum LibActiveSelector {
    ArtistSelector,
    TrackSelector,
}

pub struct FilterCache {
    pub query: String,
    pub order: Vec<Option<usize>>,
    pub indices: Vec<Vec<u32>>,
    pub utfstrings_cache: Option<Vec<Utf32String>>,
}

pub struct Filter {
    pub active: bool,
    pub query: String,
    pub cache: FilterCache,
}

#[derive(Clone)]
pub struct InfoEntry {
    pub artist: String,
    pub artist_sort: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
}
pub struct GlobalSearchState {
    pub search: Filter,
    pub contents: Option<Vec<InfoEntry>>,
    pub results_state: ListState,
}

pub struct LibraryState {
    pub artist_search: Filter,
    pub global_search: GlobalSearchState,
    pub active: LibActiveSelector,
    pub contents: Vec<ArtistData>,
    pub artist_state: ListState,
}

pub struct QueueSelector {
    pub search: Filter,
    pub contents: Vec<Song>,
    pub state: TableState,
}

pub struct Model {
    pub state: State,
    pub status: Status,
    pub conn: Client<StreamTypes>,
    pub idle_conn: IdleClient<StreamTypes>,
    pub screen: Screen,
    pub library: LibraryState,
    pub queue: QueueSelector,
    pub currentsong: Option<Song>,
    pub matcher: nucleo_matcher::Matcher,
    pub config: Config,
    pub parse_state: Vec<KeyEvent>,
    pub frame_size: Rect,
}

impl Model {
    pub fn new(frame_size: Rect) -> Result<Self> {
        let config = Config::default().try_read_config()?;
        let mut conn = Self::make_connection(&config)?;
        let idle_conn = IdleClient::new(
            Self::make_connection(&config)?,
            &[Subsystem::Database, Subsystem::Player, Subsystem::Options],
        )?;
        Ok(Model {
            state: State::Running,
            status: conn.status()?,
            conn,
            idle_conn,
            screen: config.screens.first().cloned().unwrap_or(Screen::Library),
            library: LibraryState::new(),
            queue: QueueSelector::new(),
            currentsong: None,
            matcher: {
                let mut default_config = nucleo_matcher::Config::DEFAULT;
                default_config.prefer_prefix = config.nucleo_prefer_prefix;
                Matcher::new(default_config)
            },
            config,
            parse_state: Vec::new(),
            frame_size,
        })
    }

    pub fn make_connection(conf: &Config) -> Result<Client<StreamTypes>> {
        if let Some(mpd_url) = &conf.mpd_address {
            Client::<StreamTypes>::connect(mpd_url).map_err(|e| {
                format!("failed to connect to MPD at {}: {}", mpd_url, e).into()
            })
        } else {
            Ok(Client::<StreamTypes>::default())
        }
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
        let mut entries = Vec::new();
        for vec in res.iter_mut() {
            let ie = InfoEntry::try_from(vec)
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;
            if !ie.is_redundant() {
                entries.push(ie);
            }
        }
        self.library.global_search.contents = Some(entries);
        self.library.global_search.search.reset_cache();
        Ok(())
    }

    pub fn jump_to(&mut self, target: InfoEntry) -> Result<()> {
        // order: albumartist albumartistsort album title
        let artist_idx = self
            .library
            .contents()
            .position(|i| i.name == target.artist);
        self.library.artist_state.set_selected(artist_idx);

        if target.album.is_some() || target.title.is_some() {
            self.library.active = LibActiveSelector::TrackSelector;
        } else {
            self.library.active = LibActiveSelector::ArtistSelector;
        }

        if target.album.is_none() {
            return Ok(());
        }
        if self.library.selected_item().is_some_and(|i| !i.fetched) {
            build_library::add_tracks(self)?;
        }
        if let Some(artist) = self.library.selected_item_mut() {
            let mut idx: Option<usize> = None;
            artist.expand_all();
            if let Some(track_name) = target.title {
                idx = artist.contents().iter().position(|i| match i.item {
                    ItemRef::Song(s) => {
                        s.title.as_ref().is_some_and(|i| *i == track_name)
                    }
                    _ => false,
                });
            } else if let Some(album_name) = target.album {
                idx = artist.contents().iter().position(|i| match i.item {
                    ItemRef::Album(a) => a.name == *album_name,
                    _ => false,
                });
            }
            artist.set_selected(idx);
        }
        Ok(())
    }
}
