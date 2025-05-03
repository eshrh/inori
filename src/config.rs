use crate::model::*;
use crate::view::Theme;
use platform_dirs::AppDirs;
use ratatui::style::Modifier;
use ratatui::style::Style;
use std::fs;
use toml::Table;
use toml::Value;
pub mod keybind;
use keybind::{get_message, KeybindMap};

pub struct Config {
    pub keybindings: KeybindMap,
    pub theme: Theme,
    pub seek_seconds: i64,
    pub mpd_address: Option<String>,
}

impl Config {
    pub fn default() -> Self {
        Config {
            keybindings: KeybindMap::default(),
            theme: Theme::new(),
            seek_seconds: 5,
            mpd_address: None,
        }
    }
    pub fn try_read_config(mut self) -> Self {
        let app_dirs = AppDirs::new(Some("inori"), true);
        let config_file_path =
            app_dirs.map(|d| d.config_dir.join("config.toml"));

        if let Some(Ok(contents)) = config_file_path.map(fs::read_to_string) {
            let toml = contents.parse::<Table>().expect("failed to parse toml");
            for (key, value) in toml {
                match (key.as_str(), value) {
                    ("keybindings", Value::Table(t)) => self.read_keybinds(t),
                    ("seek_seconds", Value::Integer(k)) if k > 0 => {
                        self.seek_seconds = k
                    }
                    ("theme", Value::Table(t)) => self.read_theme(t),
                    ("dvorak_keybindings", Value::Boolean(true)) => {
                        self.keybindings = self.keybindings.with_dvorak_style();
                    }
                    ("qwerty_keybindings", Value::Boolean(true)) => {
                        self.keybindings = self.keybindings.with_qwerty_style();
                    }
                    ("mpd_address", Value::String(addr)) => {
                        self.mpd_address = Some(addr);
                    }
                    (_k, _v) => panic!("unknown key {} or value {}", _k, _v),
                }
            }
        }
        self
    }
    pub fn read_keybinds(&mut self, t: Table) {
        for (key, value) in t {
            match (get_message(&key), value) {
                (Some(m), Value::String(s)) => {
                    let keybinds = keybind::parse_keybind(s).unwrap();
                    self.keybindings.insert(m.clone(), &keybinds);
                }
                (Some(m), Value::Array(a)) => {
                    for s in a {
                        if let Value::String(s) = s {
                            let keybinds = keybind::parse_keybind(s).unwrap();
                            self.keybindings.insert(m.clone(), &keybinds);
                        } else {
                            panic!(
                                "keybind {} for command {} must be a string",
                                s, key
                            )
                        }
                    }
                }
                (Some(_), other) => panic!(
                    "keybind {} for command {} must be a string or an array",
                    other, key
                ),
                (None, _) => panic!("message {} does not exist", key),
            }
        }
    }
    pub fn read_theme(&mut self, t: Table) {
        for (key, value) in t {
            match (key.as_str(), value) {
                ("item_highlight_active", Value::Table(t)) => {
                    self.theme.item_highlight_active = deserialize_style(t);
                }
                ("item_highlight_inactive", Value::Table(t)) => {
                    self.theme.item_highlight_inactive = deserialize_style(t);
                }
                ("block_active", Value::Table(t)) => {
                    self.theme.block_active = deserialize_style(t);
                }
                ("status_artist", Value::Table(t)) => {
                    self.theme.status_artist = deserialize_style(t);
                }
                ("status_album", Value::Table(t)) => {
                    self.theme.status_album = deserialize_style(t);
                }
                ("status_title", Value::Table(t)) => {
                    self.theme.status_title = deserialize_style(t);
                }
                ("artist_sort", Value::Table(t)) => {
                    self.theme.artist_sort = deserialize_style(t);
                }
                ("album", Value::Table(t)) => {
                    self.theme.album = deserialize_style(t);
                }
                ("playing", Value::Table(t)) => {
                    self.theme.playing = deserialize_style(t);
                }
                ("paused", Value::Table(t)) => {
                    self.theme.paused = deserialize_style(t);
                }
                ("stopped", Value::Table(t)) => {
                    self.theme.stopped = deserialize_style(t);
                }
                ("slash_span", Value::Table(t)) => {
                    self.theme.slash_span = deserialize_style(t);
                }
                ("search_query_active", Value::Table(t)) => {
                    self.theme.search_query_active = deserialize_style(t);
                }
                ("search_query_inactive", Value::Table(t)) => {
                    self.theme.search_query_inactive = deserialize_style(t);
                }
                ("progress_bar_filled", Value::Table(t)) => {
                    self.theme.progress_bar_filled = deserialize_style(t)
                }
                ("progress_bar_unfilled", Value::Table(t)) => {
                    self.theme.progress_bar_unfilled = deserialize_style(t)
                }
                (other, _) => panic!("theme option {} not found", other),
            }
        }
    }
}

fn modifiers_arr(modifiers: &Vec<Value>) -> String {
    let mut m = String::new();
    for i in modifiers {
        if let Value::String(s) = i {
            if Modifier::from_name(s).is_some() {
                if !m.is_empty() {
                    m.push('|');
                }
                m.push_str(s);
            } else {
                panic!(
                    "Error while parsing theme modifier array: unknown modifier \"{}\""
                    ,s
                )
            }
        }
    }
    m
}

pub fn deserialize_style(mut t: Table) -> Style {
    if !t.contains_key("add_modifier") {
        t.insert("add_modifier".into(), Value::String("".into()));
    }
    if !t.contains_key("sub_modifier") {
        t.insert("sub_modifier".into(), Value::String("".into()));
    }
    if let Value::Array(a) = t.get("add_modifier").unwrap() {
        t.insert("add_modifier".into(), Value::String(modifiers_arr(a)));
    }
    if let Value::Array(a) = t.get("sub_modifier").unwrap() {
        t.insert("sub_modifier".into(), Value::String(modifiers_arr(a)));
    }
    t.try_into().expect("Style parse failure")
}
