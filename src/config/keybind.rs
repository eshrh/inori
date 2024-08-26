use crate::event_handler::Result;
use crate::update::Message;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

pub enum KeybindTarget {
    Map(KeybindMap),
    Msg(Message),
}
pub struct KeybindMap(pub HashMap<KeyEvent, KeybindTarget>);
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
                parse_keybind_single(&word[2..])
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::CONTROL,
            ))
        } else if word.starts_with("M-") {
            out.push(KeyEvent::new(
                parse_keybind_single(&word[2..])
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::META,
            ))
        } else if word.starts_with("S-") {
            out.push(KeyEvent::new(
                parse_keybind_single(&word[2..])
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::SUPER,
            ))
        } else if word.starts_with("C-M-") {
            out.push(KeyEvent::new(
                parse_keybind_single(&word[4..])
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::CONTROL | KeyModifiers::META,
            ))
        } else {
            out.push(KeyEvent::new(
                parse_keybind_single(&word)
                    .unwrap_or_else(|| panic!("couldn't parse {}", word)),
                KeyModifiers::empty(),
            ))
        }
    }
    Ok(out)
}
