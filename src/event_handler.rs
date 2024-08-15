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
        // check that poll time > tick_interval is not causing issues.
        // maybe one should tick every loop anyway... timer not necessary.
        let poll_time = Duration::from_millis(16);
        let tick_interval = Duration::from_millis(250);
        let (tx, rx) = std::sync::mpsc::channel();
        let mut now = Instant::now();
        std::thread::spawn(move || loop {
            if crossterm::event::poll(poll_time).expect("event poll failed") {
                match crossterm::event::read().expect("event read failed") {
                    crossterm::event::Event::Key(e) => tx.send(Event::Key(e)),
                    crossterm::event::Event::Resize(_, _) => Ok(()),
                    _ => unimplemented!(),
                }
                .expect("event send failed")
            }
            if now.elapsed() >= tick_interval {
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
