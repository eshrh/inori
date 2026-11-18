#![cfg(unix)]
#![allow(dead_code)]

#[path = "../src"]
mod implementation {
    pub(crate) mod backend;
    pub(crate) mod config;
    pub(crate) mod event_handler;
    pub(crate) mod matching;
    pub(crate) mod model;
    pub(crate) mod update;
    pub(crate) mod util;
    pub(crate) mod view;

    mod app {
        include!("../src/app.rs");

        #[cfg(test)]
        mod tests {
            use super::*;
            use crate::backend::Snapshot;
            use crate::config::keybind::KeybindMap;
            use crate::model::title_alias::{AliasMaps, NoopTitleAliasStore};
            use crate::model::{
                LibActiveSelector, LibraryState, QueueSelector,
            };
            use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
            use std::io::{BufRead, BufReader, Write};
            use std::os::unix::net::UnixListener;
            use std::path::PathBuf;
            use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
            use std::sync::Arc;
            use std::thread;

            struct Peer {
                path: PathBuf,
                stopped: Arc<AtomicBool>,
                accept: Option<thread::JoinHandle<()>>,
            }

            impl Peer {
                fn new() -> Self {
                    static NEXT: AtomicUsize = AtomicUsize::new(0);
                    let path = std::env::temp_dir().join(format!(
                        "inori-app-{}-{}.sock",
                        std::process::id(),
                        NEXT.fetch_add(1, Ordering::Relaxed),
                    ));
                    let listener = UnixListener::bind(&path).unwrap();
                    listener.set_nonblocking(true).unwrap();
                    let stopped = Arc::new(AtomicBool::new(false));
                    let worker_stopped = stopped.clone();
                    let accept = thread::spawn(move || {
                        while !worker_stopped.load(Ordering::Acquire) {
                            match listener.accept() {
                                Ok((mut socket, _)) => {
                                    socket
                                        .write_all(b"OK MPD 0.23.0\n")
                                        .unwrap();
                                    let reader = BufReader::new(
                                        socket.try_clone().unwrap(),
                                    );
                                    thread::spawn(move || {
                                        for line in reader.lines() {
                                            let Ok(line) = line else {
                                                return;
                                            };
                                            let command = line
                                                .split_whitespace()
                                                .next()
                                                .unwrap_or("");
                                            if command == "idle" {
                                                continue;
                                            }
                                            let response = if command
                                                == "status"
                                            {
                                                "state: stop\nplaylistlength: 0\nOK\n"
                                            } else {
                                                "OK\n"
                                            };
                                            if socket
                                                .write_all(response.as_bytes())
                                                .is_err()
                                            {
                                                return;
                                            }
                                        }
                                    });
                                }
                                Err(error)
                                    if error.kind()
                                        == std::io::ErrorKind::WouldBlock =>
                                {
                                    thread::sleep(Duration::from_millis(2));
                                }
                                Err(error) => {
                                    panic!("fake MPD accept: {error}")
                                }
                            }
                        }
                    });
                    Self {
                        path,
                        stopped,
                        accept: Some(accept),
                    }
                }

                async fn app(&self, mut model: Model) -> App {
                    model.config.mpd_address =
                        Some(self.path.to_string_lossy().into_owned());
                    let mut app = App::new(model).unwrap();
                    let event = tokio::time::timeout(
                        Duration::from_secs(2),
                        app.events.recv(),
                    )
                    .await
                    .unwrap()
                    .unwrap();
                    let backend::Event::Connected { session, .. } = event
                    else {
                        panic!("backend did not establish a session");
                    };
                    app.session = Some(session);
                    app
                }
            }

            impl Drop for Peer {
                fn drop(&mut self) {
                    self.stopped.store(true, Ordering::Release);
                    self.accept.take().unwrap().join().unwrap();
                    let _ = std::fs::remove_file(&self.path);
                }
            }

            fn model() -> Model {
                Model {
                    state: State::Running,
                    error: None,
                    status: mpd::Status::default(),
                    screen: Screen::Queue,
                    library: LibraryState::new(),
                    queue: QueueSelector::new(),
                    currentsong: None,
                    matcher: nucleo_matcher::Matcher::new(
                        nucleo_matcher::Config::DEFAULT,
                    ),
                    config: crate::config::Config::default(),
                    parse_state: Vec::new(),
                    frame_size: Rect::new(0, 0, 100, 30),
                    title_alias_store: Box::new(NoopTitleAliasStore),
                    aliases: AliasMaps::default(),
                }
            }

