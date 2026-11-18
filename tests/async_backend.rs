#![cfg(unix)]
#![allow(dead_code)]

#[path = "../src"]
mod implementation {
    pub(crate) mod backend;
}
use implementation::backend;

use backend::{
    Backend, Command, Event, QueueTarget, ReadRequest, Refresh, Session, Toggle,
};
use mpd::State as PlayState;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};

#[derive(Default)]
struct Gate {
    enabled: AtomicBool,
    entered: AtomicBool,
    open: Mutex<bool>,
    changed: Condvar,
}

impl Gate {
    fn arm(&self) {
        self.entered.store(false, Ordering::Release);
        *self.open.lock().unwrap() = false;
        self.enabled.store(true, Ordering::Release);
    }

    fn wait(&self) {
        self.entered.store(true, Ordering::Release);
        let _guard = self
            .changed
            .wait_while(self.open.lock().unwrap(), |open| !*open)
            .unwrap();
    }

    fn release(&self) {
        *self.open.lock().unwrap() = true;
        self.changed.notify_all();
    }

    async fn entered(&self) {
        timeout(Duration::from_secs(2), async {
            while !self.entered.load(Ordering::Acquire) {
                sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("MPD request did not reach the delayed operation");
    }
}

#[derive(Clone)]
struct Entry {
    id: u32,
    file: String,
}

#[derive(Default)]
struct State {
    songs: Vec<Entry>,
    playing: Option<u32>,
    paused: bool,
    repeat: bool,
    elapsed: u64,
    commands: Vec<String>,
    disconnect_after: Option<String>,
    reject_bulk: bool,
    status_override: Option<String>,
}

impl State {
    fn status(&self) -> String {
        if let Some(status) = &self.status_override {
            return status.clone();
        }
        let mut result = format!(
            "state: {}\nplaylistlength: {}\nrepeat: {}\n",
            if self.playing.is_none() {
                "stop"
            } else if self.paused {
                "pause"
            } else {
                "play"
            },
            self.songs.len(),
            u8::from(self.repeat),
        );
        if let Some(song) = self
            .playing
            .and_then(|id| self.songs.iter().position(|song| song.id == id))
        {
            result.push_str(&format!(
                "song: {song}\nsongid: {}\ntime: {}:60\nelapsed: {}\nduration: 60\n",
                self.songs[song].id, self.elapsed, self.elapsed,
            ));
        }
        result
    }

    fn info(&self, entry: &Entry) -> String {
        let position =
            self.songs.iter().position(|s| s.id == entry.id).unwrap();
        format!(
            "file: {}\nTitle: {}\nAlbumArtist: Artist\nAlbum: Album\nPos: {position}\nId: {}\n",
            entry.file, entry.file, entry.id,
        )
    }

    fn execute(&mut self, parts: &[String]) -> String {
        match parts[0].as_str() {
            "status" => self.status(),
            "replay_gain_status" => "replay_gain_mode: off\n".into(),
            "playlistinfo" => self.songs.iter().map(|s| self.info(s)).collect(),
            "currentsong" => self
                .songs
                .iter()
                .find(|s| Some(s.id) == self.playing)
                .map(|s| self.info(s))
                .unwrap_or_default(),
            "findadd" => {
                let id = 100 + self.songs.len() as u32;
                let file = parts[1]
                    .strip_prefix("(file == ")
                    .and_then(|query| query.strip_suffix(')'))
                    .map(arguments)
                    .map(|mut values| values.remove(0))
                    .unwrap_or_else(|| "matched.flac".into());
                self.songs.push(Entry { id, file });
                String::new()
            }
            "play" => {
                self.playing = Some(self.songs[parts[1].parse::<usize>().unwrap()].id);
                self.paused = false;
                String::new()
            }
            "repeat" => {
                self.repeat = parts[1] == "1";
                String::new()
            }
            "seekid" => {
                assert_eq!(Some(parts[1].parse().unwrap()), self.playing);
                self.elapsed = parts[2].parse().unwrap();
                String::new()
            }
            "clear" => {
                self.songs.clear();
                self.playing = None;
                String::new()
            }
            "deleteid" => {
                let id: u32 = parts[1].parse().unwrap();
                let previous = self.songs.len();
                self.songs.retain(|song| song.id != id);
                if self.songs.len() == previous {
                    "ACK [50@0] {deleteid} No such song\n".into()
                } else {
                    String::new()
                }
            }
            "listallinfo" if self.reject_bulk => {
                "ACK [4@0] {listallinfo} permission denied\n".into()
            }
            "listallinfo" | "find" => {
                "file: alpha.flac\nTitle: Alpha\nAlbumArtist: Artist\nAlbum: Album\n".into()
            }
            "list" => "AlbumArtist: Artist\nAlbumArtistSort: Artist\n".into(),
            "noidle" => String::new(),
            other => panic!("unexpected MPD command {other}"),
        }
    }
}

struct Peer {
    path: PathBuf,
    state: Arc<Mutex<State>>,
    stopped: Arc<AtomicBool>,
    gate: Arc<Gate>,
    accept: Option<thread::JoinHandle<()>>,
}

impl Peer {
    fn new(delayed: &'static str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "inori-async-{}-{}.sock",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        let listener = UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).unwrap();
        let state = Arc::new(Mutex::new(State::default()));
        let stopped = Arc::new(AtomicBool::new(false));
        let gate = Arc::new(Gate::default());
        gate.enabled.store(
            !matches!(delayed, "status" | "playlistinfo"),
            Ordering::Release,
        );
        let worker_state = state.clone();
        let worker_stopped = stopped.clone();
        let worker_gate = gate.clone();
        let accept = thread::spawn(move || {
            while !worker_stopped.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((socket, _)) => {
                        let state = worker_state.clone();
                        let gate = worker_gate.clone();
                        let reader = socket.try_clone().unwrap();
                        thread::spawn(move || {
                            serve(socket, reader, state, delayed, gate, None)
                        });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => panic!("fake MPD accept failed: {e}"),
                }
            }
        });
        Self {
            path,
            state,
            stopped,
            gate,
            accept: Some(accept),
        }
    }

    async fn start(&self) -> (Backend, Session, mpsc::Receiver<Event>) {
        connected(Backend::start(Some(
            self.path.to_string_lossy().into_owned(),
        )))
        .await
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.gate.release();
        self.stopped.store(true, Ordering::Release);
        self.accept.take().unwrap().join().unwrap();
        let _ = std::fs::remove_file(&self.path);
    }
}

struct TcpPeer {
    address: String,
    state: Arc<Mutex<State>>,
    connections: Arc<AtomicUsize>,
    stopped: Arc<AtomicBool>,
    accept: Option<thread::JoinHandle<()>>,
}

impl TcpPeer {
    fn new(inactivity: Duration) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        listener.set_nonblocking(true).unwrap();
        let state = Arc::new(Mutex::new(State::default()));
        let connections = Arc::new(AtomicUsize::new(0));
        let stopped = Arc::new(AtomicBool::new(false));
        let worker_state = state.clone();
        let worker_connections = connections.clone();
        let worker_stopped = stopped.clone();
        let accept = thread::spawn(move || {
            while !worker_stopped.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((socket, _)) => {
                        worker_connections.fetch_add(1, Ordering::Release);
                        socket.set_read_timeout(Some(inactivity)).unwrap();
                        let reader = socket.try_clone().unwrap();
                        let idle_socket = socket.try_clone().unwrap();
                        let state = worker_state.clone();
                        thread::spawn(move || {
                            serve(
                                socket,
                                reader,
                                state,
                                "",
                                Arc::new(Gate::default()),
                                Some(idle_socket),
                            );
                        });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => panic!("fake TCP MPD accept failed: {e}"),
                }
            }
        });
        Self {
            address,
            state,
            connections,
            stopped,
            accept: Some(accept),
        }
    }

