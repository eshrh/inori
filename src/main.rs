extern crate mpd;
use model::State;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
            LeaveAlternateScreen,
        },
        ExecutableCommand,
    },
    Terminal,
};
use std::io::stdout;
use update::Update;
mod config;
mod event_handler;
mod model;
mod update;
mod util;
mod view;
use event_handler::{Event, Result};

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut model = model::Model::new().expect("Failed to init.");

    update::update_tick(&mut model)?;
    update::update_screens(&mut model, Update::empty())?;
    terminal.draw(|f| view::view(&mut model, f))?;

    let event_handler = event_handler::EventHandler::new();
    loop {
        match event_handler.next()? {
            Event::Tick => update::update_tick(&mut model)?,
            Event::Key(k) => {
                let update = update::handle_key(&mut model, k)?;
                update::update_screens(&mut model, update)?;
            }
        }
        terminal.draw(|f| view::view(&mut model, f))?;
        if let State::Done = model.state {
            break;
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
