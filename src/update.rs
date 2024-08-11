use crate::event_handler::Result;
use crate::model::{AlbumData, ArtistData, Model, Screen, State};
use crate::util::{safe_decrement, safe_increment};
use ratatui::crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use std::option::Option;

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
pub enum Message {
    Direction(Dirs),
    PlayPause,
    Enter,
    Quit,
    Switch(SwitchTo),
    Delete,
}

mod build_library;
mod handlers;
mod updaters;

pub fn update_tick(model: &mut Model) -> Result<()> {
    model.status = model.conn.status()?;
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
            Some(Message::Direction(Dirs::Horiz(Horizontal::Left)))
        }
        KeyCode::Char('d') => {
            Some(Message::Direction(Dirs::Horiz(Horizontal::Right)))
        }
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('p') => Some(Message::PlayPause),
        KeyCode::Char('1') => Some(Message::Switch(SwitchTo::Library)),
        KeyCode::Char('2') => Some(Message::Switch(SwitchTo::Queue)),
        KeyCode::Char('3') => Some(Message::Switch(SwitchTo::Playlist)),
        KeyCode::Enter => Some(Message::Enter),
        KeyCode::Backspace => Some(Message::Delete),
        _ => None,
    }
}

pub fn handle_event(model: &mut Model, k: KeyEvent) -> Result<()> {
    let msg = parse_msg(k);
    match msg {
        Some(Message::Quit) => model.state = State::Done,
        Some(Message::Switch(SwitchTo::Library)) => {
            model.screen = Screen::Library
        }
        Some(Message::Switch(SwitchTo::Queue)) => model.screen = Screen::Queue,
        Some(Message::Switch(SwitchTo::Playlist)) => {
            model.screen = Screen::Playlist
        }
        Some(Message::PlayPause) => model.conn.toggle_pause()?,
        Some(other) => match model.screen {
            Screen::Library => handlers::handle_library(model, other)?,
            Screen::Queue => handlers::handle_queue(model, other)?,
            Screen::Playlist => handlers::handle_playlist(model, other)?,
        },
        None => {}
    }
    Ok(())
}