            fn song(id: u32, pos: u32) -> Song {
                Song {
                    file: format!("Artist/{id}.flac"),
                    title: Some(format!("Song {id}")),
                    artist: Some("Artist".into()),
                    tags: vec![
                        ("AlbumArtist".into(), "Artist".into()),
                        ("Album".into(), "Album".into()),
                    ],
                    place: Some(mpd::song::QueuePlace {
                        id: Id(id),
                        pos,
                        prio: 0,
                    }),
                    ..Song::default()
                }
            }

            fn queue(ids: &[u32]) -> Vec<Song> {
                ids.iter()
                    .enumerate()
                    .map(|(pos, id)| song(*id, pos as u32))
                    .collect()
            }

            fn ids(app: &App) -> Vec<u32> {
                app.model
                    .queue
                    .songs
                    .items
                    .iter()
                    .map(|s| s.place.unwrap().id.0)
                    .collect()
            }

            fn press(app: &mut App, code: KeyCode) {
                assert!(app
                    .handle_input(Event::Key(KeyEvent::new(
                        code,
                        KeyModifiers::empty(),
                    )))
                    .unwrap());
            }

            fn completed(app: &mut App, ids: &[u32], error: Option<&str>) {
                assert!(app
                    .apply(backend::Event::Completed {
                        snapshot: Box::new(Snapshot {
                            queue: Some(queue(ids)),
                            ..Snapshot::default()
                        }),
                        error: error.map(str::to_owned),
                    })
                    .unwrap());
            }

            fn fill_commands(app: &App) {
                let mut accepted = 0;
                loop {
                    match app
                        .session
                        .as_ref()
                        .unwrap()
                        .try_command(Command::TogglePause)
                    {
                        Ok(()) => accepted += 1,
                        Err(SubmitError::Full) => break,
                        Err(error) => {
                            panic!("unexpected submit failure: {error}")
                        }
                    }
                }
                assert_eq!(accepted, 32);
            }

            fn artists(model: &mut Model) {
                update::build_library::apply_artists(
                    model,
                    vec![
                        ("AlbumArtist".into(), "Artist".into()),
                        ("AlbumArtistSort".into(), "Artist".into()),
                        ("AlbumArtist".into(), "Other".into()),
                        ("AlbumArtistSort".into(), "Other".into()),
                    ],
                );
            }

            #[tokio::test]
            async fn refresh_and_non_edit_completions_preserve_pending_queue_edits(
            ) {
                let peer = Peer::new();
                let mut app = peer.app(model()).await;
                app.replace_queue(queue(&[10, 11, 12, 13]));
                app.model.queue.set_selected(Some(0));
                press(&mut app, KeyCode::Char('p'));
                press(&mut app, KeyCode::Right);
                press(&mut app, KeyCode::Backspace);
                press(&mut app, KeyCode::Right);
                assert_eq!(ids(&app), [12, 11, 13]);
                assert_eq!(app.unacked.len(), 4);

                app.apply(backend::Event::Completed {
                    snapshot: Box::default(),
                    error: None,
                })
                .unwrap();
                assert_eq!(app.unacked.len(), 3);
                app.apply(backend::Event::Refreshed(Box::new(Snapshot {
                    queue: Some(queue(&[10, 11, 12, 13])),
                    ..Snapshot::default()
                })))
                .unwrap();
                assert_eq!(ids(&app), [12, 11, 13]);
                assert_eq!(app.unacked.len(), 3);

                completed(&mut app, &[11, 10, 12, 13], None);
                assert_eq!(ids(&app), [12, 11, 13]);
                assert_eq!(app.unacked.len(), 2);
                completed(&mut app, &[11, 12, 13], None);
                assert_eq!(ids(&app), [12, 11, 13]);
                completed(&mut app, &[12, 11, 13], None);
                assert!(app.unacked.is_empty());
                assert_eq!(app.model.queue.selected_id(), Some(Id(11)));
                for (position, song) in
                    app.model.queue.songs.items.iter().enumerate()
                {
                    assert_eq!(song.place.unwrap().pos, position as u32);
                }
            }

            #[tokio::test]
            async fn a_rejected_command_is_completed_and_later_projection_survives(
            ) {
                let peer = Peer::new();
                let mut app = peer.app(model()).await;
                app.replace_queue(queue(&[10, 11, 12]));
                app.model.queue.set_selected(Some(0));
                press(&mut app, KeyCode::Backspace);
                press(&mut app, KeyCode::Right);
                assert_eq!(ids(&app), [12, 11]);
                completed(&mut app, &[10, 11, 12], Some("No such song"));
                assert_eq!(ids(&app), [10, 12, 11]);
                assert_eq!(app.unacked.len(), 1);
                assert_eq!(app.model.error.as_deref(), Some("No such song"));
                completed(&mut app, &[10, 12, 11], None);
                assert!(app.unacked.is_empty());
                assert_eq!(app.model.queue.selected_id(), Some(Id(11)));
            }

