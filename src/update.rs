use crate::backend::Command;
pub use crate::backend::Toggle;
use crate::config::keybind::{KeybindMap, KeybindTarget};
use crate::model::{Model, Screen, State};
use ratatui::crossterm::event::{self, KeyCode, KeyEvent};

pub mod build_library;
mod handlers;
mod updaters;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Vertical {
    Up,
    Down,
    Top,
    Bottom,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Horizontal {
    Left,
    Right,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Dirs {
    Vert(Vertical),
    Horiz(Horizontal),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum SearchMsg {
    Start,
    End,
}

#[derive(PartialEq, Clone, Debug)]
pub enum SeekDirection {
    Forward,
    Backward,
}

#[derive(Clone, Debug)]
pub enum Message {
    Direction(Dirs),
    ScrollScreenful(Vertical),
    PlayPause,
    NextSong,
    PreviousSong,
    Seek(SeekDirection),
    UpdateDB,
    Select,
    SelectAndNext,
    SwitchState(State),
    SwitchScreen(usize),
    ToggleScreen,
    Delete,
    TogglePanel,
    Fold,
    Clear,
    LocalSearch(SearchMsg),
    GlobalSearch(SearchMsg),
    Escape,
    Set(Toggle),
}

pub struct Action {
    pub command: Command,
    pub on_accept: Option<Message>,
}

impl Action {
    fn new(command: Command) -> Self {
        Self {
            command,
            on_accept: None,
        }
    }

    fn then(mut self, message: Message) -> Self {
        self.on_accept = Some(message);
        self
    }
}

pub fn update_screens(model: &mut Model) {
    match model.screen {
        Screen::Library => updaters::update_library(model),
        Screen::Queue => updaters::update_queue(model),
    }
}

fn parse_msg(
    key: event::KeyEvent,
    state: &mut Vec<KeyEvent>,
    keybinds: &KeybindMap,
) -> Option<Message> {
    state.push(key);
    match keybinds.lookup(state) {
        Some(KeybindTarget::Msg(m)) => {
            state.clear();
            Some(m.clone())
        }
        Some(KeybindTarget::Map(_)) => None,
        None => {
            state.clear();
            None
        }
    }
}

pub fn handle_key(model: &mut Model, key: KeyEvent) -> Option<Action> {
    match model.state {
        State::Searching => match model.screen {
            Screen::Library => {
                handlers::library_handler::handle_search(model, key)
            }
            Screen::Queue => handlers::queue_handler::handle_search(model, key),
        },
        State::Running => {
            let message = parse_msg(
                key,
                &mut model.parse_state,
                &model.config.keybindings,
            )?;
            handle_msg(model, message)
        }
        State::Done => None,
    }
}

pub fn handle_msg(model: &mut Model, message: Message) -> Option<Action> {
    if matches!(
        message,
        Message::Direction(_)
            | Message::ScrollScreenful(_)
            | Message::SwitchScreen(_)
            | Message::ToggleScreen
            | Message::Escape
    ) {
        model.library.pending_jump = None;
    }
    match message {
        Message::SwitchState(state) => {
            model.state = state;
            None
        }
        Message::SwitchScreen(to) => {
            if let Some(screen) =
                to.checked_sub(1).and_then(|i| model.config.screens.get(i))
            {
                model.screen = screen.clone();
                model.state = State::Running;
                model.library.global_search.entries.filter.set_off();
            }
            None
        }
        Message::UpdateDB => Some(Action::new(Command::UpdateDatabase)),
        Message::PlayPause => Some(Action::new(Command::TogglePause)),
        Message::NextSong => Some(Action::new(Command::Next)),
        Message::PreviousSong => Some(Action::new(Command::Previous)),
        Message::Seek(direction) => Some(Action::new(Command::Seek {
            seconds: match direction {
                SeekDirection::Forward => model.config.seek_seconds,
                SeekDirection::Backward => -model.config.seek_seconds,
            },
        })),
        Message::Set(toggle) => Some(Action::new(Command::Toggle(toggle))),
        Message::Clear => Some(Action::new(Command::Clear)),
        other => match model.screen {
            Screen::Library => {
                handlers::library_handler::handle_library(model, other)
            }
            Screen::Queue => {
                handlers::queue_handler::handle_queue(model, other)
            }
        },
    }
}
