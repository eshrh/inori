use crate::event_handler::Result;
use crate::model::*;
use crate::update::*;
use crate::{update::Message, view::Theme};
use phf::phf_map;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::env;
use std::fs;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};
use toml::Table;
use toml::Value;

pub enum KeybindTarget {
    Map(KeybindMap),
    Msg(Message),
}
pub struct KeybindMap(HashMap<KeyEvent, KeybindTarget>);

impl KeybindMap {
    pub fn insert(&mut self, msg: Message, bind: &[KeyEvent]) {
        if bind.len() == 1 {
            self.0.insert(bind[0], KeybindTarget::Msg(msg));
            return;
        }
        if self.0.contains_key(&bind[0]) {
            match self.0.get_mut(&bind[0]).unwrap() {
                KeybindTarget::Map(m) => {
                    return m.insert(msg, &bind[1..]);
                }
                KeybindTarget::Msg(_m) => panic!("keybind shadowed"),
            }
        } else {
            self.0.insert(
                bind[0],
                KeybindTarget::Map(KeybindMap(HashMap::new())),
            );
            self.insert(msg, bind)
        }
    }
    pub fn lookup(&self, bind: &[KeyEvent]) -> Option<&KeybindTarget> {
        if bind.len() == 0 {
            return None;
        }
        if bind.len() == 1 {
            return self.0.get(&bind[0]);
        }
        match self.0.get(&bind[0]) {
            Some(KeybindTarget::Map(m)) => m.lookup(&bind[1..]),
            None => None,
            o => o,
        }
    }
}

pub struct Config {
    pub keybindings: KeybindMap,
    pub colors: Theme,
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
    "tog_repeat" => Message::Set(Toggle::Repeat),
    "tog_single" => Message::Set(Toggle::Single),
    "tog_consume" => Message::Set(Toggle::Consume),
    "tog_random" => Message::Set(Toggle::Random),
};

impl Config {
    pub fn new() -> Result<Self> {
        let config_file_path = match env::var("XDG_CONFIG_HOME") {
            Ok(s) => Some(PathBuf::from(s + "/inori/config.toml")),
            Err(_) => match env::var("HOME") {
                Ok(home) => {
                    Some(PathBuf::from(home + "/.config/inori/config.toml"))
                }
                Err(_) => None,
            },
        };
        let mut defaults = Self::defaults();
        if let Some(Ok(c)) = config_file_path.map(|p| fs::read_to_string(p)) {
            Self::read(c, &mut defaults);
        }
        Ok(defaults)
    }
    pub fn read(contents: String, config: &mut Config) {
        let toml = contents.parse::<Table>().expect("failed to parse toml");
        for (key, value) in toml {
            match (key.as_str(), value) {
                ("keybindings", Value::Table(t)) => {
                    Self::read_keybinds(t, config)
                }
                (_k, _v) => panic!("unknown key {} or value {}", _k, _v),
            }
        }
    }
    pub fn parse_keybind_single(s: &str) -> Option<KeyCode> {
        if s.len() == 1 {
            if let Some(c) = s.chars().nth(0) {
                Some(KeyCode::Char(c))
            } else {
                None
            }
        } else {
            match s {
                "<space>" => Some(KeyCode::Char(' ')),
                "<esc>" => Some(KeyCode::Esc),
                "<tab>" => Some(KeyCode::Tab),
                "<backspace>" => Some(KeyCode::Backspace),
                "<delete>" => Some(KeyCode::Delete),
                "<up>" => Some(KeyCode::Up),
                "<down>" => Some(KeyCode::Down),
                "<left>" => Some(KeyCode::Left),
                "<right>" => Some(KeyCode::Right),
                _ => None,
            }
        }
    }
    pub fn parse_keybind(s: String) -> Result<Vec<KeyEvent>> {
        let mut out: Vec<KeyEvent> = Vec::new();
        for word in s.split(' ') {
            if word.starts_with("C-") {
                out.push(KeyEvent::new(
                    Self::parse_keybind_single(&word[2..])
                        .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                    KeyModifiers::CONTROL,
                ))
            } else if word.starts_with("M-") {
                out.push(KeyEvent::new(
                    Self::parse_keybind_single(&word[2..])
                        .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                    KeyModifiers::META,
                ))
            } else if word.starts_with("S-") {
                out.push(KeyEvent::new(
                    Self::parse_keybind_single(&word[2..])
                        .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                    KeyModifiers::SUPER,
                ))
            } else if word.starts_with("C-M-") {
                out.push(KeyEvent::new(
                    Self::parse_keybind_single(&word[4..])
                        .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                    KeyModifiers::CONTROL | KeyModifiers::META,
                ))
            } else {
                out.push(KeyEvent::new(
                    Self::parse_keybind_single(&word)
                        .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                    KeyModifiers::empty(),
                ))
            }
        }
        Ok(out)
    }
    pub fn read_keybinds(t: Table, config: &mut Config) {
        for (key, value) in t {
            match (MESSAGES.get(&key), value) {
                (Some(m), Value::String(s)) => {
                    let keybinds = Self::parse_keybind(s)
                        .unwrap_or_else(|_| panic!("Couldn't parse keybinds"));
                    config.keybindings.insert(m.clone(), &keybinds);
                }
                (Some(_), other) => panic!(
                    "keybind {} for command {} must be a string",
                    other, key
                ),
                (None, _) => panic!("message {} does not exist", key),
            }
        }
    }
    pub fn defaults() -> Self {
        Config {
            keybindings: KeybindMap(HashMap::new()),
            colors: Theme::new(),
        }
    }
}
