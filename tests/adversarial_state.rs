#![allow(dead_code)]

include!("../src/main.rs");

use model::proto::Searchable;
use model::{InfoEntry, ItemRef, Model, Screen};
use mpd::Song;
use update::build_library::{apply_artists, apply_tracks};

fn model() -> Model {
    Model {
        state: model::State::Running,
        error: None,
        status: mpd::Status::default(),
        screen: Screen::Library,
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

#[test]
fn entering_a_delayed_global_result_still_finishes_its_jump() {
    let mut model = model();
    artists(&mut model);
    model.state = model::State::Searching;
    model.library.global_search.entries.filter.set_on();
    model.library.global_search.entries.filter.query = "Beta2".into();
    model.apply_global_search_contents(vec![song("Artist B", "Beta2")]);
    assert_eq!(
        model.library.pending_jump.as_ref().unwrap().file,
        "Artist B/Beta2.flac"
    );
    let command = update::handle_key(
        &mut model,
        ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Enter,
            ratatui::crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(matches!(
        command,
        Some(update::Action {
            command: backend::Command::Enqueue(_),
            ..
        })
    ));
    apply_tracks(
        &mut model,
        "Artist B",
        vec![song("Artist B", "Beta1"), song("Artist B", "Beta2")],
    );
    let selected = model
        .library
        .selected_item()
        .unwrap()
        .selected_item()
        .unwrap();
    assert!(
        matches!(selected.item, ItemRef::Song(song) if song.title.as_deref() == Some("Beta2")),
        "deferred jump was canceled by Enter: {:?}",
        selected.item
    );
}

#[test]
fn closing_global_search_still_finishes_the_selected_deferred_jump() {
    let mut model = model();
    artists(&mut model);
    model.state = model::State::Searching;
    model.library.global_search.entries.filter.set_on();
    model.library.global_search.entries.filter.query = "Beta2".into();
    model.apply_global_search_contents(vec![song("Artist B", "Beta2")]);
    assert_eq!(
        model.library.pending_jump.as_ref().unwrap().file,
        "Artist B/Beta2.flac"
    );
    update::handle_key(
        &mut model,
        ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Esc,
            ratatui::crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert!(matches!(model.state, model::State::Running));
    assert!(!model.library.global_search.entries.filter.active);
    assert!(model.library.pending_jump.is_some());
    apply_tracks(
        &mut model,
        "Artist B",
        vec![song("Artist B", "Beta1"), song("Artist B", "Beta2")],
    );
    assert!(model.library.pending_jump.is_none());
    let selected = model
        .library
        .selected_item()
        .unwrap()
        .selected_item()
        .unwrap();
    assert!(
        matches!(selected.item, ItemRef::Song(song) if song.title.as_deref() == Some("Beta2")),
        "deferred jump was canceled by Esc: {:?}",
        selected.item
    );
}
