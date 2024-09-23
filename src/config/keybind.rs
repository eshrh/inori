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
const EMPTY: KeyModifiers = KeyModifiers::empty();

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
        "next_song" => Some(Message::NextSong),
        "previous_song" => Some(Message::PreviousSong),
        "seek" => Some(Message::Seek(SeekDirection::Forward)),
        "seek_backwards" => Some(Message::Seek(SeekDirection::Backward)),
        _ => None,
    }
}

impl KeybindMap {
    pub fn default() -> Self {
        let mut keybindings = HashMap::new();
        keybindings.insert(
            KeyEvent::new(KeyCode::Up, EMPTY),
            Msg(Direction(Dirs::Vert(Vertical::Up))),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Down, EMPTY),
            Msg(Direction(Dirs::Vert(Vertical::Down))),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Left, EMPTY),
            Msg(Direction(Dirs::Horiz(Horizontal::Left))),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Right, EMPTY),
            Msg(Direction(Dirs::Horiz(Horizontal::Right))),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Home, EMPTY),
            Msg(Direction(Dirs::Vert(Vertical::Top))),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::End, EMPTY),
            Msg(Direction(Dirs::Vert(Vertical::Bottom))),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            Msg(GlobalSearch(SearchMsg::Start)),
        );

        keybindings
            .insert(KeyEvent::new(KeyCode::Char('p'), EMPTY), Msg(PlayPause));
        keybindings.insert(KeyEvent::new(KeyCode::Enter, EMPTY), Msg(Select));
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('1'), EMPTY),
            Msg(SwitchScreen(super::Screen::Library)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('2'), EMPTY),
            Msg(SwitchScreen(super::Screen::Queue)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('q'), EMPTY),
            Msg(SwitchState(super::State::Done)),
        );
        keybindings
            .insert(KeyEvent::new(KeyCode::Backspace, EMPTY), Msg(Delete));
        keybindings
            .insert(KeyEvent::new(KeyCode::Tab, EMPTY), Msg(ToggleScreen));
        keybindings.insert(KeyEvent::new(KeyCode::Char(' '), EMPTY), Msg(Fold));
        keybindings
            .insert(KeyEvent::new(KeyCode::Char('-'), EMPTY), Msg(Clear));
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('/'), EMPTY),
            Msg(LocalSearch(SearchMsg::Start)),
        );
        keybindings.insert(KeyEvent::new(KeyCode::Esc, EMPTY), Msg(Escape));
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('r'), EMPTY),
            Msg(Set(Toggle::Repeat)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('z'), EMPTY),
            Msg(Set(Toggle::Random)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('s'), EMPTY),
            Msg(Set(Toggle::Single)),
        );
        keybindings.insert(
            KeyEvent::new(KeyCode::Char('c'), EMPTY),
            Msg(Set(Toggle::Consume)),
        );
        Self(keybindings)
    }
    pub fn with_dvorak_style(mut self) -> Self {
        self.0.insert(
            KeyEvent::new(KeyCode::Char('t'), EMPTY),
            Msg(Direction(Dirs::Vert(Vertical::Up))),
        );
        self.0.insert(
            KeyEvent::new(KeyCode::Char('h'), EMPTY),
            Msg(Direction(Dirs::Vert(Vertical::Down))),
        );
        self.0.insert(
            KeyEvent::new(KeyCode::Char('d'), EMPTY),
            Msg(Direction(Dirs::Horiz(Horizontal::Left))),
        );
        self.0.insert(
            KeyEvent::new(KeyCode::Char('n'), EMPTY),
            Msg(Direction(Dirs::Horiz(Horizontal::Right))),
        );
        self.0.insert(
            KeyEvent::new(KeyCode::Char('<'), EMPTY),
            Msg(Direction(Dirs::Vert(Vertical::Top))),
        );
        self.0.insert(
            KeyEvent::new(KeyCode::Char('>'), EMPTY),
            Msg(Direction(Dirs::Vert(Vertical::Bottom))),
        );
        self.0.insert(
            KeyEvent::new(KeyCode::Char('g'), EMPTY),
            Msg(GlobalSearch(SearchMsg::Start)),
        );
        self.0.insert(
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL),
            Msg(Escape),
        );
        self
    }
    pub fn with_qwerty_style(mut self) -> Self {
        self.0.insert(
            KeyEvent::new(KeyCode::Char('k'), EMPTY),
            Msg(Direction(Dirs::Vert(Vertical::Up))),
        );
        self.0.insert(
            KeyEvent::new(KeyCode::Char('j'), EMPTY),
            Msg(Direction(Dirs::Vert(Vertical::Down))),
        );
        self.0.insert(
            KeyEvent::new(KeyCode::Char('h'), EMPTY),
            Msg(Direction(Dirs::Horiz(Horizontal::Left))),
        );
        self.0.insert(
            KeyEvent::new(KeyCode::Char('l'), EMPTY),
            Msg(Direction(Dirs::Horiz(Horizontal::Right))),
        );
        self.0.insert(
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL),
            Msg(GlobalSearch(SearchMsg::Start)),
        );
        self.0.insert(
            KeyEvent::new(KeyCode::Char('G'), KeyModifiers::empty()),
            Msg(Direction(Dirs::Vert(Vertical::Bottom))),
        );
        self.insert(
            Direction(Dirs::Vert(Vertical::Top)),
            &[
                KeyEvent::new(KeyCode::Char('g'), EMPTY),
                KeyEvent::new(KeyCode::Char('g'), EMPTY),
            ],
        );
        self
    }
    pub fn insert(&mut self, msg: Message, bind: &[KeyEvent]) {
        if bind.len() == 1 {
            self.0.insert(bind[0], KeybindTarget::Msg(msg));
            return;
        }
        if self.0.contains_key(&bind[0]) {
            match self.0.get_mut(&bind[0]).unwrap() {
                KeybindTarget::Map(m) => m.insert(msg, &bind[1..]),
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
            "<home>" => Some(KeyCode::Home),
            "<end>" => Some(KeyCode::End),
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
