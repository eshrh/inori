use crate::config::keybind::{KeybindMap, KeybindTarget};
use crate::event_handler::Result;
use crate::model::proto::Searchable;
use crate::model::{Model, Screen, State};
use crate::util::{safe_decrement, safe_increment};
use bitflags::bitflags;
use mpd::status::State as PlayState;
use ratatui::crossterm::event::{self, KeyCode, KeyEvent};
use std::option::Option;
use std::time::Duration;

pub mod build_library;
mod handlers;
mod updaters;

bitflags! {
    pub struct Update: u8 {
        const QUEUE = 0b00000001;
        const CURRENT_ARTIST = 0b00000010;
        const STATUS = 0b00000100;
        const CURRENT_SONG = 0b00001000;
        const START_PLAYING = 0b00010000;
    }
}

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

#[derive(Clone, Debug)]
pub enum Toggle {
    Repeat,
    Random,
    Single,
    Consume,
}

#[derive(PartialEq, Clone, Debug)]
pub enum SeekDirection {
    Forward,
    Backward,
}

#[derive(Clone, Debug)]
pub enum Message {
    Direction(Dirs),
    PlayPause,
    NextSong,
    PreviousSong,
    Seek(SeekDirection),
    Select,
    SwitchState(State),
    SwitchScreen(Screen),
    Delete,
    ToggleScreen,
    TogglePanel,
    Fold,
    Clear,
    LocalSearch(SearchMsg),
    GlobalSearch(SearchMsg),
    Escape,
    Set(Toggle),
}

pub fn update_tick(model: &mut Model) -> Result<()> {
    update_screens(model, Update::all())?;
    Ok(())
}

pub fn update_screens(model: &mut Model, update: Update) -> Result<()> {
    if update.contains(Update::QUEUE) {
        model.queue.contents = model.conn.queue().unwrap_or_default();
    }
    if update.contains(Update::CURRENT_ARTIST)
        && model.library.selected_item_mut().is_some()
    {
        build_library::add_tracks(model)?;
    }
    if update.contains(Update::START_PLAYING) {
        if !update.contains(Update::QUEUE) {
            model.queue.contents = model.conn.queue().unwrap_or_default();
        }
        model.update_status()?;
        if model.status.queue_len > 0 && model.status.state == mpd::State::Stop
        {
            model.conn.switch(0)?;
        }
    }
    if update.contains(Update::CURRENT_SONG) {
        model.update_currentsong()?;
    }
    if update.contains(Update::STATUS) {
        model.update_status()?;
    }
    match model.screen {
        Screen::Library => updaters::update_library(model)?,
        Screen::Queue => updaters::update_queue(model)?,
    }
    Ok(())
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

pub fn handle_key(model: &mut Model, k: KeyEvent) -> Result<Update> {
    match model.state {
        State::Searching => match model.screen {
            Screen::Library => {
                Ok(handlers::library_handler::handle_search(model, k)?)
            }
            Screen::Queue => {
                Ok(handlers::queue_handler::handle_search(model, k)?)
            }
        },
        State::Running => {
            if let Some(m) =
                parse_msg(k, &mut model.parse_state, &model.config.keybindings)
            {
                Ok(handle_msg(model, m)?)
            } else {
                Ok(Update::empty())
            }
        }
        State::Done => Ok(Update::empty()),
    }
}

pub fn handle_msg(model: &mut Model, m: Message) -> Result<Update> {
    match m {
        Message::SwitchState(state) => {
            model.state = state;
            Ok(Update::empty())
        }
        Message::SwitchScreen(to) => {
            model.screen = match to {
                Screen::Library => Screen::Library,
                Screen::Queue => Screen::Queue,
            };
            Ok(Update::empty())
        }
        Message::PlayPause => {
            model.conn.toggle_pause()?;
            Ok(Update::STATUS)
        }
        Message::NextSong => match model.status.state {
            PlayState::Stop => Ok(Update::empty()),
            _ => {
                model.conn.next()?;
                Ok(Update::CURRENT_SONG | Update::STATUS)
            }
        },
        Message::PreviousSong => match model.status.state {
            PlayState::Stop => Ok(Update::empty()),
            _ => {
                model.conn.prev()?;
                Ok(Update::CURRENT_SONG | Update::STATUS)
            }
        },
        Message::Seek(direction) => {
            let mut update_flags = Update::empty();

            if let (Some((current_pos, total)), Some(queue_pos)) =
                (model.status.time, model.status.song)
            {
                let delta = Duration::from_secs(
                    model.config.seek_seconds.unsigned_abs(),
                );

                let new_pos = if direction == SeekDirection::Backward {
                    (current_pos.checked_sub(delta))
                        .unwrap_or(Duration::default())
                } else {
                    (current_pos.checked_add(delta)).unwrap_or(total).min(total)
                };

                if new_pos >= total {
                    model.conn.next()?;
                    update_flags |= Update::CURRENT_SONG | Update::STATUS;
                } else {
                    model.conn.seek(queue_pos.pos, new_pos)?;
                    update_flags |= Update::STATUS;
                }
            }
            Ok(update_flags)
        }
        Message::Set(t) => {
            match t {
                Toggle::Repeat => model.conn.repeat(!model.status.repeat),
                Toggle::Single => model.conn.single(!model.status.single),
                Toggle::Random => model.conn.random(!model.status.random),
                Toggle::Consume => model.conn.consume(!model.status.consume),
            }?;
            Ok(Update::STATUS)
        }
        Message::Clear => {
            model.conn.clear()?;
            Ok(Update::STATUS | Update::QUEUE | Update::CURRENT_SONG)
        }
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
