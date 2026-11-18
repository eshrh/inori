#![allow(dead_code)]

include!("../src/main.rs");

use model::proto::{Searchable, Selector};
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
fn a_late_track_reply_updates_its_artist_without_retargeting_selection() {
    let mut model = model();
    artists(&mut model);
    model.library.set_selected(Some(1));
    apply_tracks(&mut model, "Artist A", vec![song("Artist A", "Alpha")]);
    assert_eq!(model.library.selected_item().unwrap().name, "Artist B");
    assert!(model.library.artists.items[0].fetched);
    assert_eq!(
        model.library.artists.items[0].albums[0].tracks[0]
            .title
            .as_deref(),
        Some("Alpha")
    );
    assert!(!model.library.artists.items[1].fetched);
    assert!(model.library.artists.items[1].albums.is_empty());
}

#[test]
fn the_latest_global_target_survives_both_deferred_loading_steps() {
    let mut model = model();
    model.jump_to(target("Beta1"));
    model.jump_to(target("Beta2"));
    assert!(!model.library.loaded);
    artists(&mut model);
    assert_eq!(model.library.selected_item().unwrap().name, "Artist B");
    assert_eq!(
        model.library.pending_jump.as_ref().unwrap().file,
        "Artist B/Beta2.flac"
    );
    apply_tracks(&mut model, "Artist A", vec![song("Artist A", "Alpha")]);
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
    let ItemRef::Song(song) = selected.item else {
        panic!("jump selected an album");
    };
    assert_eq!(song.file, "Artist B/Beta2.flac");
}

#[test]
fn leaving_the_screen_cancels_a_deferred_global_jump() {
    let mut model = model();
    artists(&mut model);
    model.jump_to(target("Beta2"));
    assert!(model.library.pending_jump.is_some());
    update::handle_msg(&mut model, update::Message::SwitchScreen(2));
    assert!(model.library.pending_jump.is_none());
    apply_tracks(
        &mut model,
        "Artist B",
        vec![song("Artist B", "Beta1"), song("Artist B", "Beta2")],
    );
    assert!(matches!(model.screen, Screen::Queue));
    assert!(matches!(
        model
            .library
            .selected_item()
            .unwrap()
            .selected_item()
            .unwrap()
            .item,
        ItemRef::Album(_)
    ));
}

#[test]
fn refreshed_track_matches_keep_manual_folds_and_recompute_for_new_songs() {
    let mut model = model();
    artists(&mut model);
    apply_tracks(
        &mut model,
        "Artist B",
        vec![song("Artist B", "Beta1"), song("Artist B", "Beta2")],
    );
    model.library.set_selected(Some(1));
    let artist = &mut model.library.artists.items[1];
    artist.search.active = true;
    artist.search.query = "Beta".into();
    artist.update_search(&mut model.matcher, &model.aliases);
    artist.albums[0].expanded = false;
    artist.set_selected(Some(0));
    apply_tracks(
        &mut model,
        "Artist B",
        vec![
            song("Artist B", "Beta1"),
            song("Artist B", "Beta2"),
            song("Artist B", "Beta3"),
        ],
    );
    let artist = model.library.selected_item().unwrap();
    assert_eq!(artist.search.query, "Beta");
    assert!(!artist.albums[0].expanded);
    assert_eq!(artist.search.cache.order.len(), 3);
    assert_eq!(artist.selected(), Some(0));
}

#[test]
fn zero_match_queue_refresh_keeps_the_show_all_fallback() {
    let mut model = model();
    let mut first = song("Artist A", "Alpha");
    first.place = Some(mpd::song::QueuePlace {
        id: mpd::Id(100),
        pos: 0,
        prio: 0,
    });
    model.queue.songs.filter.active = true;
    model.queue.songs.filter.query = "zzzzzz".into();
    model
        .queue
        .replace_songs(vec![first.clone()], &mut model.matcher, 30);
    model.queue.set_selected(Some(0));
    let mut second = song("Artist B", "Beta");
    second.place = Some(mpd::song::QueuePlace {
        id: mpd::Id(101),
        pos: 1,
        prio: 0,
    });
    model
        .queue
        .replace_songs(vec![first, second], &mut model.matcher, 30);
    assert_eq!(model.queue.display_len(), 2);
    assert_eq!(model.queue.songs.filter.query, "zzzzzz");
    assert_eq!(model.queue.selected_id(), Some(mpd::Id(100)));
}

