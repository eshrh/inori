use crate::event_handler::Result;
use mpd::Song;
use platform_dirs::AppDirs;
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
        let parsed: serde_json::Value = serde_json::from_str(&content)?;

        let entries = parsed.as_array().ok_or_else(|| {
            format!(
                "{} must contain a JSON array of alias objects",
                self.path.display()
            )
        })?;

        let mut title = HashMap::new();
        let mut album = HashMap::new();

        for (idx, entry) in entries.iter().enumerate() {
            let obj = entry.as_object().ok_or_else(|| {
                format!(
                    "{} entry {} must be an object",
                    self.path.display(),
                    idx
                )
            })?;

            let alias = obj
                .get("alias")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    format!(
                        "{} entry {} must include string key \"alias\"",
                        self.path.display(),
                        idx
                    )
                })?
                .to_string();
            let variants = obj
                .get("variants")
                .map(|v| {
                    let items = v.as_array().ok_or_else(|| {
                        format!(
                            "{} entry {} key \"variants\" must be an array",
                            self.path.display(),
                            idx
                        )
                    })?;
                    let mut out = Vec::with_capacity(items.len());
                    for item in items {
                        let s = item.as_str().ok_or_else(|| {
                            format!(
                                "{} entry {} variants must contain strings",
                                self.path.display(),
                                idx
                            )
                        })?;
                        out.push(s.to_string());
                    }
                    Ok::<Vec<String>, String>(out)
                })
                .transpose()
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?
                .unwrap_or_default();
            let alias_entry = AliasEntry { alias, variants };

            let path_key = obj.get("path").and_then(|v| v.as_str());
            let album_key = obj.get("album").and_then(|v| v.as_str());

            match (path_key, album_key) {
                (Some(key), None) => {
                    title.insert(SongPath(key.to_string()), alias_entry);
                }
                (None, Some(key)) => {
                    album.insert(AlbumName(key.to_string()), alias_entry);
                }
                (Some(_), Some(_)) => {
                    return Err(format!(
                    "{} entry {} may contain only one of \"path\" or \"album\"",
                    self.path.display(),
                    idx
                )
                    .into())
                }
                (None, None) => {
                    return Err(format!(
                        "{} entry {} must contain one of \"path\" or \"album\"",
                        self.path.display(),
                        idx
                    )
                    .into())
                }
            }
        }

        Ok(AliasMaps { title, album })
    }
}