    async fn start(&self) -> (Backend, Session, mpsc::Receiver<Event>) {
        connected(Backend::start(Some(self.address.clone()))).await
    }
}

impl Drop for TcpPeer {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        self.accept.take().unwrap().join().unwrap();
    }
}

fn arguments(line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut argument = String::new();
    let mut quoted = false;
    let mut escaped = false;
    let mut started = false;
    for character in line.chars() {
        if escaped {
            argument.push(character);
            escaped = false;
        } else if character == '\\' && quoted {
            escaped = true;
        } else if character == '"' {
            quoted = !quoted;
            started = true;
        } else if character.is_whitespace() && !quoted {
            if started {
                result.push(std::mem::take(&mut argument));
                started = false;
            }
        } else {
            argument.push(character);
            started = true;
        }
    }
    assert!(!quoted && !escaped, "invalid MPD quoting: {line:?}");
    if started {
        result.push(argument);
    }
    result
}

fn serve(
    mut socket: impl Read + Write,
    reader: impl Read,
    state: Arc<Mutex<State>>,
    delayed: &str,
    gate: Arc<Gate>,
    idle_socket: Option<TcpStream>,
) {
    if delayed == "greeting" {
        gate.wait();
    }
    if socket.write_all(b"OK MPD 0.23.0\n").is_err() {
        return;
    }
    let reader = BufReader::new(reader);
    let mut list = false;
    let mut response = String::new();
    for raw in reader.lines() {
        let Ok(line) = raw else {
            return;
        };
        state.lock().unwrap().commands.push(line.clone());
        let parts = arguments(&line);
        if parts[0] == delayed && gate.enabled.load(Ordering::Acquire) {
            gate.wait();
        }
        match parts[0].as_str() {
            "idle" => {
                if let Some(socket) = &idle_socket {
                    socket.set_read_timeout(None).unwrap();
                }
                continue;
            }
            "command_list_begin" => {
                list = true;
                continue;
            }
            "command_list_end" => list = false,
            _ => response.push_str(&state.lock().unwrap().execute(&parts)),
        }
        {
            let mut state = state.lock().unwrap();
            if state.disconnect_after.as_deref() == Some(&parts[0]) {
                state.disconnect_after.take();
                return;
            }
        }
        if !list {
            if !response.starts_with("ACK ") {
                response.push_str("OK\n");
            }
            if socket.write_all(response.as_bytes()).is_err() {
                return;
            }
            response.clear();
        }
    }
}

