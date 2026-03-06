extern crate mpd;
use mpd::client::StreamTypes;
use std::collections::HashSet;
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
use crate::title_alias::{
    AliasMaps, JsonTitleAliasStore, NoopTitleAliasStore, TitleAliasStore,
};
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
    pub alias: Option<String>,
    pub alias_variants: Vec<String>,
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

pub struct FilteredView<T> {
    pub filter: Filter,
    pub items: Vec<T>,
}

impl<T> FilteredView<T> {
    pub fn new() -> Self {
        Self {
            filter: Filter::new(),
            items: Vec::new(),
        }
    }

    pub fn replace_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.filter.reset_cache();
    }
}

#[derive(Clone)]
pub struct InfoEntry {
    pub file: String,
    pub artist: String,
    pub artist_sort: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub album_alias: Option<String>,
    pub album_alias_variants: Vec<String>,
    pub title_alias: Option<String>,
    pub title_alias_variants: Vec<String>,
}
pub struct GlobalSearchState {
    pub entries: FilteredView<InfoEntry>,
    pub loaded: bool,
    pub results_state: ListState,
}

pub struct LibraryState {
    pub artists: FilteredView<ArtistData>,
    pub global_search: GlobalSearchState,
    pub active: LibActiveSelector,
    pub artist_state: ListState,
}

pub struct QueueSelector {
    pub songs: FilteredView<Song>,
    pub state: TableState,
    pub aliases: AliasMaps,
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
    pub title_alias_store: Box<dyn TitleAliasStore>,
    pub aliases: AliasMaps,
}

impl Model {
    pub fn new(frame_size: Rect) -> Result<Self> {
        let config = Config::default().try_read_config()?;
        let mut conn = Self::make_connection(&config)?;
        let idle_conn = IdleClient::new(
            Self::make_connection(&config)?,
            &[Subsystem::Database, Subsystem::Player, Subsystem::Options],
        )?;
        let title_alias_store: Box<dyn TitleAliasStore> =
            JsonTitleAliasStore::from_default_path()
                .map(|store| Box::new(store) as Box<dyn TitleAliasStore>)
                .unwrap_or_else(|| Box::new(NoopTitleAliasStore));
        let aliases = title_alias_store.load_all()?;

        let mut queue = QueueSelector::new();
        queue.set_aliases(&aliases);

        Ok(Model {
            state: State::Running,
            status: conn.status()?,
            conn,
            idle_conn,
            screen: config.screens.first().cloned().unwrap_or(Screen::Library),
            library: LibraryState::new(),
            queue,
            currentsong: None,
            matcher: {
                let mut default_config = nucleo_matcher::Config::DEFAULT;
                default_config.prefer_prefix = config.nucleo_prefer_prefix;
                Matcher::new(default_config)
            },
            config,
            parse_state: Vec::new(),
            frame_size,
            title_alias_store,
            aliases,
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

    pub fn reload_aliases(&mut self) -> Result<()> {
        self.aliases = self.title_alias_store.load_all()?;
        self.queue.set_aliases(&self.aliases);
        for artist in &mut self.library.artists.items {
            for album in &mut artist.albums {
                if let Some(entry) = self.aliases.album_for_name(&album.name) {
                    album.alias = Some(entry.alias.clone());
                    album.alias_variants = entry.variants.clone();
                } else {
                    album.alias = None;
                    album.alias_variants.clear();
                }
            }
            artist.search.reset_cache();
        }
        Ok(())
    }

    pub fn update_global_search_contents(&mut self) -> Result<()> {
        let res = self.conn.listallinfo()?;
        let mut entries = Vec::new();
        let mut seen_albums: HashSet<(String, Option<String>, String)> =
            HashSet::new();
        for song in res {
            let artist = song
                .tags
                .iter()
                .find_map(|(k, v)| (k == "AlbumArtist").then(|| v.clone()))
                .or(song.artist.clone());
            let Some(artist) = artist else {
                continue;
            };
            let artist_sort = song
                .tags
                .iter()
                .find_map(|(k, v)| (k == "AlbumArtistSort").then(|| v.clone()));
            let album = song
                .tags
                .iter()
                .find_map(|(k, v)| (k == "Album").then(|| v.clone()));
            let album_alias_entry = self.aliases.album_for_song(&song);
            let album_alias =
                album_alias_entry.map(|entry| entry.alias.clone());
            let album_alias_variants = album_alias_entry
                .map_or_else(Vec::new, |entry| entry.variants.clone());
            let title_alias_entry = self.aliases.title_for_path(&song.file);
            let title_alias =
                title_alias_entry.map(|entry| entry.alias.clone());
            let title_alias_variants = title_alias_entry
                .map_or_else(Vec::new, |entry| entry.variants.clone());

            if let Some(album_name) = album.clone() {
                let album_key =
                    (artist.clone(), artist_sort.clone(), album_name.clone());
                if seen_albums.insert(album_key) {
                    entries.push(InfoEntry {
                        file: String::new(),
                        artist: artist.clone(),
                        artist_sort: artist_sort.clone(),
                        album: Some(album_name),
                        title: None,
                        album_alias: album_alias.clone(),
                        album_alias_variants: album_alias_variants.clone(),
                        title_alias: None,
                        title_alias_variants: Vec::new(),
                    });
                }
            }

            let ie = InfoEntry {
                file: song.file.clone(),
                artist,
                artist_sort,
                album,
                album_alias,
                album_alias_variants,
                title: song.title.clone(),
                title_alias,
                title_alias_variants,
            };
            if !ie.is_redundant() {
                entries.push(ie);
            }
        }
        self.library.global_search.entries.replace_items(entries);
        self.library.global_search.loaded = true;
        Ok(())
    }

    pub fn jump_to(&mut self, target: InfoEntry) -> Result<()> {
        // order: albumartist albumartistsort album title
        let artist_idx = (0..self.library.display_len()).find(|&i| {
            self.library
                .display_get(i)
                .is_some_and(|artist| artist.name == target.artist)
        });
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
            if !target.file.is_empty() {
                idx = artist.contents().iter().position(|i| match i.item {
                    ItemRef::Song(s) => s.file == target.file,
                    _ => false,
                });
            }
            if idx.is_none() {
                if let Some(track_name) = target.title.as_ref() {
                    idx = artist.contents().iter().position(|i| match i.item {
                        ItemRef::Song(s) => {
                            s.title.as_ref() == Some(track_name)
                        }
                        _ => false,
                    });
                }
            }
            if idx.is_none() {
                if let Some(album_name) = target.album.as_ref() {
                    idx = artist.contents().iter().position(|i| match i.item {
                        ItemRef::Album(a) => a.name == *album_name,
                        _ => false,
                    });
                }
            }
            artist.set_selected(idx);
        }
        Ok(())
    }
}