#[test]
fn global_enter_uses_the_result_while_its_tracks_are_pending() {
    let mut model = model();
    artists(&mut model);
    apply_tracks(&mut model, "Artist B", vec![song("Artist B", "Old")]);
    model.jump_to(target("Old"));
    artists(&mut model);
    model.state = model::State::Searching;
    model.library.global_search.entries.filter.set_on();
    model.library.global_search.entries.filter.query = "New".into();
    model.apply_global_search_contents(vec![song("Artist B", "New")]);
    assert_eq!(
        model.library.pending_jump.as_ref().unwrap().file,
        "Artist B/New.flac"
    );
    let command = update::handle_key(
        &mut model,
        ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Enter,
            ratatui::crossterm::event::KeyModifiers::empty(),
        ),
    );
    let Some(update::Action {
        command: backend::Command::Enqueue(backend::QueueTarget::Song(file)),
        ..
    }) = command
    else {
        panic!("global selection did not enqueue a song");
    };
    assert_eq!(file, "Artist B/New.flac");
}

struct AliasStore(model::title_alias::AliasMaps);

impl model::title_alias::TitleAliasStore for AliasStore {
    fn load_all(&self) -> event_handler::Result<model::title_alias::AliasMaps> {
        Ok(self.0.clone())
    }
}

fn aliases(entries: &[(&str, &str)]) -> model::title_alias::AliasMaps {
    let mut maps = model::title_alias::AliasMaps::default();
    for (path, alias) in entries {
        maps.title.insert(
            model::title_alias::SongPath((*path).into()),
            model::title_alias::AliasEntry {
                alias: (*alias).into(),
                variants: Vec::new(),
            },
        );
    }
    maps
}

#[test]
fn unchanged_aliases_keep_completed_track_matches() {
    let mut model = model();
    artists(&mut model);
    apply_tracks(
        &mut model,
        "Artist B",
        vec![song("Artist B", "Beta1"), song("Artist B", "Beta2")],
    );
    model.library.set_selected(Some(1));
    let artist = &mut model.library.artists.items[1];
    artist.search.set_on();
    artist.search.query = "Beta".into();
    artist.update_search(&mut model.matcher, &model.aliases);
    artist.albums[0].expanded = false;
    artist.set_selected(Some(0));
    let order = artist.search.cache.order.clone();
    let indices = artist.search.cache.indices.clone();
    model.reload_aliases().unwrap();
    let artist = &model.library.artists.items[1];
    assert_eq!(artist.search.cache.order, order);
    assert_eq!(artist.search.cache.indices, indices);
    assert!(!artist.albums[0].expanded);
    assert_eq!(artist.selected(), Some(0));
}

#[test]
fn changed_aliases_recompute_track_matches_without_changing_folds_or_selection()
{
    let mut model = model();
    artists(&mut model);
    apply_tracks(
        &mut model,
        "Artist B",
        vec![song("Artist B", "Beta1"), song("Artist B", "Beta2")],
    );
    let artist = &mut model.library.artists.items[1];
    artist.search.set_on();
    artist.search.query = "Needle".into();
    artist.update_search(&mut model.matcher, &model.aliases);
    artist.albums[0].expanded = false;
    artist.set_selected(Some(0));
    model.title_alias_store =
        Box::new(AliasStore(aliases(&[("Artist B/Beta2.flac", "Needle")])));
    model.reload_aliases().unwrap();
    let artist = &model.library.artists.items[1];
    assert_eq!(artist.search.cache.query, "Needle");
    assert_eq!(artist.search.cache.order.len(), 1);
    assert_eq!(artist.search.cache.indices.len(), 1);
    assert!(!artist.albums[0].expanded);
    assert_eq!(artist.selected(), Some(0));
}