            #[tokio::test]
            async fn now_playing_style_follows_id_while_projection_changes_position(
            ) {
                let peer = Peer::new();
                let mut app = peer.app(model()).await;
                let songs = queue(&[10, 11, 12]);
                app.model.status.song = songs[1].place;
                app.replace_queue(songs);
                app.model.queue.set_selected(Some(0));
                press(&mut app, KeyCode::Backspace);
                assert_eq!(app.model.status.song.unwrap().pos, 1);
                assert_eq!(
                    app.model.queue.songs.items[0].place.unwrap().pos,
                    0
                );

                let area = Rect::new(0, 0, 100, 6);
                let mut buffer = ratatui::buffer::Buffer::empty(area);
                let table = crate::view::queue_renderer::make_queue(
                    &mut app.model,
                    &crate::view::Theme::default(),
                );
                ratatui::widgets::Widget::render(table, area, &mut buffer);
                let playing = &buffer[(1, 1)];
                let other = &buffer[(1, 2)];
                assert_eq!(playing.symbol(), "S");
                assert!(playing.modifier.contains(
                    ratatui::style::Modifier::ITALIC
                        | ratatui::style::Modifier::BOLD,
                ));
                assert!(!other
                    .modifier
                    .contains(ratatui::style::Modifier::BOLD));
            }

            #[tokio::test]
            async fn a_full_command_queue_does_not_delete_or_move_selection() {
                let peer = Peer::new();
                let mut app = peer.app(model()).await;
                app.replace_queue(queue(&[10, 11, 12]));
                app.model.queue.set_selected(Some(1));
                fill_commands(&app);
                press(&mut app, KeyCode::Backspace);
                assert_eq!(ids(&app), [10, 11, 12]);
                assert_eq!(app.model.queue.selected_id(), Some(Id(11)));
                assert!(app.unacked.is_empty());
                assert_eq!(
                    app.model.error.as_deref(),
                    Some("MPD request queue is full")
                );
            }

            #[tokio::test]
            async fn a_full_command_queue_does_not_advance_either_library_selector(
            ) {
                let peer = Peer::new();
                let mut model = model();
                model.screen = Screen::Library;
                artists(&mut model);
                model.config.keybindings.insert(
                    update::Message::SelectAndNext,
                    &[KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())],
                );
                let mut app = peer.app(model).await;
                fill_commands(&app);
                press(&mut app, KeyCode::Char('a'));
                assert_eq!(app.model.library.selected(), Some(0));
                update::build_library::apply_tracks(
                    &mut app.model,
                    "Artist",
                    queue(&[10, 11]),
                );
                app.model.library.active = LibActiveSelector::TrackSelector;
                app.model
                    .library
                    .selected_item_mut()
                    .unwrap()
                    .set_selected(Some(1));
                press(&mut app, KeyCode::Char('a'));
                assert_eq!(
                    app.model.library.selected_item().unwrap().selected(),
                    Some(1)
                );
                assert!(app.unacked.is_empty());
            }

            #[tokio::test]
            async fn accepted_select_and_next_advances_after_submission() {
                let peer = Peer::new();
                let mut model = model();
                model.screen = Screen::Library;
                artists(&mut model);
                model.config.keybindings = KeybindMap::default();
                model.config.keybindings.insert(
                    update::Message::SelectAndNext,
                    &[KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())],
                );
                let mut app = peer.app(model).await;
                press(&mut app, KeyCode::Char('a'));
                assert_eq!(app.model.library.selected(), Some(1));
                assert_eq!(app.unacked.len(), 1);
                app.model.library.set_selected(Some(0));
                update::build_library::apply_tracks(
                    &mut app.model,
                    "Artist",
                    queue(&[10, 11]),
                );
                app.model.library.active = LibActiveSelector::TrackSelector;
                app.model
                    .library
                    .selected_item_mut()
                    .unwrap()
                    .set_selected(Some(1));
                press(&mut app, KeyCode::Char('a'));
                assert_eq!(
                    app.model.library.selected_item().unwrap().selected(),
                    Some(2)
                );
                assert_eq!(app.unacked.len(), 2);
            }

