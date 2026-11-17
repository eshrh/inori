use bitflags::bitflags;
use mpd::{Id, Song, Status, Subsystem};
use std::fmt;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::sleep;

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
    Connected {
        session: Session,
        snapshot: Box<Snapshot>,
    },
    Disconnected(String),
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
            Self::Closed => write!(f, "MPD is disconnected"),
        }
    }
}

impl std::error::Error for SubmitError {}

enum Request {
    Command(Command),
    Refresh(Refresh),
}

pub struct Session {
    requests: mpsc::Sender<Request>,
    reads: mpsc::Sender<ReadRequest>,
}

impl Session {
    pub fn try_command(&self, command: Command) -> Result<(), SubmitError> {
        submit(&self.requests, Request::Command(command))
    }

    pub fn try_refresh(&self, refresh: Refresh) -> Result<(), SubmitError> {
        submit(&self.requests, Request::Refresh(refresh))
    }

    pub fn try_read(&self, request: ReadRequest) -> Result<(), SubmitError> {
        submit(&self.reads, request)
    }
}

pub struct Backend(JoinHandle<()>);

impl Backend {
    pub fn start(address: Option<String>) -> (Self, mpsc::Receiver<Event>) {
        let (events, receiver) = mpsc::channel(64);
        let task = tokio::spawn(async move {
            let (mut client, address) =
                match connection::select(address.as_deref()).await {
                    Ok(connection) => connection,
                    Err(error) => {
                        let _ = events.send(Event::Error(error)).await;
                        return;
                    }
                };
            let mut idle = match address.connect().await {
                Ok(client) => client,
                Err(error) => {
                    let _ = events
                        .send(Event::Error(format!("{address}: {error}")))
                        .await;
                    return;
                }
            };
            loop {
                let result = worker::run(client, idle, &address, &events).await;
                if events.is_closed() {
                    return;
                }
                let error = result
                    .err()
                    .unwrap_or_else(|| "MPD session stopped".into());
                let reconnect =
                    error.downcast_ref::<mpd::error::Error>().is_some_and(
                        |error| matches!(error, mpd::error::Error::Io(_)),
                    ) || error.is::<std::io::Error>();
                if !reconnect {
                    let _ = events
                        .send(Event::Error(format!(
                            "MPD at {address}: {error}"
                        )))
                        .await;
                    return;
                }
                if events
                    .send(Event::Disconnected(format!(
                        "MPD at {address}: {error}"
                    )))
                    .await
                    .is_err()
                {
                    return;
                }
                loop {
                    sleep(Duration::from_secs(1)).await;
                    if events.is_closed() {
                        return;
                    }
                    if let Ok(next) = address.connect().await {
                        if let Ok(next_idle) = address.connect().await {
                            client = next;
                            idle = next_idle;
                            break;
                        }
                    }
                }
            }
        });
        (Self(task), receiver)
    }

    pub fn shutdown(&self) {
        self.0.abort();
    }
}

impl Drop for Backend {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn submit<T>(sender: &mpsc::Sender<T>, request: T) -> Result<(), SubmitError> {
    sender.try_send(request).map_err(|error| match error {
        mpsc::error::TrySendError::Full(_) => SubmitError::Full,
        mpsc::error::TrySendError::Closed(_) => SubmitError::Closed,
    })
}