#[test]
fn changed_alias_ranking_keeps_queue_selection_by_id() {
    let mut model = model();
    model.title_alias_store = Box::new(AliasStore(aliases(&[
        ("Artist A/Other.flac", "xNeedle"),
        ("Artist A/Another.flac", "Needle"),
    ])));
    model.reload_aliases().unwrap();
    let mut first = song("Artist A", "Other");
    first.place = Some(mpd::song::QueuePlace {
        id: mpd::Id(100),
        pos: 0,
        prio: 0,
    });
    let mut second = song("Artist A", "Another");
    second.place = Some(mpd::song::QueuePlace {
        id: mpd::Id(101),
        pos: 1,
        prio: 0,
    });
    model.queue.songs.filter.set_on();
    model.queue.songs.filter.query = "Needle".into();
    model
        .queue
        .replace_songs(vec![first, second], &mut model.matcher, 30);
    model.queue.set_selected(Some(0));
    assert_eq!(model.queue.selected_id(), Some(mpd::Id(101)));
    model.title_alias_store = Box::new(AliasStore(aliases(&[
        ("Artist A/Other.flac", "Needle"),
        ("Artist A/Another.flac", "xNeedle"),
    ])));
    model.reload_aliases().unwrap();
    assert_eq!(model.queue.id_at(0), Some(mpd::Id(100)));
    assert_eq!(model.queue.selected_id(), Some(mpd::Id(101)));
    assert_eq!(model.queue.selected(), Some(1));
}

#[test]
fn a_large_query_keeps_completed_results_until_the_worker_finishes() {
    let mut model = model();
    model.screen = Screen::Queue;
    let songs = (0..matching::SYNC_LIMIT + 1)
        .map(|index| song("Artist A", &format!("Beta {index}")))
        .collect();
    model.queue.songs.filter.set_on();
    model.queue.songs.filter.query = "Beta".into();
    model.queue.replace_songs(songs, &mut model.matcher, 30);
    model.state = model::State::Searching;
    update::handle_key(
        &mut model,
        ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Char('x'),
            ratatui::crossterm::event::KeyModifiers::empty(),
        ),
    );
    assert_eq!(model.queue.songs.filter.cache.query, "Beta");
    assert_eq!(model.queue.songs.filter.query, "Betax");
    assert_eq!(model.queue.display_len(), matching::SYNC_LIMIT + 1);
    let job = model.pending_search().unwrap();
    let matches = matching::rank(
        &job.query,
        &job.haystacks,
        &mut model.matcher,
        job.top_k,
        || false,
    )
    .unwrap();
    assert!(model.apply_match(matching::Matched {
        target: job.target,
        query: job.query,
        haystacks: job.haystacks,
        matches,
    }));
    assert_eq!(model.queue.songs.filter.cache.query, "Betax");
    assert!(model.queue.songs.filter.cache.order.is_empty());
    assert_eq!(model.queue.display_len(), matching::SYNC_LIMIT + 1);
}

#[test]
fn a_late_duplicate_match_does_not_reopen_manual_folds() {
    let mut model = model();
    artists(&mut model);
    let songs = (0..matching::SYNC_LIMIT + 1)
        .map(|index| song("Artist B", &format!("Beta {index}")))
        .collect();
    apply_tracks(&mut model, "Artist B", songs);
    model.library.set_selected(Some(1));
    model.library.active = model::LibActiveSelector::TrackSelector;
    let artist = &mut model.library.artists.items[1];
    artist.search.set_on();
    artist.search.query = "Bet".into();
    artist.update_search_query(&mut model.matcher, &model.aliases);
    let job = model.pending_search().unwrap();
    let result = |matcher: &mut nucleo_matcher::Matcher| matching::Matched {
        target: job.target.clone(),
        query: job.query.clone(),
        haystacks: job.haystacks.clone(),
        matches: matching::rank(
            &job.query,
            &job.haystacks,
            matcher,
            job.top_k,
            || false,
        )
        .unwrap(),
    };
    let first = result(&mut model.matcher);
    assert!(model.apply_match(first));
    let artist = &mut model.library.artists.items[1];
    artist.albums[0].expanded = false;
    artist.set_selected(Some(0));
    let duplicate = result(&mut model.matcher);
    assert!(!model.apply_match(duplicate));
    assert!(!model.library.artists.items[1].albums[0].expanded);
    assert_eq!(model.library.artists.items[1].selected(), Some(0));
}
