use crate::event_handler::Result;
use crate::model::{Screen, State};
use crate::update::Message::{self, *};
use crate::update::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

pub enum KeybindTarget {
    Map(KeybindMap),
    Msg(Message),
}
use KeybindTarget::*;

pub struct KeybindMap(pub HashMap<KeyEvent, KeybindTarget>);

#[cfg(all(feature = "dvorak_movement_keys", feature = "qwerty_movement_keys"))]
compile_error!("Cannot have both default dvorak and qwerty movement keys");

pub fn get_message(s: &str) -> Option<Message> {
    match s {
        "up" => Some(Message::Direction(Dirs::Vert(Vertical::Up))),
        "down" => Some(Message::Direction(Dirs::Vert(Vertical::Down))),
        "top" => Some(Message::Direction(Dirs::Vert(Vertical::Top))),
        "bottom" => Some(Message::Direction(Dirs::Vert(Vertical::Bottom))),
        "left" => Some(Message::Direction(Dirs::Horiz(Horizontal::Left))),
        "right" => Some(Message::Direction(Dirs::Horiz(Horizontal::Right))),
        "toggle_playpause" => Some(Message::PlayPause),
        "select" => Some(Message::Select),
        "quit" => Some(Message::SwitchState(State::Done)),
        "switch_to_library" => Some(Message::SwitchScreen(Screen::Library)),
        "switch_to_queue" => Some(Message::SwitchScreen(Screen::Queue)),
        "toggle_screen_lq" => Some(Message::ToggleScreen),
        "toggle_panel" => Some(Message::TogglePanel),
        "fold" => Some(Message::Fold),
        "clear_queue" => Some(Message::Clear),
        "local_search" => Some(Message::LocalSearch(SearchMsg::Start)),
        "global_search" => Some(Message::GlobalSearch(SearchMsg::Start)),
        "escape" => Some(Message::Escape),
        "delete" => Some(Message::Delete),
        "toggle_repeat" => Some(Message::Set(Toggle::Repeat)),
        "toggle_single" => Some(Message::Set(Toggle::Single)),
        "toggle_consume" => Some(Message::Set(Toggle::Consume)),
        "toggle_random" => Some(Message::Set(Toggle::Random)),
        _ => None,
    }
}

impl KeybindMap {
    pub fn default() -> Self {
        let mut keybindings = HashMap::new();
        let empty = KeyModifiers::empty();
        #[cfg(feature = "dvorak_movement_keys")]
        {
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('t'), empty),
                Msg(Direction(Dirs::Vert(Vertical::Up))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('h'), empty),
                Msg(Direction(Dirs::Vert(Vertical::Down))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('d'), empty),
                Msg(Direction(Dirs::Horiz(Horizontal::Left))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('n'), empty),
                Msg(Direction(Dirs::Horiz(Horizontal::Left))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('<'), empty),
                Msg(Direction(Dirs::Vert(Vertical::Top))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('>'), empty),
                Msg(Direction(Dirs::Vert(Vertical::Bottom))),
            );
        }
        #[cfg(feature = "qwerty_movement_keys")]
        {
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('k'), empty),
                Msg(Direction(Dirs::Vert(Vertical::Up))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('j'), empty),
                Msg(Direction(Dirs::Vert(Vertical::Down))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('h'), empty),
                Msg(Direction(Dirs::Horiz(Horizontal::Left))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('l'), empty),
                Msg(Direction(Dirs::Horiz(Horizontal::Left))),
            );
        }
        keybindings
            .insert(KeyEvent::new(KeyCode::Char('p'), empty), Msg(PlayPause));
        keybindings.insert(KeyEvent::new(KeyCode::Enter, empty), Msg(Select));
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('1'), empty),
            Msg(SwitchScreen(super::Screen::Library)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('2'), empty),
            Msg(SwitchScreen(super::Screen::Queue)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('q'), empty),
            Msg(SwitchState(super::State::Done)),
        );
        keybindings
            .insert(KeyEvent::new(KeyCode::Backspace, empty), Msg(Delete));
        keybindings
            .insert(KeyEvent::new(KeyCode::Tab, empty), Msg(ToggleScreen));
        keybindings.insert(KeyEvent::new(KeyCode::Char(' '), empty), Msg(Fold));
        keybindings
            .insert(KeyEvent::new(KeyCode::Char('-'), empty), Msg(Clear));
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('/'), empty),
            Msg(LocalSearch(SearchMsg::Start)),
        );
        keybindings.insert(KeyEvent::new(KeyCode::Esc, empty), Msg(Escape));
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('r'), empty),
            Msg(Set(Toggle::Repeat)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('z'), empty),
            Msg(Set(Toggle::Random)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('s'), empty),
            Msg(Set(Toggle::Single)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('c'), empty),
            Msg(Set(Toggle::Consume)),
        );

        if cfg!(feature = "dvorak_movement_keys") {
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('g'), empty),
                Msg(GlobalSearch(SearchMsg::Start)),
            );
            Self(keybindings)
        } else {
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('G'), empty),
                Msg(Direction(Dirs::Vert(Vertical::Bottom))),
            );
            keybindings.insert(
                KeyEvent::new(KeyCode::Char('G'), KeyModifiers::CONTROL),
                Msg(GlobalSearch(SearchMsg::Start)),
            );
            let mut k = Self(keybindings);
            k.insert(
                Direction(Dirs::Vert(Vertical::Top)),
                &[KeyEvent::new(KeyCode::Char('g'), empty),
                    KeyEvent::new(KeyCode::Char('g'), empty)],
            );
            k
        }
    }
    pub fn insert(&mut self, msg: Message, bind: &[KeyEvent]) {
        if bind.len() == 1 {
            self.0.insert(bind[0], KeybindTarget::Msg(msg));
            return;
        }
        if self.0.contains_key(&bind[0]) {
            match self.0.get_mut(&bind[0]).unwrap() {
                KeybindTarget::Map(m) => {
                    m.insert(msg, &bind[1..])
                }
                KeybindTarget::Msg(_m) => {
                    self.0.insert(
                        bind[0],
                        KeybindTarget::Map(KeybindMap(HashMap::new())),
                    );
                    self.insert(msg, bind)
                }
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
        if bind.is_empty() {
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

pub fn parse_keybind_single(s: &str) -> Option<KeyCode> {
    if s.len() == 1 {
        s.chars().next().map(KeyCode::Char)
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
            "<enter>" => Some(KeyCode::Enter),
            _ => None,
        }
    }
}
pub fn parse_keybind(s: String) -> Result<Vec<KeyEvent>> {
    let mut out: Vec<KeyEvent> = Vec::new();
    for word in s.split(' ') {
        if let Some(suffix) = word.strip_prefix("C-") {
            out.push(KeyEvent::new(
                parse_keybind_single(suffix)
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::CONTROL,
            ))
        } else if let Some(suffix) = word.strip_prefix("M-") {
            out.push(KeyEvent::new(
                parse_keybind_single(suffix)
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::META,
            ))
        } else if let Some(suffix) = word.strip_prefix("S-") {
            out.push(KeyEvent::new(
                parse_keybind_single(suffix)
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::SUPER,
            ))
        } else if let Some(suffix) = word.strip_prefix("C-M-") {
            out.push(KeyEvent::new(
                parse_keybind_single(suffix)
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::CONTROL | KeyModifiers::META,
            ))
        } else {
            out.push(KeyEvent::new(
                parse_keybind_single(word)
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::empty(),
            ))
        }
    }
    Ok(out)
}
