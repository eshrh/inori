use crate::event_handler::Result;
use crate::model::proto::Searchable;
use crate::model::{Model, Screen, State};
use crate::util::{safe_decrement, safe_increment};
use bitflags::bitflags;
use ratatui::crossterm::event::{self, KeyCode, KeyEvent};
use std::option::Option;

pub mod build_library;
mod handlers;
mod updaters;

bitflags! {
    pub struct Update: u8 {
        const QUEUE = 0x01;
        const CURRENT_ARTIST = 0x02;
        const STATUS = 0x04;
        const CURRENT_SONG = 0x08;
        const START_PLAYING = 0x10;
    }
}

#[derive(PartialEq, Eq)]
pub enum SwitchTo {
    Library,
    Queue,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Vertical {
    Up,
    Down,
}

#[derive(PartialEq, Eq)]
pub enum Horizontal {
    Left,
    Right,
}

#[derive(PartialEq, Eq)]
pub enum Dirs {
    Vert(Vertical),
    Horiz(Horizontal),
}

#[derive(PartialEq, Eq)]
pub enum SearchMsg {
    Start,
    End,
}

pub enum Toggle {
    Repeat,
    Random,
    Single,
    Consume,
}

pub enum Message {
    Direction(Dirs),
    PlayPause,
    Enter,
    SwitchState(State),
    SwitchScreen(SwitchTo),
    Delete,
    Tab,
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
        model.queue.contents = model.conn.queue().unwrap_or(vec![]);
    }
    if update.contains(Update::CURRENT_ARTIST) {
        if !model.library.selected_item_mut().is_some_and(|i| i.fetched) {
            build_library::add_tracks(model)?;
        }
    }
    if update.contains(Update::STATUS) {
        model.update_status()?;
    }
    if update.contains(Update::CURRENT_SONG) {
        model.update_currentsong()?;
    }
    if update.contains(Update::START_PLAYING) {
        if !update.contains(Update::QUEUE) {
            model.update_status()?;
        }
        if model.status.queue_len > 0 && model.status.state == mpd::State::Stop
        {
            model.conn.switch(0)?;
        }
    }
    match model.screen {
        Screen::Library => updaters::update_library(model)?,
        Screen::Queue => updaters::update_queue(model)?,
    }
    Ok(())
}

fn parse_msg(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('t') => {
            Some(Message::Direction(Dirs::Vert(Vertical::Up)))
        }
        KeyCode::Char('h') => {
            Some(Message::Direction(Dirs::Vert(Vertical::Down)))
        }
        KeyCode::Char('n') => {
            Some(Message::Direction(Dirs::Horiz(Horizontal::Right)))
        }
        KeyCode::Char('d') => {
            Some(Message::Direction(Dirs::Horiz(Horizontal::Left)))
        }
        KeyCode::Char('q') => Some(Message::SwitchState(State::Done)),
        KeyCode::Char('p') => Some(Message::PlayPause),

        KeyCode::Char('r') => Some(Message::Set(Toggle::Repeat)),
        KeyCode::Char('z') => Some(Message::Set(Toggle::Random)),
        KeyCode::Char('s') => Some(Message::Set(Toggle::Single)),
        KeyCode::Char('c') => Some(Message::Set(Toggle::Consume)),

        KeyCode::Char(' ') => Some(Message::Fold),
        KeyCode::Char('-') => Some(Message::Clear),
        KeyCode::Char('/') => Some(Message::LocalSearch(SearchMsg::Start)),
        KeyCode::Char('g') => Some(Message::GlobalSearch(SearchMsg::Start)),
        KeyCode::Char('1') => Some(Message::SwitchScreen(SwitchTo::Library)),
        KeyCode::Char('2') => Some(Message::SwitchScreen(SwitchTo::Queue)),
        KeyCode::Tab => Some(Message::Tab),
        KeyCode::Enter => Some(Message::Enter),
        KeyCode::Backspace => Some(Message::Delete),
        KeyCode::Esc => Some(Message::Escape),
        _ => None,
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
            if let Some(m) = parse_msg(k) {
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
                SwitchTo::Library => Screen::Library,
                SwitchTo::Queue => Screen::Queue,
            };
            Ok(Update::empty())
        }
        Message::PlayPause => {
            model.conn.toggle_pause()?;
            Ok(Update::STATUS)
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
            Ok(Update::STATUS | Update::QUEUE)
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