            #[tokio::test]
            async fn stale_replies_cannot_apply_or_clear_new_generation_load_markers(
            ) {
                let peer = Peer::new();
                let mut model = model();
                model.screen = Screen::Library;
                artists(&mut model);
                model.library.global_search.entries.filter.set_on();
                model.library.global_search.loaded = true;
                let mut app = peer.app(model).await;
                app.loads.generation = 4;
                app.loads.artists = true;
                app.loads.tracks.insert("Artist".into());
                app.loads.global_search = true;
                app.apply(backend::Event::Idle(vec![Subsystem::Database]))
                    .unwrap();
                assert_eq!(app.loads.generation, 5);
                assert!(!app.loads.artists);
                assert!(app.loads.tracks.is_empty());
                assert!(!app.loads.global_search);
                app.schedule_loads();
                assert!(app.loads.artists);

                assert!(!app
                    .apply(backend::Event::Artists {
                        generation: 4,
                        artists: vec![("AlbumArtist".into(), "Stale".into())],
                    })
                    .unwrap());
                assert!(app.loads.artists);
                assert_eq!(app.model.library.artists.items[0].name, "Artist");
                app.apply(backend::Event::Artists {
                    generation: 5,
                    artists: vec![("AlbumArtist".into(), "Artist".into())],
                })
                .unwrap();
                app.schedule_loads();
                assert!(app.loads.tracks.contains("Artist"));
                assert!(app.loads.global_search);
                assert!(!app
                    .apply(backend::Event::Tracks {
                        generation: 4,
                        artist: "Artist".into(),
                        songs: queue(&[10]),
                    })
                    .unwrap());
                assert!(app.loads.tracks.contains("Artist"));
                assert!(!app.model.library.artists.items[0].fetched);
                assert!(!app
                    .apply(backend::Event::GlobalSearch {
                        generation: 4,
                        songs: queue(&[10]),
                    })
                    .unwrap());
                assert!(app.loads.global_search);
                assert!(!app.model.library.global_search.loaded);

                app.apply(backend::Event::Tracks {
                    generation: 5,
                    artist: "Artist".into(),
                    songs: queue(&[11]),
                })
                .unwrap();
                assert!(!app.loads.tracks.contains("Artist"));
                assert!(app.model.library.artists.items[0].fetched);
                app.apply(backend::Event::GlobalSearch {
                    generation: 5,
                    songs: queue(&[11]),
                })
                .unwrap();
                assert!(!app.loads.global_search);
                assert!(app.model.library.global_search.loaded);
                assert!(app
                    .model
                    .library
                    .global_search
                    .entries
                    .items
                    .iter()
                    .any(|e| e.file == "Artist/11.flac"));
            }

            #[tokio::test]
            async fn disconnected_sessions_discard_edits_and_reject_new_mutations(
            ) {
                let peer = Peer::new();
                let mut app = peer.app(model()).await;
                app.replace_queue(queue(&[10, 11, 12]));
                app.model.queue.set_selected(Some(0));
                press(&mut app, KeyCode::Backspace);
                assert_eq!(ids(&app), [11, 12]);
                app.loads.generation = 7;
                app.loads.artists = true;
                app.loads.tracks.insert("Artist".into());
                app.loads.global_search = true;
                app.loads.refresh = true;
                app.want = Refresh::all();
                app.apply(backend::Event::Disconnected(
                    "connection lost".into(),
                ))
                .unwrap();
                assert!(app.session.is_none());
                assert!(app.unacked.is_empty());
                assert!(!app.loads.refresh);
                assert!(app.want.is_empty());
                assert_eq!(app.loads.generation, 8);
                assert!(
                    !app.loads.artists
                        && app.loads.tracks.is_empty()
                        && !app.loads.global_search
                );
                assert!(
                    !app.model.library.loaded
                        && !app.model.library.global_search.loaded
                );
                press(&mut app, KeyCode::Down);
                assert_eq!(app.model.queue.selected_id(), Some(Id(12)));
                press(&mut app, KeyCode::Backspace);
                assert_eq!(ids(&app), [11, 12]);
                assert!(app.unacked.is_empty());
                assert_eq!(
                    app.model.error.as_deref(),
                    Some("MPD is disconnected")
                );
                press(&mut app, KeyCode::Char('q'));
                assert!(matches!(app.model.state, State::Done));
            }

