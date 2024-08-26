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
        let poll_time = Duration::from_millis(16);
        let tick_interval = Duration::from_millis(500);

        let (tx, rx) = std::sync::mpsc::channel();
        let mut now = Instant::now();
        let mut last_event = Instant::now();
        std::thread::spawn(move || loop {
            if crossterm::event::poll(poll_time).expect("event poll failed") {
                match crossterm::event::read().expect("event read failed") {
                    crossterm::event::Event::Key(e) => {
                        last_event = Instant::now();
                        tx.send(Event::Key(e))
                    }
                    crossterm::event::Event::Resize(_, _) => Ok(()),
                    _ => unimplemented!(),
                }
                .expect("event send failed")
            }
            // only tick when idle.
            if now.elapsed() >= tick_interval
                && (Instant::now() - last_event >= Duration::from_millis(500))
            {
                tx.send(Event::Tick).expect("tick send failed");
                now = Instant::now();
            }
        });
        EventHandler { rx }
    }

    pub fn next(&self) -> Result<Event> {
        Ok(self.rx.recv()?)
    }
}
