use crate::event_handler::Result;
use mpd::Song;
use platform_dirs::AppDirs;
use serde::Deserialize;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const DEFAULT_ALIAS_FILE: &str = "aliases.json";

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SongPath(pub String);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AlbumName(pub String);

impl Borrow<str> for SongPath {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for AlbumName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Default)]
pub struct AliasMaps {
    pub title: HashMap<SongPath, AliasEntry>,
    pub album: HashMap<AlbumName, AliasEntry>,
}

#[derive(Clone, Default)]
pub struct AliasEntry {
    pub alias: String,
    pub variants: Vec<String>,
}

#[derive(Copy, Clone)]
pub struct AliasRef<'a> {
    pub alias: &'a str,
    pub variants: &'a [String],
}

impl<'a> From<&'a AliasEntry> for AliasRef<'a> {
    fn from(value: &'a AliasEntry) -> Self {
        Self {
            alias: &value.alias,
            variants: &value.variants,
        }
    }
}

impl AliasRef<'_> {
    pub fn to_owned(self) -> (String, Vec<String>) {
        (self.alias.to_string(), self.variants.to_vec())
    }
}

impl AliasMaps {
    pub fn title_for_path(&self, path: &str) -> Option<&AliasEntry> {
        self.title.get(path)
    }

    pub fn album_for_name(&self, album: &str) -> Option<&AliasEntry> {
        self.album.get(album)
    }

    pub fn album_for_song(&self, song: &Song) -> Option<&AliasEntry> {
        song.tags
            .iter()
            .find_map(|(k, v)| (k == "Album").then_some(v))
            .and_then(|album| self.album_for_name(album))
    }

    pub fn title_ref_for_path(&self, path: &str) -> Option<AliasRef<'_>> {
        self.title_for_path(path).map(AliasRef::from)
    }

    pub fn album_ref_for_name(&self, album: &str) -> Option<AliasRef<'_>> {
        self.album_for_name(album).map(AliasRef::from)
    }

    pub fn album_ref_for_song(&self, song: &Song) -> Option<AliasRef<'_>> {
        self.album_for_song(song).map(AliasRef::from)
    }
}

pub trait TitleAliasStore {
    fn load_all(&self) -> Result<AliasMaps>;
}

pub struct NoopTitleAliasStore;

impl TitleAliasStore for NoopTitleAliasStore {
    fn load_all(&self) -> Result<AliasMaps> {
        Ok(AliasMaps {
            title: HashMap::new(),
            album: HashMap::new(),
        })
    }
}

pub struct JsonTitleAliasStore {
    path: PathBuf,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AliasJsonEntry {
    Path {
        path: String,
        alias: String,
        #[serde(default)]
        variants: Vec<String>,
    },
    Album {
        album: String,
        alias: String,
        #[serde(default)]
        variants: Vec<String>,
    },
}

impl JsonTitleAliasStore {
    pub fn from_default_path() -> Option<Self> {
        let app_dirs = AppDirs::new(Some("inori"), true)?;
        Some(Self {
            path: app_dirs.config_dir.join(DEFAULT_ALIAS_FILE),
        })
    }
}

impl TitleAliasStore for JsonTitleAliasStore {
    fn load_all(&self) -> Result<AliasMaps> {
        let content = match fs::read_to_string(&self.path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(AliasMaps::default())
            }
            Err(e) => return Err(e.into()),
        };
        let entries: Vec<AliasJsonEntry> = serde_json::from_str(&content)?;

        let mut title = HashMap::new();
        let mut album = HashMap::new();

        for entry in entries {
            match entry {
                AliasJsonEntry::Path {
                    path,
                    alias,
                    variants,
                } => {
                    title
                        .insert(SongPath(path), AliasEntry { alias, variants });
                }
                AliasJsonEntry::Album {
                    album: album_name,
                    alias,
                    variants,
                } => {
                    album.insert(
                        AlbumName(album_name),
                        AliasEntry { alias, variants },
                    );
                }
            }
        }

        Ok(AliasMaps { title, album })
    }
}
