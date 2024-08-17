use crate::event_handler::Result;
use crate::model::{AlbumData, ArtistData, Model, Screen, State};
use crate::util::{safe_decrement, safe_increment};
use ratatui::crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use std::option::Option;
mod build_library;
mod handlers;
mod updaters;

#[derive(PartialEq, Eq)]
pub enum SwitchTo {
    Library,
    Queue,
    Playlist,
}

#[derive(PartialEq, Eq)]
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
    Search(SearchMsg),
    Escape,
    Set(Toggle),
}

pub fn update_tick(model: &mut Model) -> Result<()> {
    model.status = model.conn.status()?;
    model.currentsong = model.conn.currentsong()?;
    Ok(())
}
pub fn update_screens(model: &mut Model) -> Result<()> {
    match model.screen {
        Screen::Library => updaters::update_library(model)?,
        Screen::Queue => updaters::update_queue(model)?,
        Screen::Playlist => updaters::update_playlist(model)?,
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
        KeyCode::Char('/') => Some(Message::Search(SearchMsg::Start)),
        KeyCode::Char('1') => Some(Message::SwitchScreen(SwitchTo::Library)),
        KeyCode::Char('2') => Some(Message::SwitchScreen(SwitchTo::Queue)),
        KeyCode::Char('3') => Some(Message::SwitchScreen(SwitchTo::Playlist)),
        KeyCode::Tab => Some(Message::Tab),
        KeyCode::Enter => Some(Message::Enter),
        KeyCode::Backspace => Some(Message::Delete),
        KeyCode::Esc => Some(Message::Escape),
        _ => None,
    }
}

pub fn handle_key(model: &mut Model, k: KeyEvent) -> Result<()> {
    match model.state {
        State::Searching => match model.screen {
            Screen::Library => {
                handlers::library_handler::handle_search(model, k)?
            }
            Screen::Queue => unimplemented!(),
            Screen::Playlist => unimplemented!(),
        },
        State::Running => {
            if let Some(m) = parse_msg(k) {
                handle_msg(model, m)?;
            }
        }
        State::Done => {}
    }
    Ok(())
}

pub fn handle_msg(model: &mut Model, m: Message) -> Result<()> {
    match m {
        Message::SwitchState(state) => model.state = state,
        Message::SwitchScreen(SwitchTo::Library) => {
            model.screen = Screen::Library
        }
        Message::SwitchScreen(SwitchTo::Queue) => model.screen = Screen::Queue,
        Message::SwitchScreen(SwitchTo::Playlist) => {
            model.screen = Screen::Playlist
        }
        Message::PlayPause => model.conn.toggle_pause()?,
        Message::Set(t) => {
            match t {
                Toggle::Repeat => model.conn.repeat(!model.status.repeat),
                Toggle::Single => model.conn.single(!model.status.single),
                Toggle::Random => model.conn.random(!model.status.random),
                Toggle::Consume => model.conn.consume(!model.status.consume),
            }?;
            model.update_status()?;
        }
        Message::Clear => model.conn.clear()?,
        other => match model.screen {
            Screen::Library => {
                handlers::library_handler::handle_library(model, other)?
            }
            Screen::Queue => {
                handlers::queue_handler::handle_queue(model, other)?
            }
            Screen::Playlist => handlers::handle_playlist(model, other)?,
        },
    }
    Ok(())
}
