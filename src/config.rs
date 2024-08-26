use crate::event_handler::Result;
use crate::model::*;
use crate::update::*;
use crate::{update::Message, view::Theme};
use phf::phf_map;
use ratatui::style::Style;
use std::env;
use std::fs;
use std::{collections::HashMap, path::PathBuf};
use toml::Table;
use toml::Value;
pub mod keybind;
use keybind::KeybindMap;

pub struct Config {
    pub keybindings: KeybindMap,
    pub theme: Theme,
}

static MESSAGES: phf::Map<&'static str, Message> = phf_map! {
    "up" => Message::Direction(Dirs::Vert(Vertical::Up)),
    "down" => Message::Direction(Dirs::Vert(Vertical::Down)),
    "left" => Message::Direction(Dirs::Horiz(Horizontal::Left)),
    "right" => Message::Direction(Dirs::Horiz(Horizontal::Right)),
    "toggle_playpause" => Message::PlayPause,
    "select" => Message::Select,
    "quit" => Message::SwitchState(State::Done),
    "switch_to_library" => Message::SwitchScreen(Screen::Library),
    "switch_to_queue" => Message::SwitchScreen(Screen::Queue),
    "toggle_screen_lq" => Message::ToggleScreen,
    "fold" => Message::Fold,
    "clear_queue" => Message::Clear,
    "local_search" => Message::LocalSearch(SearchMsg::Start),
    "global_search" => Message::GlobalSearch(SearchMsg::Start),
    "escape" => Message::Escape,
    "delete" => Message::Delete,
    "tog_repeat" => Message::Set(Toggle::Repeat),
    "tog_single" => Message::Set(Toggle::Single),
    "tog_consume" => Message::Set(Toggle::Consume),
    "tog_random" => Message::Set(Toggle::Random),
};

impl Config {
    pub fn default() -> Self {
        Config {
            keybindings: KeybindMap(HashMap::new()),
            theme: Theme::new(),
        }
    }
    pub fn try_read_config(mut self) -> Self {
        let path = match env::var("XDG_CONFIG_HOME") {
            Ok(s) => Some(PathBuf::from(s + "/inori/config.toml")),
            Err(_) => match env::var("HOME") {
                Ok(home) => {
                    Some(PathBuf::from(home + "/.config/inori/config.toml"))
                }
                Err(_) => None,
            },
        };

        if let Some(Ok(contents)) = path.map(|p| fs::read_to_string(p)) {
            let toml = contents.parse::<Table>().expect("failed to parse toml");
            for (key, value) in toml {
                match (key.as_str(), value) {
                    ("keybindings", Value::Table(t)) => self.read_keybinds(t),
                    ("theme", Value::Table(t)) => self.read_theme(t),
                    (_k, _v) => panic!("unknown key {} or value {}", _k, _v),
                }
            }
        }
        self
    }
    pub fn read_keybinds(&mut self, t: Table) {
        for (key, value) in t {
            match (MESSAGES.get(&key), value) {
                (Some(m), Value::String(s)) => {
                    let keybinds = keybind::parse_keybind(s)
                        .unwrap_or_else(|_| panic!("Couldn't parse keybinds"));
                    self.keybindings.insert(m.clone(), &keybinds);
                }
                (Some(_), other) => panic!(
                    "keybind {} for command {} must be a string",
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
                (other, _) => panic!("theme option {} not found", other),
            }
        }
    }
}

pub fn deserialize_style(mut t: Table) -> Style {
    if !t.contains_key("add_modifier") {
        t.insert("add_modifier".into(), Value::String("".into()));
    }
    if !t.contains_key("sub_modifier") {
        t.insert("sub_modifier".into(), Value::String("".into()));
    }
    t.try_into().expect("Style parse failure")
}
