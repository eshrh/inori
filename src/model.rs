extern crate mpd;
use mpd::error::Result;
use mpd::search::Window;
use mpd::{Client, Song, Status};
use nucleo_matcher::{Matcher, Utf32String};
use ratatui::crossterm::event::KeyEvent;
use ratatui::widgets::*;
use std::env;
use std::net::TcpStream;
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

#[cfg(unix)]
use {std::os::unix::net::UnixStream, std::path::PathBuf};

#[derive(Clone, Debug)]
pub enum Screen {
    Library,
    Queue,
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

pub enum Connection {
    #[cfg(unix)]
    UnixSocket(Client<UnixStream>),
    TcpSocket(Client<TcpStream>),
}

pub struct Model {
    pub state: State,
    pub status: Status,
    pub conn: Connection,
    pub screen: Screen,
    pub library: LibraryState,
    pub queue: QueueSelector,
    pub currentsong: Option<Song>,
    pub matcher: nucleo_matcher::Matcher,
    pub config: Config,
    pub parse_state: Vec<KeyEvent>,
    pub window_height: Option<usize>,
}

impl Model {
    pub fn new() -> Result<Self> {
        let mpd_host =
            env::var("MPD_HOST").unwrap_or_else(|_| "localhost".to_string());
        let mpd_port =
            env::var("MPD_PORT").unwrap_or_else(|_| "6600".to_string());
        let mpd_url = format!("{mpd_host}:{mpd_port}");

        #[cfg(unix)]
        let mut conn = if env::var("MPD_HOST").is_ok()
            && PathBuf::from(&mpd_host).exists()
        {
            let client = Client::<UnixStream>::connect(&mpd_host)?;
            Connection::UnixSocket(client)
        } else {
            let client = Client::<TcpStream>::connect(&mpd_url)?;
            Connection::TcpSocket(client)
        };
        #[cfg(not(unix))]
        let mut conn = {
            let client = Client::<TcpStream>::connect(&mpd_url)?;
            Connection::TcpSocket(client)
        };

        Ok(Model {
            state: State::Running,
            status: conn.status()?,
            conn,
            screen: Screen::Library,
            library: LibraryState::new(),
            queue: QueueSelector::new(),
            currentsong: None,
            matcher: {
                let mut default_config = nucleo_matcher::Config::DEFAULT;
                default_config.prefer_prefix = true;
                Matcher::new(default_config)
            },
            config: Config::default().try_read_config(),
            parse_state: Vec::new(),
            window_height: Some(100),
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

        if target.album.is_some() || target.title.is_some() {
            self.library.active = LibActiveSelector::TrackSelector;
        } else {
            self.library.active = LibActiveSelector::ArtistSelector;
        }

        if target.album.is_none() {
            return;
        }
        if self.library.selected_item().is_some_and(|i| !i.fetched) {
            build_library::add_tracks(self)
                .expect("couldn't add tracks on the fly while searching");
        }
        if let Some(artist) = self.library.selected_item_mut() {
            let mut idx: Option<usize> = None;
            artist.expand_all();
            if let Some(track_name) = target.title {
                idx = artist.contents().iter().position(|i| match i.item {
                    ItemRef::Song(s) => {
                        *s.title.as_ref().unwrap() == track_name
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
    }
}

impl Connection {
    pub(crate) fn status(&mut self) -> Result<Status> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.status(),
            Connection::TcpSocket(conn) => conn.status(),
        }
    }

    pub(crate) fn currentsong(&mut self) -> Result<Option<Song>> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.currentsong(),
            Connection::TcpSocket(conn) => conn.currentsong(),
        }
    }

    pub(crate) fn list_groups(
        &mut self,
        vec: Vec<&str>,
    ) -> Result<Vec<Vec<String>>> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.list_groups(vec),
            Connection::TcpSocket(conn) => conn.list_groups(vec),
        }
    }

    pub(crate) fn queue(&mut self) -> Result<Vec<Song>> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.queue(),
            Connection::TcpSocket(conn) => conn.queue(),
        }
    }

    pub(crate) fn delete(&mut self, p: u32) -> Result<()> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.delete(p),
            Connection::TcpSocket(conn) => conn.delete(p),
        }
    }

    pub(crate) fn swap(&mut self, p: u32, to: u32) -> Result<()> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.swap(p, to),
            Connection::TcpSocket(conn) => conn.swap(p, to),
        }
    }

    pub(crate) fn switch(&mut self, pos: u32) -> Result<()> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.switch(pos),
            Connection::TcpSocket(conn) => conn.switch(pos),
        }
    }

    pub(crate) fn findadd(&mut self, query: &mut mpd::Query) -> Result<()> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.findadd(query),
            Connection::TcpSocket(conn) => conn.findadd(query),
        }
    }

    pub(crate) fn find<W>(
        &mut self,
        query: &mut mpd::Query,
        window: W,
    ) -> Result<Vec<Song>>
    where
        W: Into<Window>,
    {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.find(query, window),
            Connection::TcpSocket(conn) => conn.find(query, window),
        }
    }

    pub(crate) fn list_group_2(
        &mut self,
        terms: (String, String),
    ) -> Result<Vec<(String, String)>> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.list_group_2(terms),
            Connection::TcpSocket(conn) => conn.list_group_2(terms),
        }
    }

    pub(crate) fn clear(&mut self) -> Result<()> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.clear(),
            Connection::TcpSocket(conn) => conn.clear(),
        }
    }

    pub(crate) fn consume(&mut self, value: bool) -> Result<()> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.consume(value),
            Connection::TcpSocket(conn) => conn.consume(value),
        }
    }

    pub(crate) fn random(&mut self, value: bool) -> Result<()> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.random(value),
            Connection::TcpSocket(conn) => conn.random(value),
        }
    }

    pub(crate) fn single(&mut self, value: bool) -> Result<()> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.single(value),
            Connection::TcpSocket(conn) => conn.single(value),
        }
    }

    pub(crate) fn repeat(&mut self, value: bool) -> Result<()> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.repeat(value),
            Connection::TcpSocket(conn) => conn.repeat(value),
        }
    }

    pub(crate) fn toggle_pause(&mut self) -> Result<()> {
        match self {
            #[cfg(unix)]
            Connection::UnixSocket(conn) => conn.toggle_pause(),
            Connection::TcpSocket(conn) => conn.toggle_pause(),
        }
    }
}