async fn connected(
    (backend, mut events): (Backend, mpsc::Receiver<Event>),
) -> (Backend, Session, mpsc::Receiver<Event>) {
    let Event::Connected { session, snapshot } = next(&mut events).await else {
        panic!("backend did not establish a session");
    };
    assert!(snapshot.status.is_some());
    assert!(snapshot.queue.is_some());
    assert!(snapshot.current_song.is_some());
    (backend, session, events)
}

async fn next(events: &mut mpsc::Receiver<Event>) -> Event {
    let event = timeout(Duration::from_secs(2), events.recv())
        .await
        .expect("MPD response timed out")
        .expect("MPD event stream closed");
    if let Event::Error(error) = &event {
        panic!("{error}");
    }
    event
}

#[tokio::test]
async fn startup_and_shutdown_do_not_wait_for_greeting() {
    let peer = Peer::new("greeting");
    let start = Instant::now();
    let (backend, events) =
        Backend::start(Some(peer.path.to_string_lossy().into_owned()));
    assert!(start.elapsed() < Duration::from_millis(250));
    peer.gate.entered().await;
    let start = Instant::now();
    backend.shutdown();
    assert!(start.elapsed() < Duration::from_millis(350));
    timeout(Duration::from_secs(1), async {
        while !events.is_closed() {
            sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .expect("startup task survived shutdown");
}

#[tokio::test]
async fn a_blocked_bulk_read_does_not_block_commands_and_keeps_generation() {
    let peer = Peer::new("listallinfo");
    let (backend, session, mut events) = peer.start().await;
    session
        .try_read(ReadRequest::GlobalSearch { generation: 42 })
        .unwrap();
    peer.gate.entered().await;
    session
        .try_command(Command::Toggle(Toggle::Repeat))
        .unwrap();
    let Event::Completed {
        snapshot,
        error: None,
    } = next(&mut events).await
    else {
        panic!("control command did not complete while bulk read was blocked");
    };
    assert!(snapshot.status.unwrap().repeat);
    peer.gate.release();
    let Event::GlobalSearch { generation, songs } = next(&mut events).await
    else {
        panic!("expected global search completion");
    };
    assert_eq!(generation, 42);
    assert_eq!(songs[0].file, "alpha.flac");
    backend.shutdown();
}

#[tokio::test]
async fn rapid_enqueue_toggle_and_seek_commands_preserve_order_and_intent() {
    let peer = Peer::new("");
    let (backend, session, mut events) = peer.start().await;
    for command in [
        Command::Enqueue(QueueTarget::Song("alpha.flac".into())),
        Command::Enqueue(QueueTarget::Song("beta.flac".into())),
        Command::Toggle(Toggle::Repeat),
        Command::Toggle(Toggle::Repeat),
        Command::Seek { seconds: 5 },
        Command::Seek { seconds: 5 },
    ] {
        session.try_command(command).unwrap();
    }
    let mut final_status = None;
    for _ in 0..6 {
        let Event::Completed {
            snapshot,
            error: None,
        } = next(&mut events).await
        else {
            panic!("expected command snapshot");
        };
        final_status = snapshot.status;
    }
    let status = final_status.unwrap();
    assert_eq!(status.state, PlayState::Play);
    assert!(!status.repeat);
    assert_eq!(status.time.unwrap().0.as_secs(), 10);
    let state = peer.state.lock().unwrap();
    assert_eq!(
        state
            .songs
            .iter()
            .map(|s| s.file.as_str())
            .collect::<Vec<_>>(),
        ["alpha.flac", "beta.flac"],
        "wire: {:?}",
        state.commands
    );
    let edits: Vec<_> = state
        .commands
        .iter()
        .filter(|line| {
            ["findadd ", "play ", "repeat ", "seekid "]
                .iter()
                .any(|prefix| line.starts_with(prefix))
        })
        .map(String::as_str)
        .collect();
    assert_eq!(
        edits,
        [
            r#"findadd "(file == \"alpha.flac\")""#,
            "play \"0\"",
            r#"findadd "(file == \"beta.flac\")""#,
            "repeat \"1\"",
            "repeat \"0\"",
            "seekid \"100\" \"5\"",
            "seekid \"100\" \"10\"",
        ]
    );
    drop(state);
    backend.shutdown();
}

#[tokio::test]
async fn bounded_request_queue_reports_pressure_without_blocking() {
    let peer = Peer::new("status");
    let (backend, session, _events) = peer.start().await;
    peer.gate.arm();
    session.try_refresh(Refresh::STATUS).unwrap();
    peer.gate.entered().await;
    let start = Instant::now();
    let mut submitted = 0;
    loop {
        match session.try_command(Command::Clear) {
            Ok(()) => submitted += 1,
            Err(backend::SubmitError::Full) => break,
            Err(error) => panic!("unexpected submission error {error}"),
        }
        assert!(submitted < 100, "request queue grew without bounds");
    }
    assert!(submitted > 0);
    assert!(start.elapsed() < Duration::from_millis(100));
    let start = Instant::now();
    backend.shutdown();
    assert!(start.elapsed() < Duration::from_millis(350));
}

#[tokio::test]
async fn library_reads_succeed_after_a_parked_tcp_connection_would_expire() {
    let peer = TcpPeer::new(Duration::from_millis(250));
    let (backend, session, mut events) = peer.start().await;
    session
        .try_read(ReadRequest::Artists { generation: 1 })
        .unwrap();
    assert!(matches!(next(&mut events).await, Event::Artists { .. }));
    let first_connections = peer.connections.load(Ordering::Acquire);
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(800) {
        session.try_refresh(Refresh::STATUS).unwrap();
        assert!(matches!(next(&mut events).await, Event::Refreshed(_)));
        sleep(Duration::from_millis(50)).await;
    }
    session
        .try_read(ReadRequest::GlobalSearch { generation: 2 })
        .unwrap();
    let Event::GlobalSearch { generation, songs } = next(&mut events).await
    else {
        panic!("library read failed after inactivity");
    };
    assert_eq!(generation, 2);
    assert_eq!(songs[0].file, "alpha.flac");
    assert!(peer.connections.load(Ordering::Acquire) > first_connections);
    backend.shutdown();
}

#[tokio::test]
async fn slow_library_responses_succeed_while_controls_remain_available() {
    let peer = Peer::new("listallinfo");
    let (backend, session, mut events) = peer.start().await;
    session
        .try_read(ReadRequest::GlobalSearch { generation: 42 })
        .unwrap();
    peer.gate.entered().await;
    sleep(Duration::from_millis(5300)).await;
    session
        .try_command(Command::Toggle(Toggle::Repeat))
        .unwrap();
    let Event::Completed {
        snapshot,
        error: None,
    } = next(&mut events).await
    else {
        panic!("a slow bulk response prevented a control command");
    };
    assert!(snapshot.status.unwrap().repeat);
    peer.gate.release();
    let Event::GlobalSearch { generation, songs } = next(&mut events).await
    else {
        panic!("slow valid library response was rejected");
    };
    assert_eq!(generation, 42);
    assert_eq!(songs[0].file, "alpha.flac");
    backend.shutdown();
}

#[tokio::test]
async fn shutdown_interrupts_a_blocked_library_response() {
    let peer = Peer::new("listallinfo");
    let (backend, session, _events) = peer.start().await;
    session
        .try_read(ReadRequest::GlobalSearch { generation: 1 })
        .unwrap();
    peer.gate.entered().await;
    let start = Instant::now();
    backend.shutdown();
    assert!(start.elapsed() < Duration::from_millis(350));
}

#[tokio::test]
async fn stale_queue_ids_complete_with_an_error_and_leave_commands_usable() {
    let peer = Peer::new("");
    {
        let mut state = peer.state.lock().unwrap();
        state.songs = vec![
            Entry {
                id: 7,
                file: "alpha.flac".into(),
            },
            Entry {
                id: 8,
                file: "beta.flac".into(),
            },
        ];
        state.playing = Some(7);
    }
    let (backend, session, mut events) = peer.start().await;
    session.try_command(Command::Delete(mpd::Id(999))).unwrap();
    let Event::Completed {
        snapshot,
        error: Some(error),
    } = next(&mut events).await
    else {
        panic!("server ACK did not complete the rejected command");
    };
    assert!(error.contains("No such song"));
    assert_eq!(snapshot.queue.unwrap().len(), 2);
    assert_eq!(snapshot.status.unwrap().queue_len, 2);
    assert_eq!(
        snapshot.current_song.unwrap().unwrap().place.unwrap().id,
        mpd::Id(7)
    );
    session
        .try_command(Command::Toggle(Toggle::Repeat))
        .unwrap();
    let Event::Completed {
        snapshot,
        error: None,
    } = next(&mut events).await
    else {
        panic!("worker did not survive a rejected queue command");
    };
    assert!(snapshot.status.unwrap().repeat);
    assert!(timeout(Duration::from_millis(100), events.recv())
        .await
        .is_err());
    backend.shutdown();
}

#[tokio::test]
async fn enqueue_targets_preserve_unicode_quotes_and_backslashes_on_the_wire() {
    let peer = Peer::new("");
    let (backend, session, mut events) = peer.start().await;
    let artist = r#"宇多田 "quoted" \ path"#;
    let album = r#"Été "夜" \ box"#;
    let file = r#"音楽/雪 "mix" \ live.flac"#;
    for target in [
        QueueTarget::Artist(artist.into()),
        QueueTarget::Album {
            artist: artist.into(),
            album: album.into(),
        },
        QueueTarget::Song(file.into()),
    ] {
        session.try_command(Command::Enqueue(target)).unwrap();
    }
    for _ in 0..3 {
        assert!(matches!(
            next(&mut events).await,
            Event::Completed { error: None, .. }
        ));
    }
    let state = peer.state.lock().unwrap();
    let queries: Vec<_> = state
        .commands
        .iter()
        .filter(|line| line.starts_with("findadd "))
        .map(|line| {
            let mut parts = arguments(line);
            assert_eq!(parts.len(), 2);
            parts.remove(1)
        })
        .collect();
    assert_eq!(
        queries,
        [
            r#"(AlbumArtist == "宇多田 \"quoted\" \\ path")"#,
            r#"((AlbumArtist == "宇多田 \"quoted\" \\ path") AND (Album == "Été \"夜\" \\ box"))"#,
            r#"(file == "音楽/雪 \"mix\" \\ live.flac")"#,
        ]
    );
    assert_eq!(state.songs.last().unwrap().file, file);
    drop(state);
    backend.shutdown();
}

#[tokio::test]
async fn shutdown_does_not_wait_for_a_full_event_channel_to_be_drained() {
    let peer = Peer::new("");
    let (backend, session, events) = peer.start().await;
    timeout(Duration::from_secs(2), async {
        let mut submitted = 0;
        while submitted < 66 {
            match session.try_command(Command::Toggle(Toggle::Repeat)) {
                Ok(()) => submitted += 1,
                Err(backend::SubmitError::Full) => {
                    sleep(Duration::from_millis(1)).await
                }
                Err(error) => panic!("unexpected submission error: {error}"),
            }
        }
        loop {
            let executed = peer
                .state
                .lock()
                .unwrap()
                .commands
                .iter()
                .filter(|line| line.starts_with("repeat "))
                .count();
            if events.len() == events.max_capacity() && executed >= 65 {
                break;
            }
            sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("worker did not reach event-channel backpressure");
    let start = Instant::now();
    backend.shutdown();
    assert!(start.elapsed() < Duration::from_millis(350));
    assert_eq!(events.len(), events.max_capacity());
    timeout(Duration::from_secs(1), async {
        while !events.is_closed() {
            sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .expect("MPD tasks survived shutdown");
    assert!(
        events.is_closed(),
        "an MPD worker retained its event sender"
    );
}

#[tokio::test]
async fn reconnect_does_not_replay_an_unknown_mutation_or_queued_old_commands()
{
    let peer = Peer::new("");
    {
        let mut state = peer.state.lock().unwrap();
        state.songs = vec![
            Entry {
                id: 7,
                file: "alpha.flac".into(),
            },
            Entry {
                id: 8,
                file: "beta.flac".into(),
            },
        ];
        state.disconnect_after = Some("deleteid".into());
    }
    let (backend, old, mut events) = peer.start().await;
    old.try_command(Command::Delete(mpd::Id(7))).unwrap();
    old.try_command(Command::Toggle(Toggle::Repeat)).unwrap();
    assert!(matches!(next(&mut events).await, Event::Disconnected(_)));
    assert_eq!(
        old.try_command(Command::Clear),
        Err(backend::SubmitError::Closed)
    );
    let Event::Connected {
        session: current,
        snapshot: initial,
    } = next(&mut events).await
    else {
        panic!("backend did not reconnect");
    };
    assert_eq!(initial.queue.as_ref().unwrap().len(), 1);
    assert!(!initial.status.as_ref().unwrap().repeat);
    current.try_refresh(Refresh::all()).unwrap();
    let Event::Refreshed(snapshot) = next(&mut events).await else {
        panic!("reconnected session did not refresh");
    };
    let queue = snapshot.queue.unwrap();
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].place.unwrap().id, mpd::Id(8));
    assert!(!snapshot.status.unwrap().repeat);
    {
        let state = peer.state.lock().unwrap();
        assert_eq!(
            state
                .commands
                .iter()
                .filter(|line| line.starts_with("deleteid "))
                .count(),
            1
        );
        assert!(!state
            .commands
            .iter()
            .any(|line| line.starts_with("repeat ")));
    }
    current
        .try_command(Command::Toggle(Toggle::Repeat))
        .unwrap();
    let Event::Completed {
        snapshot,
        error: None,
    } = next(&mut events).await
    else {
        panic!("new session did not accept a new command");
    };
    assert!(snapshot.status.unwrap().repeat);
    backend.shutdown();
}

#[tokio::test]
async fn permanent_library_ack_stops_instead_of_reconnecting_and_retrying() {
    let peer = Peer::new("");
    peer.state.lock().unwrap().reject_bulk = true;
    let (backend, session, mut events) = peer.start().await;
    session
        .try_read(ReadRequest::GlobalSearch { generation: 1 })
        .unwrap();
    let Some(Event::Error(error)) =
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap()
    else {
        panic!("permanent bulk error was not fatal");
    };
    assert!(error.contains("permission denied"));
    assert!(timeout(Duration::from_secs(2), events.recv())
        .await
        .unwrap()
        .is_none());
    assert_eq!(
        peer.state
            .lock()
            .unwrap()
            .commands
            .iter()
            .filter(|line| line.starts_with("listallinfo"))
            .count(),
        1
    );
    assert_eq!(
        session.try_command(Command::Clear),
        Err(backend::SubmitError::Closed)
    );
    backend.shutdown();
}

#[tokio::test]
async fn malformed_initial_status_never_announces_a_session_or_retries() {
    let peer = Peer::new("");
    peer.state.lock().unwrap().status_override =
        Some("state: impossible\n".into());
    let (backend, mut events) =
        Backend::start(Some(peer.path.to_string_lossy().into_owned()));
    let Some(Event::Error(_)) = timeout(Duration::from_secs(2), events.recv())
        .await
        .unwrap()
    else {
        panic!("malformed initial status was not fatal");
    };
    assert!(timeout(Duration::from_secs(2), events.recv())
        .await
        .unwrap()
        .is_none());
    assert_eq!(
        peer.state
            .lock()
            .unwrap()
            .commands
            .iter()
            .filter(|line| line.as_str() == "status")
            .count(),
        1
    );
    backend.shutdown();
}
