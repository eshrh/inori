use super::*;
use crate::backend::QueueTarget;
use crate::model::ItemRef::*;
use crate::model::LibActiveSelector::*;
use crate::view::layout::library_layout::LibraryLayout;
use crate::view::layout::InoriLayout;

pub fn handle_library(model: &mut Model, msg: Message) -> Option<Action> {
    match msg {
        Message::LocalSearch(SearchMsg::Start) => {
            match model.library.active {
                ArtistSelector => {
                    model.library.artists.filter.set_on();
                    if model.library.len() != 0 {
                        model.library.set_selected(Some(0))
                    }
                }
                TrackSelector => {
                    if let Some(a) = model.library.selected_item_mut() {
                        a.search.set_on();
                    }
                }
            };
            model.state = State::Searching;
            None
        }
        Message::LocalSearch(SearchMsg::End) => {
            model.state = State::Running;
            if model.library.global_search.entries.filter.active {
                model.library.global_search.entries.filter.set_off();
            }
            None
        }
        Message::GlobalSearch(SearchMsg::Start) => {
            model.state = State::Searching;
            model.library.artists.filter.set_off();
            model.library.global_search.entries.filter.set_on();
            None
        }
        Message::Escape => {
            match model.library.active {
                ArtistSelector => model.library.artists.filter.set_off(),
                TrackSelector => {
                    if let Some(a) = model.library.selected_item_mut() {
                        a.search.set_off();
                        a.expand_all();
                    }
                }
            };
            None
        }
        Message::ToggleScreen => {
            handle_library(model, Message::LocalSearch(SearchMsg::End));
            model.screen = Screen::Queue;
            None
        }
        Message::TogglePanel => {
            model.library.active = match model.library.active {
                TrackSelector => ArtistSelector,
                ArtistSelector => TrackSelector,
            };
            None
        }
        other => match model.library.active {
            ArtistSelector => handle_library_artist(model, other),
            TrackSelector => handle_library_track(model, other),
        },
    }
}

pub fn handle_search(model: &mut Model, k: KeyEvent) -> Option<Action> {
    match (
        &model.library.active,
        model.library.global_search.entries.filter.active,
    ) {
        (_, true) => {
            if let Some(m) = handle_search_k(
                &mut model.library.global_search,
                k,
                &mut model.matcher,
                model.frame_size.height.into(),
            ) {
                if matches!(m, Message::Select) {
                    add_search_result(model)
                } else {
                    handle_msg(model, m)
                }
            } else {
                model.library.pending_jump = None;
                model.finish_global_search();
                None
            }
        }
        (ArtistSelector, _) => {
            if let Some(m) = handle_search_k(
                &mut model.library,
                k,
                &mut model.matcher,
                model.frame_size.height.into(),
            ) {
                handle_msg(model, m)
            } else {
                None
            }
        }
        (TrackSelector, _) => {
            if let Some(artist) = model.library.selected_item_mut() {
                let msg = handle_search_k_tracksel(
                    artist,
                    k,
                    &mut model.matcher,
                    &model.aliases,
                );
                if let Some(m) = msg {
                    handle_msg(model, m)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

pub fn handle_library_artist(
    model: &mut Model,
    msg: Message,
) -> Option<Action> {
    match msg {
        Message::Direction(Dirs::Vert(d)) => {
            handle_vertical(d, &mut model.library);
            None
        }
        Message::ScrollScreenful(v) => {
            let k = LibraryLayout::new(model.frame_size, model)
                .artist_select
                .height;
            scroll_screenful(v, k.into(), &mut model.library);
            None
        }
        Message::Direction(Dirs::Horiz(Horizontal::Right)) => {
            model.library.active = TrackSelector;
            if let Some(f) = model.library.selected_item_mut() {
                f.init();
            }
            None
        }
        Message::Select => add_artist(model),
        Message::SelectAndNext => Some(
            add_artist(model)?
                .then(Message::Direction(Dirs::Vert(Vertical::Down))),
        ),

        _ => None,
    }
}

pub fn add_artist(model: &Model) -> Option<Action> {
    let artist = model.library.selected_item()?;
    Some(Action::new(Command::Enqueue(QueueTarget::Artist(
        artist.name.clone(),
    ))))
}

fn add_search_result(model: &Model) -> Option<Action> {
    let entry = model.library.global_search.selected_item()?;
    let target = if !entry.file.is_empty() {
        QueueTarget::Song(entry.file.clone())
    } else if let Some(album) = &entry.album {
        QueueTarget::Album {
            artist: entry.artist.clone(),
            album: album.clone(),
        }
    } else {
        QueueTarget::Artist(entry.artist.clone())
    };
    Some(Action::new(Command::Enqueue(target)))
}

pub fn add_item(model: &Model) -> Option<Action> {
    let artist = model.library.selected_item()?;
    let target = match artist.selected_item()?.item {
        Album(album) => QueueTarget::Album {
            artist: artist.name.clone(),
            album: album.name.clone(),
        },
        Song(song) => QueueTarget::Song(song.file.clone()),
    };
    Some(Action::new(Command::Enqueue(target)))
}

pub fn handle_library_track(model: &mut Model, msg: Message) -> Option<Action> {
    match msg {
        Message::Direction(Dirs::Vert(d)) => {
            if let Some(art) = model.library.selected_item_mut() {
                handle_vertical(d, art);
            }
            None
        }
        Message::ScrollScreenful(v) => {
            let k = LibraryLayout::new(model.frame_size, model)
                .track_select
                .height;
            if let Some(art) = model.library.selected_item_mut() {
                scroll_screenful(v, k.into(), art);
            }
            None
        }
        Message::Direction(Dirs::Horiz(Horizontal::Left)) => {
            model.library.active = ArtistSelector;
            None
        }
        Message::Select => add_item(model),
        Message::SelectAndNext => Some(
            add_item(model)?
                .then(Message::Direction(Dirs::Vert(Vertical::Down))),
        ),
        Message::Fold | Message::Direction(Dirs::Horiz(Horizontal::Right)) => {
            if let Some(art) = model.library.selected_item_mut() {
                if let Some(album) = art.selected_album_mut() {
                    album.expanded = !album.expanded;
                } else if let Some(idx) = art.selected() {
                    for i in (0..idx).rev() {
                        if let Some(Album(_)) =
                            art.contents().get(i).map(|i| &i.item)
                        {
                            art.set_selected(Some(i));
                        }
                        if let Some(album) = art.selected_album_mut() {
                            album.expanded = !album.expanded;
                            break;
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}
