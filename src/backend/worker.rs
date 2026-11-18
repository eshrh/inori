use super::{
    Command, Event, QueueTarget, ReadRequest, Refresh, Request, Session,
    Snapshot, Toggle,
};
use mpd::connection::{Address, Client};
use mpd::{Query, Status, Subsystem, Term};
use std::borrow::Cow::Borrowed;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::timeout;

type Result<T> = std::result::Result<T, mpd::error::Error>;
type WorkerResult =
    std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub(super) async fn run(
    command: Client,
    idle: Client,
    address: &Address,
    events: &mpsc::Sender<Event>,
) -> WorkerResult {
    let (requests, request_receiver) = mpsc::channel(32);
    let (reads, read_receiver) = mpsc::channel(8);
    let mut tasks = JoinSet::new();
    tasks.spawn(commands(
        command,
        request_receiver,
        events.clone(),
        address.to_string(),
        Session { requests, reads },
    ));
    tasks.spawn(notifications(idle, events.clone()));
    tasks.spawn(library(address.clone(), read_receiver, events.clone()));
    let result = match tasks.join_next().await {
        Some(Ok(result)) => result,
        Some(Err(error)) => Err(error.into()),
        None => Ok(()),
    };
    tasks.shutdown().await;
    result
}

async fn commands(
    mut client: Client,
    mut requests: mpsc::Receiver<Request>,
    events: mpsc::Sender<Event>,
    address: String,
    session: Session,
) -> WorkerResult {
    let initial = timeout(
        Duration::from_secs(30),
        snapshot(&mut client, Refresh::all()),
    )
    .await
    .map_err(command_timeout)??;
    events
        .send(Event::Connected {
            session,
            snapshot: Box::new(initial),
        })
        .await?;
    while let Some(request) = requests.recv().await {
        let event = timeout(Duration::from_secs(30), async {
            match request {
                Request::Command(command) => {
                    complete(&mut client, command, &address).await
                }
                Request::Refresh(flags) => snapshot(&mut client, flags)
                    .await
                    .map(|snapshot| Event::Refreshed(Box::new(snapshot))),
            }
        })
        .await
        .map_err(command_timeout)??;
        events.send(event).await?;
    }
    Ok(())
}

fn command_timeout(_: tokio::time::error::Elapsed) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::TimedOut, "MPD command timed out")
}

async fn notifications(
    mut client: Client,
    events: mpsc::Sender<Event>,
) -> WorkerResult {
    let subsystems = [
        Subsystem::Database,
        Subsystem::Update,
        Subsystem::Player,
        Subsystem::Queue,
        Subsystem::Options,
    ];
    loop {
        let changes = client
            .run("idle", &subsystems[..])
            .await?
            .list("changed")
            .into_iter()
            .map(|change| change.parse::<Subsystem>())
            .collect::<std::result::Result<Vec<_>, _>>()?;
        events.send(Event::Idle(changes)).await?;
    }
}

async fn library(
    address: Address,
    mut requests: mpsc::Receiver<ReadRequest>,
    events: mpsc::Sender<Event>,
) -> WorkerResult {
    while let Some(request) = requests.recv().await {
        let mut client = address.connect().await?;
        let event = match request {
            ReadRequest::Artists { generation } => Event::Artists {
                generation,
                artists: client
                    .run("list", ("albumartistsort", "group", "albumartist"))
                    .await?
                    .pairs(),
            },
            ReadRequest::Tracks { generation, artist } => {
                let mut query = Query::new();
                query.and(Term::Tag(Borrowed("AlbumArtist")), &*artist);
                let songs =
                    client.run("find", &query).await?.structs("file")?;
                Event::Tracks {
                    generation,
                    artist,
                    songs,
                }
            }
            ReadRequest::GlobalSearch { generation } => Event::GlobalSearch {
                generation,
                songs: client.run("listallinfo", ()).await?.structs("file")?,
            },
        };
        drop(client);
        events.send(event).await?;
    }
    Ok(())
}

async fn status(client: &mut Client) -> Result<Status> {
    client.run("status", ()).await?.one()
}

