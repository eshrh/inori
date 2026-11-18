#![allow(dead_code)]

include!("../src/main.rs");

use model::proto::{Searchable, Selector};
use model::{InfoEntry, Model, Screen};
use mpd::Song;
use update::build_library::{apply_artists, apply_tracks};

fn model() -> Model {
    Model {
        state: model::State::Running,
        error: None,
        status: mpd::Status::default(),
        screen: Screen::Library,
        previous_screen: Screen::Queue,
        library: model::LibraryState::new(),
        queue: model::QueueSelector::new(),
        currentsong: None,
        matcher: nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT),
        config: config::Config::default(),
        parse_state: Vec::new(),
        frame_size: ratatui::layout::Rect::new(0, 0, 100, 30),
        title_alias_store: Box::new(model::title_alias::NoopTitleAliasStore),
        aliases: model::title_alias::AliasMaps::default(),
    }
}

fn artists(model: &mut Model) {
    apply_artists(
        model,
        vec![
            ("AlbumArtist".into(), "Artist A".into()),
            ("AlbumArtistSort".into(), "Artist A".into()),
            ("AlbumArtist".into(), "Artist B".into()),
            ("AlbumArtistSort".into(), "Artist B".into()),
        ],
    );
}

fn song(artist: &str, title: &str) -> Song {
    Song {
        file: format!("{artist}/{title}.flac"),
        title: Some(title.into()),
        artist: Some(artist.into()),
        tags: vec![
            ("AlbumArtist".into(), artist.into()),
            ("Album".into(), "Album".into()),
        ],
        ..Song::default()
    }
}

fn target(title: &str) -> InfoEntry {
    InfoEntry {
        file: format!("Artist B/{title}.flac"),
        artist: "Artist B".into(),
        artist_sort: None,
        album: Some("Album".into()),
        title: Some(title.into()),
        album_alias: None,
        album_alias_variants: Vec::new(),
        title_alias: None,
        title_alias_variants: Vec::new(),
    }
}

fn large_track_query() -> (Model, matching::Matched) {
    let mut model = model();
    artists(&mut model);
    apply_tracks(
        &mut model,
        "Artist A",
        (0..matching::SYNC_LIMIT)
            .map(|i| song("Artist A", &format!("Needle{i}")))
            .collect(),
    );
    model.library.set_selected(Some(0));
    model.library.active = model::LibActiveSelector::TrackSelector;
    model
        .library
        .selected_item_mut()
        .unwrap()
        .set_selected(Some(0));
    model.library.selected_item_mut().unwrap().search.set_on();
    model.state = model::State::Searching;
    update::handle_key(
        &mut model,
        ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Char('N'),
            ratatui::crossterm::event::KeyModifiers::empty(),
        ),
    );
    let job = model
        .pending_search()
        .expect("large query was not deferred");
    let matches = matching::rank(
        &job.query,
        &job.haystacks,
        &mut model.matcher,
        job.top_k,
        || false,
    )
    .unwrap();
    let result = matching::Matched {
        target: job.target,
        query: job.query,
        haystacks: job.haystacks,
        matches,
    };
    update::handle_key(
        &mut model,
        ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Esc,
            ratatui::crossterm::event::KeyModifiers::empty(),
        ),
    );
    (model, result)
}

#[test]
fn a_delayed_track_query_must_not_undo_a_manual_fold_after_escape() {
    let (mut model, result) = large_track_query();
    update::handle_msg(&mut model, update::Message::Fold);
    assert!(!model.library.selected_item().unwrap().albums[0].expanded);
    assert!(!model.apply_match(result));
    assert!(!model.library.selected_item().unwrap().albums[0].expanded,
        "query completion reopened an album the user folded after leaving search");
}

#[test]
fn a_delayed_track_query_must_not_undo_manual_navigation_after_escape() {
    let (mut model, result) = large_track_query();
    update::handle_msg(
        &mut model,
        update::Message::Direction(update::Dirs::Vert(
            update::Vertical::Bottom,
        )),
    );
    let before = model.library.selected_item().unwrap().selected();
    assert_eq!(before, Some(matching::SYNC_LIMIT));
    assert!(!model.apply_match(result));
    assert_eq!(
        model.library.selected_item().unwrap().selected(),
        before,
        "query completion reset a cursor the user moved after leaving search"
    );
}

fn large_global_query() -> (Model, matching::Matched) {
    let mut model = model();
    artists(&mut model);
    model.state = model::State::Searching;
    model.library.global_search.entries.filter.set_on();
    let mut songs: Vec<_> = (0..matching::SYNC_LIMIT)
        .map(|i| song("Artist A", &format!("Filler{i}")))
        .collect();
    songs.push(song("Artist B", "Needle"));
    model.apply_global_search_contents(songs);
    update::handle_key(
        &mut model,
        ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Char('N'),
            ratatui::crossterm::event::KeyModifiers::empty(),
        ),
    );
    let job = model
        .pending_search()
        .expect("global query was not deferred");
    let matches = matching::rank(
        &job.query,
        &job.haystacks,
        &mut model.matcher,
        job.top_k,
        || false,
    )
    .unwrap();
    let result = matching::Matched {
        target: job.target,
        query: job.query,
        haystacks: job.haystacks,
        matches,
    };
    (model, result)
}

