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
mod matching;
mod model;
mod update;
mod util;
mod view;
use event_handler::Result;

fn main() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    let result = runtime.block_on(run());
    runtime.shutdown_timeout(std::time::Duration::from_millis(250));
    result
}

async fn run() -> Result<()> {
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
