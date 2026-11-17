extern crate mpd;
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
mod app;
mod backend;
mod config;
mod event_handler;
mod model;
mod update;
mod util;
mod view;
use event_handler::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
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

    let model = model::Model::new(terminal.get_frame().area())?;
    app::App::new(model)?.run(&mut terminal).await
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