#[test]
fn entering_a_pending_global_query_must_not_enqueue_the_previous_result() {
    let (mut model, _result) = large_global_query();
    let action = update::handle_key(
        &mut model,
        ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Enter,
            ratatui::crossterm::event::KeyModifiers::empty(),
        ),
    )
    .unwrap();
    assert!(
        matches!(action.command, backend::Command::Enqueue(backend::QueueTarget::Song(ref file)) if file == "Artist B/Needle.flac"),
        "Enter acted on the previous result before the typed query completed"
    );
}

#[test]
fn the_completed_global_query_selects_the_expected_new_result() {
    let (mut model, result) = large_global_query();
    assert!(model.apply_match(result));
    let action = update::handle_key(
        &mut model,
        ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Enter,
            ratatui::crossterm::event::KeyModifiers::empty(),
        ),
    )
    .unwrap();
    assert!(
        matches!(action.command, backend::Command::Enqueue(backend::QueueTarget::Song(ref file)) if file == "Artist B/Needle.flac")
    );
}

#[test]
fn closing_a_pending_global_query_must_still_finish_its_jump() {
    let (mut model, result) = large_global_query();
    update::handle_key(
        &mut model,
        ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Esc,
            ratatui::crossterm::event::KeyModifiers::empty(),
        ),
    );
    model.apply_match(result);
    assert_eq!(
        model.library.selected_item().unwrap().name,
        "Artist B",
        "Esc discarded the latest typed global query's deferred jump"
    );
}

#[test]
fn pending_track_query_finishes_before_rank_navigation() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let (mut model, _) = large_track_query();
    model.state = model::State::Searching;
    update::handle_key(
        &mut model,
        KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()),
    );
    assert_eq!(
        model.library.selected_item().unwrap().search.cache.query,
        "N"
    );
    assert!(
        model.pending_search().is_some(),
        "ordinary query edits should stay deferred"
    );
    update::handle_key(
        &mut model,
        KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
    );
    let artist = model.library.selected_item().unwrap();
    assert_eq!(artist.search.cache.query, "Ne");
    assert_eq!(artist.selected_item().unwrap().rank, Some(1));
    update::handle_key(
        &mut model,
        KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
    );
    assert_eq!(
        model
            .library
            .selected_item()
            .unwrap()
            .selected_item()
            .unwrap()
            .rank,
        Some(0)
    );
}

#[test]
fn switching_screens_finishes_a_pending_global_query() {
    let (mut model, _) = large_global_query();
    update::handle_key(
        &mut model,
        ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Tab,
            ratatui::crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(matches!(model.screen, Screen::Queue));
    assert_eq!(model.library.selected_item().unwrap().name, "Artist B");
}

#[test]
fn entering_a_pending_queue_query_plays_its_current_result() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut model = model();
    model.screen = Screen::Queue;
    let songs = (0..=matching::SYNC_LIMIT)
        .map(|index| {
            let mut entry = song(
                "Artist A",
                if index == matching::SYNC_LIMIT {
                    "Needle"
                } else {
                    "Filler"
                },
            );
            entry.place = Some(mpd::song::QueuePlace {
                id: mpd::Id(index as u32),
                pos: index as u32,
                prio: 0,
            });
            entry
        })
        .collect();
    model.queue.replace_songs(songs, &mut model.matcher, 30);
    model.queue.set_selected(Some(0));
    model.queue.songs.filter.set_on();
    model.state = model::State::Searching;
    update::handle_key(
        &mut model,
        KeyEvent::new(KeyCode::Char('N'), KeyModifiers::empty()),
    );
    assert!(model.pending_search().is_some());
    let action = update::handle_key(
        &mut model,
        KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()),
    )
    .unwrap();
    assert!(
        matches!(action.command, backend::Command::Play(mpd::Id(id)) if id == matching::SYNC_LIMIT as u32)
    );
}

#[test]
fn completion_boundaries_exclude_normal_query_edits() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    for code in [KeyCode::Enter, KeyCode::Esc, KeyCode::Tab] {
        assert!(update::search_boundary(KeyEvent::new(
            code,
            KeyModifiers::empty()
        )));
    }
    for character in ['n', 'p'] {
        assert!(update::search_boundary(KeyEvent::new(
            KeyCode::Char(character),
            KeyModifiers::CONTROL
        )));
        assert!(!update::search_boundary(KeyEvent::new(
            KeyCode::Char(character),
            KeyModifiers::empty()
        )));
    }
    assert!(!update::search_boundary(KeyEvent::new(
        KeyCode::Char('u'),
        KeyModifiers::CONTROL
    )));
}
