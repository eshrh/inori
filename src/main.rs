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
use std::panic;
use update::Update;
mod config;
mod event_handler;
mod model;
mod update;
mod util;
mod view;
use event_handler::{Event, Result};

fn main() -> Result<()> {
    let hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        let _ = reset_terminal();
        hook(panic);
    }));

    let _terminal_guard = TerminalGuard;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut model = model::Model::new(terminal.get_frame().area())?;
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
        model.frame_size = terminal.get_frame().area();
        terminal.draw(|f| view::view(&mut model, f))?;
        if let State::Done = model.state {
            break;
        }
    }
    Ok(())
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = reset_terminal();
    }
}

fn reset_terminal() -> Result<()> {
    let screen_result = stdout().execute(LeaveAlternateScreen).map(|_| ());
    let raw_mode_result = disable_raw_mode();
    screen_result?;
    raw_mode_result?;
    Ok(())
}
