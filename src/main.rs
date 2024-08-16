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
    let event_handler = event_handler::EventHandler::new();

    update::update_screens(&mut model)?;
    terminal.draw(|f| view::view(&mut model, f))?;

    loop {
        match event_handler.next()? {
            Event::Tick => update::update_tick(&mut model)?,
            Event::Key(k) => {
                update::handle_key(&mut model, k)?;
                update::update_screens(&mut model)?;
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
