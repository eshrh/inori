use ratatui::crossterm;
use std::time::{Duration, Instant};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub enum Event {
    Tick,
    Key(crossterm::event::KeyEvent),
}

pub struct EventHandler {
    rx: std::sync::mpsc::Receiver<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        const POLL_TIME: Duration = Duration::from_millis(16);
        const TICK_INTERVAL: Duration = Duration::from_millis(500);

        let (tx, rx) = std::sync::mpsc::channel();
        let mut now = Instant::now();
        let mut last_event = Instant::now();
        std::thread::spawn(move || loop {
            if let Ok(true) = crossterm::event::poll(poll_time) {
                let send_res = match crossterm::event::read() {
                    Ok(crossterm::event::Event::Key(e)) => {
                        last_event = Instant::now();
                        tx.send(Event::Key(e))
                    }
                    Ok(crossterm::event::Event::Resize(_, _)) => Ok(()),
                    Ok(_) => Ok(()),
                    Err(_) => Ok(()),
                };
                if send_res.is_err() {
                    break;
                }
            }
            // only tick when idle.
            let time_since_last_event: Duration = Instant::now() - last_event;
            if now.elapsed() >= TICK_INTERVAL && time_since_last_event >= TICK_INTERVAL
            {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                now = Instant::now();
            }
        });
        EventHandler { rx }
    }

    pub fn next(&self) -> Result<Event> {
        Ok(self.rx.recv()?)
    }
}