            #[tokio::test]
            async fn reconnect_applies_authoritative_snapshot_and_invalidates_old_library_replies(
            ) {
                let peer = Peer::new();
                let mut app = peer.app(model()).await;
                app.replace_queue(queue(&[10, 11]));
                app.model.queue.set_selected(Some(0));
                app.model.currentsong = Some(song(10, 0));
                app.loads.generation = 7;
                app.apply(backend::Event::Disconnected(
                    "connection lost".into(),
                ))
                .unwrap();
                let mut replacement = peer.app(model()).await;
                app.want = Refresh::STATUS;
                app.apply(backend::Event::Connected {
                    session: replacement.session.take().unwrap(),
                    snapshot: Box::new(Snapshot {
                        queue: Some(queue(&[20, 21])),
                        status: Some(mpd::Status::default()),
                        current_song: Some(None),
                    }),
                })
                .unwrap();
                assert!(app.session.is_some());
                assert!(app.model.error.is_none());
                assert_eq!(ids(&app), [20, 21]);
                assert_eq!(app.model.queue.selected_id(), Some(Id(20)));
                assert!(app.model.currentsong.is_none());
                assert_eq!(app.want.bits(), Refresh::STATUS.bits());
                assert_eq!(app.loads.generation, 9);
                assert!(!app
                    .apply(backend::Event::Artists {
                        generation: 7,
                        artists: vec![("AlbumArtist".into(), "Stale".into())],
                    })
                    .unwrap());
                assert!(!app.model.library.loaded);
                app.schedule_loads();
                assert!(app.loads.refresh && app.want.is_empty());
                assert!(app.unacked.is_empty());
            }

            #[tokio::test]
            async fn idle_changes_accumulate_while_a_targeted_refresh_is_in_flight(
            ) {
                let peer = Peer::new();
                let mut app = peer.app(model()).await;
                app.apply(backend::Event::Idle(vec![Subsystem::Options]))
                    .unwrap();
                assert_eq!(app.want.bits(), Refresh::STATUS.bits());
                app.schedule_loads();
                assert!(app.loads.refresh && app.want.is_empty());
                app.apply(backend::Event::Idle(vec![
                    Subsystem::Player,
                    Subsystem::Queue,
                ]))
                .unwrap();
                assert_eq!(app.want.bits(), Refresh::all().bits());
                app.schedule_loads();
                assert_eq!(app.want.bits(), Refresh::all().bits());
                app.apply(backend::Event::Refreshed(Box::default()))
                    .unwrap();
                app.schedule_loads();
                assert!(app.loads.refresh && app.want.is_empty());
            }

            #[tokio::test]
            async fn closed_session_channels_do_not_make_background_scheduling_fatal(
            ) {
                let peer = Peer::new();
                let mut app = peer.app(model()).await;
                app.backend.shutdown();
                tokio::time::timeout(Duration::from_secs(1), async {
                    while !app.events.is_closed() {
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                })
                .await
                .expect("backend tasks did not stop");
                assert_eq!(
                    app.session.as_ref().unwrap().try_command(Command::Clear),
                    Err(SubmitError::Closed)
                );
                app.model.screen = Screen::Library;
                app.want = Refresh::all();
                app.schedule_loads();
                assert!(!app.loads.refresh && !app.loads.artists);
                assert_eq!(app.want.bits(), Refresh::all().bits());
                assert!(matches!(app.model.state, State::Running));
            }

            struct CountingAliases(Arc<AtomicUsize>);

            impl crate::model::title_alias::TitleAliasStore for CountingAliases {
                fn load_all(&self) -> crate::event_handler::Result<AliasMaps> {
                    self.0.fetch_add(1, Ordering::Relaxed);
                    Ok(AliasMaps::default())
                }
            }

            #[tokio::test]
            async fn update_notifications_reload_aliases_without_invalidating_library(
            ) {
                let peer = Peer::new();
                let calls = Arc::new(AtomicUsize::new(0));
                let mut model = model();
                artists(&mut model);
                model.library.global_search.loaded = true;
                model.title_alias_store =
                    Box::new(CountingAliases(calls.clone()));
                let mut app = peer.app(model).await;
                app.loads.generation = 3;
                app.loads.artists = true;
                app.loads.tracks.insert("Artist".into());
                app.loads.global_search = true;
                app.apply(backend::Event::Idle(vec![Subsystem::Update]))
                    .unwrap();
                assert_eq!(calls.load(Ordering::Relaxed), 1);
                assert_eq!(app.loads.generation, 3);
                assert!(app.model.library.loaded);
                assert!(app.model.library.global_search.loaded);
                assert!(app.loads.artists);
                assert!(app.loads.tracks.contains("Artist"));
                assert!(app.loads.global_search);
                app.apply(backend::Event::Idle(vec![Subsystem::Database]))
                    .unwrap();
                assert_eq!(app.loads.generation, 4);
                assert!(!app.model.library.loaded);
                assert!(!app.model.library.global_search.loaded);
            }
        }
    }
}

use implementation::{
    backend, config, event_handler, matching, model, update, util, view,
};
