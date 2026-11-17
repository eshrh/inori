use bitflags::bitflags;
use mpd::{Id, Song, Status, Subsystem};
use std::fmt;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tokio::sync::mpsc as events;

mod connection;
mod worker;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Refresh: u8 {
        const STATUS = 0b001;
        const QUEUE = 0b010;
        const CURRENT_SONG = 0b100;
    }
}

#[derive(Clone, Debug)]
pub enum Toggle {
    Repeat,
    Random,
    Single,
    Consume,
}

pub enum QueueTarget {
    Artist(String),
    Album { artist: String, album: String },
    Song(String),
}

pub enum Command {
    TogglePause,
    Next,
    Previous,
    Seek { seconds: i64 },
    UpdateDatabase,
    Toggle(Toggle),
    Clear,
    Play(Id),
    Swap(Id, Id),
    Delete(Id),
    Enqueue(QueueTarget),
}

pub enum ReadRequest {
    Artists { generation: u64 },
    Tracks { generation: u64, artist: String },
    GlobalSearch { generation: u64 },
}

#[derive(Default)]
pub struct Snapshot {
    pub status: Option<Status>,
    pub queue: Option<Vec<Song>>,
    pub current_song: Option<Option<Song>>,
}

pub enum Event {
    Completed {
        snapshot: Box<Snapshot>,
        error: Option<String>,
    },
    Refreshed(Box<Snapshot>),
    Artists {
        generation: u64,
        artists: Vec<(String, String)>,
    },
    Tracks {
        generation: u64,
        artist: String,
        songs: Vec<Song>,
    },
    GlobalSearch {
        generation: u64,
        songs: Vec<Song>,
    },
    Idle(Vec<Subsystem>),
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmitError {
    Full,
    Closed,
}

impl fmt::Display for SubmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full => write!(f, "MPD request queue is full"),
            Self::Closed => write!(f, "MPD worker has stopped"),
        }
    }
}

impl std::error::Error for SubmitError {}

enum Request {
    Command(Command),
    Refresh(Refresh),
}

#[derive(Clone)]
struct EventSender {
    sender: events::Sender<Event>,
    stopped: Arc<AtomicBool>,
}

impl EventSender {
    fn send(&self, mut event: Event) -> bool {
        while !self.stopped.load(Ordering::Acquire) {
            match self.sender.try_send(event) {
                Ok(()) => return true,
                Err(events::error::TrySendError::Closed(_)) => return false,
                Err(events::error::TrySendError::Full(pending)) => {
                    event = pending;
                    thread::sleep(Duration::from_millis(5));
                }
            }
        }
        false
    }
}

pub struct Backend {
    commands: Option<SyncSender<Request>>,
    reads: Option<SyncSender<ReadRequest>>,
    idle: Option<SyncSender<()>>,
    stopped: Arc<AtomicBool>,
    endpoint: Arc<connection::SharedEndpoint>,
    sockets: Vec<Arc<connection::SocketControl>>,
    workers: Vec<JoinHandle<()>>,
}

impl Backend {
    pub fn start(
        address: Option<String>,
    ) -> io::Result<(Self, events::Receiver<Event>)> {
        let (event_sender, receiver) = events::channel(64);
        let stopped = Arc::new(AtomicBool::new(false));
        let sender = EventSender {
            sender: event_sender,
            stopped: stopped.clone(),
        };
        let (commands, command_receiver) = mpsc::sync_channel(32);
        let (reads, read_receiver) = mpsc::sync_channel(8);
        let (idle, idle_receiver) = mpsc::sync_channel(1);
        let endpoint =
            Arc::new(connection::SharedEndpoint::new(stopped.clone()));
        let mut backend = Self {
            commands: Some(commands),
            reads: Some(reads),
            idle: Some(idle),
            stopped,
            endpoint,
            sockets: Vec::new(),
            workers: Vec::new(),
        };

        let socket = backend.socket();
        let endpoint = backend.endpoint.clone();
        let worker_sender = sender.clone();
        backend.spawn("command", move || {
            worker::commands(
                address,
                endpoint,
                socket,
                command_receiver,
                worker_sender,
            );
        })?;
        let socket = backend.socket();
        let endpoint = backend.endpoint.clone();
        let worker_sender = sender.clone();
        backend.spawn("library", move || {
            worker::reads(endpoint, socket, read_receiver, worker_sender);
        })?;
        let socket = backend.socket();
        let endpoint = backend.endpoint.clone();
        backend.spawn("idle", move || {
            worker::idle(endpoint, socket, idle_receiver, sender);
        })?;
        Ok((backend, receiver))
    }

    pub fn try_command(&self, command: Command) -> Result<(), SubmitError> {
        submit(&self.commands, Request::Command(command))
    }

    pub fn try_refresh(&self, refresh: Refresh) -> Result<(), SubmitError> {
        submit(&self.commands, Request::Refresh(refresh))
    }

    pub fn try_read(&self, request: ReadRequest) -> Result<(), SubmitError> {
        submit(&self.reads, request)
    }

    pub fn try_idle_poll(&self) -> Result<(), SubmitError> {
        submit(&self.idle, ())
    }

    pub fn shutdown(&mut self) {
        self.stopped.store(true, Ordering::Release);
        self.endpoint.wake();
        self.commands.take();
        self.reads.take();
        self.idle.take();
        for socket in &self.sockets {
            socket.close();
        }
        let deadline = Instant::now() + Duration::from_millis(250);
        while self.workers.iter().any(|worker| !worker.is_finished())
            && Instant::now() < deadline
        {
            thread::sleep(Duration::from_millis(5));
        }
        for worker in self.workers.drain(..) {
            if worker.is_finished() {
                let _ = worker.join();
            }
        }
    }

    fn socket(&mut self) -> Arc<connection::SocketControl> {
        let socket =
            Arc::new(connection::SocketControl::new(self.stopped.clone()));
        self.sockets.push(socket.clone());
        socket
    }

    fn spawn(
        &mut self,
        name: &str,
        worker: impl FnOnce() + Send + 'static,
    ) -> io::Result<()> {
        let worker = thread::Builder::new()
            .name(format!("inori-mpd-{name}"))
            .spawn(worker)?;
        self.workers.push(worker);
        Ok(())
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn submit<T>(
    sender: &Option<SyncSender<T>>,
    request: T,
) -> Result<(), SubmitError> {
    sender
        .as_ref()
        .ok_or(SubmitError::Closed)?
        .try_send(request)
        .map_err(|error| match error {
            TrySendError::Full(_) => SubmitError::Full,
            TrySendError::Disconnected(_) => SubmitError::Closed,
        })
}