async fn complete(
    client: &mut Client,
    command: Command,
    address: &str,
) -> Result<Event> {
    let (flags, mut error) = match execute(client, command).await {
        Ok(flags) => (flags, None),
        Err(mpd::error::Error::Server(error)) => {
            (Refresh::all(), Some(error.to_string()))
        }
        Err(error) => return Err(error),
    };
    let snapshot = match snapshot(client, flags).await {
        Ok(snapshot) => snapshot,
        Err(mpd::error::Error::Server(snapshot_error)) => {
            error = Some(match error {
                Some(error) => format!("{error}; {snapshot_error}"),
                None => snapshot_error.to_string(),
            });
            Snapshot::default()
        }
        Err(error) => return Err(error),
    };
    Ok(Event::Completed {
        snapshot: Box::new(snapshot),
        error: error.map(|error| format!("MPD at {}: {error}", address)),
    })
}

async fn snapshot(client: &mut Client, flags: Refresh) -> Result<Snapshot> {
    let mut snapshot = Snapshot::default();
    if flags.contains(Refresh::QUEUE) {
        snapshot.queue =
            Some(client.run("playlistinfo", ()).await?.structs("file")?);
    }
    if flags.contains(Refresh::CURRENT_SONG) {
        snapshot.current_song =
            Some(client.run("currentsong", ()).await?.structs("file")?.pop());
    }
    if flags.contains(Refresh::STATUS) {
        snapshot.status = Some(status(client).await?);
    }
    Ok(snapshot)
}

async fn execute(client: &mut Client, command: Command) -> Result<Refresh> {
    let playback = Refresh::STATUS | Refresh::CURRENT_SONG;
    match command {
        Command::TogglePause => {
            client.run("pause", ()).await?;
            Ok(Refresh::STATUS)
        }
        Command::Next | Command::Previous => {
            if status(client).await?.state != mpd::State::Stop {
                if matches!(command, Command::Next) {
                    client.run("next", ()).await?;
                } else {
                    client.run("previous", ()).await?;
                }
            }
            Ok(playback)
        }
        Command::Seek { seconds } => {
            let status = status(client).await?;
            if let (Some((position, total)), Some(song)) =
                (status.time, status.song)
            {
                let delta = Duration::from_secs(seconds.unsigned_abs());
                let position = if seconds < 0 {
                    position.saturating_sub(delta)
                } else {
                    position.checked_add(delta).unwrap_or(total).min(total)
                };
                if position >= total {
                    client.run("next", ()).await?;
                } else {
                    client
                        .run("seekid", (song.id, position.as_secs_f64()))
                        .await?;
                }
            }
            Ok(playback)
        }
        Command::UpdateDatabase => {
            client.run("update", ()).await?;
            Ok(Refresh::STATUS)
        }
        Command::Toggle(toggle) => {
            let status = status(client).await?;
            let (option, value) = match toggle {
                Toggle::Repeat => ("repeat", !status.repeat),
                Toggle::Random => ("random", !status.random),
                Toggle::Single => ("single", !status.single),
                Toggle::Consume => ("consume", !status.consume),
            };
            client.run(option, u8::from(value)).await?;
            Ok(Refresh::STATUS)
        }
        Command::Clear => {
            client.run("clear", ()).await?;
            Ok(Refresh::all())
        }
        Command::Play(id) => {
            client.run("playid", id).await?;
            Ok(playback)
        }
        Command::Swap(from, to) => {
            client.run("swapid", (from, to)).await?;
            Ok(Refresh::all())
        }
        Command::Delete(id) => {
            client.run("deleteid", id).await?;
            Ok(Refresh::all())
        }
        Command::Enqueue(target) => {
            let mut query = Query::new();
            match target {
                QueueTarget::Artist(artist) => {
                    query.and(Term::Tag(Borrowed("AlbumArtist")), artist);
                }
                QueueTarget::Album { artist, album } => {
                    query
                        .and(Term::Tag(Borrowed("AlbumArtist")), artist)
                        .and(Term::Tag(Borrowed("Album")), album);
                }
                QueueTarget::Song(file) => {
                    query.and(Term::File, file);
                }
            }
            client.run("findadd", &query).await?;
            let status = status(client).await?;
            if status.queue_len > 0 && status.state == mpd::State::Stop {
                client.run("play", 0u32).await?;
            }
            Ok(Refresh::all())
        }
    }
}
