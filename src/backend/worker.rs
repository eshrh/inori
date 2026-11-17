use super::connection::{SharedEndpoint, SocketControl};
use super::{
    Command, Event, EventSender, QueueTarget, ReadRequest, Refresh, Request,
    Snapshot, Toggle,
};
use mpd::client::StreamTypes;
use mpd::idle::IdleClient;
use mpd::{Client, Query, Subsystem, Term};
use std::borrow::Cow::Borrowed;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;

type Result<T> = std::result::Result<T, mpd::error::Error>;

pub(super) fn commands(
    address: Option<String>,
    endpoint: Arc<SharedEndpoint>,
    socket: Arc<SocketControl>,
    requests: Receiver<Request>,
    events: EventSender,
) {
    run("command", &endpoint, &socket, &events, || {
        let mut client = endpoint.select(address.as_deref(), &socket)?;
        while let Ok(request) = requests.recv() {
            if events.stopped.load(Ordering::Acquire) {
                break;
            }
            let event = match request {
                Request::Command(command) => {
                    complete(&mut client, command, &endpoint)
                }
                Request::Refresh(flags) => snapshot(&mut client, flags)
                    .map(|snapshot| Event::Refreshed(Box::new(snapshot))),
            }
            .map_err(|error| error.to_string())?;
            if !events.send(event) {
                break;
            }
        }
        Ok(())
    });
}

pub(super) fn reads(
    endpoint: Arc<SharedEndpoint>,
    socket: Arc<SocketControl>,
    requests: Receiver<ReadRequest>,
    events: EventSender,
) {
    run("library", &endpoint, &socket, &events, || {
        while let Ok(request) = requests.recv() {
            if events.stopped.load(Ordering::Acquire) {
                break;
            }
            let mut client = endpoint.connect(&socket)?;
            let result = read(&mut client, request);
            socket.close();
            drop(client);
            if !events.send(result.map_err(|error| error.to_string())?) {
                break;
            }
        }
        Ok(())
    });
}

fn read(
    client: &mut Client<StreamTypes>,
    request: ReadRequest,
) -> Result<Event> {
    Ok(match request {
        ReadRequest::Artists { generation } => Event::Artists {
            generation,
            artists: client.list_group_2((
                "albumartistsort".into(),
                "albumartist".into(),
            ))?,
        },
        ReadRequest::Tracks { generation, artist } => {
            let songs = client.find(
                Query::new().and(Term::Tag(Borrowed("AlbumArtist")), &*artist),
                None,
            )?;
            Event::Tracks {
                generation,
                artist,
                songs,
            }
        }
        ReadRequest::GlobalSearch { generation } => Event::GlobalSearch {
            generation,
            songs: client.listallinfo()?,
        },
    })
}

pub(super) fn idle(
    endpoint: Arc<SharedEndpoint>,
    socket: Arc<SocketControl>,
    requests: Receiver<()>,
    events: EventSender,
) {
    run("idle", &endpoint, &socket, &events, || {
        let client = endpoint.connect(&socket)?;
        let mut idle = IdleClient::new(
            client,
            &[
                Subsystem::Database,
                Subsystem::Player,
                Subsystem::Options,
                Subsystem::Update,
            ],
        )
        .map_err(|error| error.to_string())?;
        while requests.recv().is_ok() {
            if events.stopped.load(Ordering::Acquire) {
                break;
            }
            if !events.send(Event::Idle(
                idle.get().map_err(|error| error.to_string())?,
            )) {
                break;
            }
        }
        Ok(())
    });
}

fn run(
    name: &str,
    endpoint: &SharedEndpoint,
    socket: &SocketControl,
    events: &EventSender,
    worker: impl FnOnce() -> std::result::Result<(), String>,
) {
    let result = worker();
    socket.close();
    if let Err(error) = result {
        events.send(Event::Error(format!(
            "MPD {name} worker at {}: {error}",
            endpoint.description()
        )));
    }
}

fn complete(
    client: &mut Client<StreamTypes>,
    command: Command,
    endpoint: &SharedEndpoint,
) -> Result<Event> {
    let (flags, mut error) = match execute(client, command) {
        Ok(flags) => (flags, None),
        Err(mpd::error::Error::Server(error)) => {
            (Refresh::all(), Some(error.to_string()))
        }
        Err(error) => return Err(error),
    };
    let snapshot = match snapshot(client, flags) {
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
        error: error
            .map(|error| format!("MPD at {}: {error}", endpoint.description())),
    })
}

fn snapshot(
    client: &mut Client<StreamTypes>,
    flags: Refresh,
) -> Result<Snapshot> {
    let mut snapshot = Snapshot::default();
    if flags.contains(Refresh::QUEUE) {
        snapshot.queue = Some(client.queue()?);
    }
    if flags.contains(Refresh::CURRENT_SONG) {
        snapshot.current_song = Some(client.currentsong()?);
    }
    if flags.contains(Refresh::STATUS) {
        snapshot.status = Some(client.status()?);
    }
    Ok(snapshot)
}

fn execute(
    client: &mut Client<StreamTypes>,
    command: Command,
) -> Result<Refresh> {
    let playback = Refresh::STATUS | Refresh::CURRENT_SONG;
    match command {
        Command::TogglePause => {
            client.toggle_pause()?;
            Ok(Refresh::STATUS)
        }
        Command::Next | Command::Previous => {
            if client.status()?.state != mpd::State::Stop {
                if matches!(command, Command::Next) {
                    client.next()?;
                } else {
                    client.prev()?;
                }
            }
            Ok(playback)
        }
        Command::Seek { seconds } => {
            let status = client.status()?;
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
                    client.next()?;
                } else {
                    client.seek(song.id, position)?;
                }
            }
            Ok(playback)
        }
        Command::UpdateDatabase => {
            client.update()?;
            Ok(Refresh::STATUS)
        }
        Command::Toggle(toggle) => {
            let status = client.status()?;
            match toggle {
                Toggle::Repeat => client.repeat(!status.repeat),
                Toggle::Random => client.random(!status.random),
                Toggle::Single => client.single(!status.single),
                Toggle::Consume => client.consume(!status.consume),
            }?;
            Ok(Refresh::STATUS)
        }
        Command::Clear => {
            client.clear()?;
            Ok(Refresh::all())
        }
        Command::Play(id) => {
            client.switch(id)?;
            Ok(playback)
        }
        Command::Swap(from, to) => {
            client.swap(from, to)?;
            Ok(Refresh::all())
        }
        Command::Delete(id) => {
            client.delete(id)?;
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
            client.findadd(&query)?;
            let status = client.status()?;
            if status.queue_len > 0 && status.state == mpd::State::Stop {
                client.switch(0)?;
            }
            Ok(Refresh::all())
        }
    }
}
