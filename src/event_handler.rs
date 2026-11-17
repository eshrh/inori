use ratatui::crossterm::event;
use std::io;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::sync::mpsc;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct EventHandler {
    rx: mpsc::Receiver<io::Result<event::Event>>,
    stopped: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(128);
        let stopped = Arc::new(AtomicBool::new(false));
        let worker_stopped = stopped.clone();
        let thread = thread::spawn(move || {
            while !worker_stopped.load(Ordering::Relaxed) {
                let result = match event::poll(Duration::from_millis(16)) {
                    Ok(false) => continue,
                    Ok(true) => event::read(),
                    Err(error) => Err(error),
                };
                let failed = result.is_err();
                if tx.blocking_send(result).is_err() || failed {
                    break;
                }
            }
        });
        Self {
            rx,
            stopped,
            thread: Some(thread),
        }
    }

    pub async fn next(&mut self) -> Result<event::Event> {
        self.rx
            .recv()
            .await
            .ok_or("terminal event reader stopped")?
            .map_err(Into::into)
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        self.rx.close();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
