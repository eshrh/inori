use super::*;
use crate::event_handler::Result;
use crate::model::ItemRef::*;
use crate::model::LibActiveSelector::*;
use mpd::Query;
use mpd::Term;
use std::borrow::Cow::Borrowed;

pub fn handle_library(model: &mut Model, msg: Message) -> Result<Update> {
    match msg {
        Message::LocalSearch(SearchMsg::Start) => {
            match model.library.active {
                ArtistSelector => {
                    model.library.artist_search.set_on();
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
            Ok(Update::empty())
        }
        Message::LocalSearch(SearchMsg::End) => {
            model.state = State::Running;
            if model.library.global_search.search.active {
                model.library.global_search.search.set_off();
            }
            Ok(Update::empty())
        }
        Message::GlobalSearch(SearchMsg::Start) => {
            model.state = State::Searching;
            model.library.artist_search.set_off();
            model.library.global_search.search.set_on();
            if model.library.global_search.contents.is_none() {
                model.update_global_search_contents()?;
            }
            Ok(Update::empty())
        }
        Message::Escape => {
            match model.library.active {
                ArtistSelector => model.library.artist_search.set_off(),
                TrackSelector => {
                    if let Some(a) = model.library.selected_item_mut() {
                        a.search.set_off();
                        a.expand_all();
                    }
                }
            };
            Ok(Update::empty())
        }
        Message::Tab => {
            model.screen = Screen::Queue;
            Ok(Update::empty())
        }
        other => match model.library.active {
            ArtistSelector => handle_library_artist(model, other),
            TrackSelector => handle_library_track(model, other),
        },
    }
}

pub fn handle_search(model: &mut Model, k: KeyEvent) -> Result<Update> {
    match (
        &model.library.active,
        model.library.global_search.search.active,
    ) {
        (_, true) => {
            if let Some(m) = handle_search_k(
                &mut model.library.global_search,
                k,
                &mut model.matcher,
            ) {
                handle_msg(model, m)
            } else {
                if let Some(item) = model.library.global_search.selected_item()
                {
                    model.jump_to(item.clone());
                }
                Ok(Update::empty())
            }
        }
        (ArtistSelector, _) => {
            if let Some(m) =
                handle_search_k(&mut model.library, k, &mut model.matcher)
            {
                handle_msg(model, m)
            } else {
                Ok(Update::empty())
            }
        }
        (TrackSelector, _) => {
            if let Some(artist) = model.library.selected_item_mut() {
                let msg =
                    handle_search_k_tracksel(artist, k, &mut model.matcher);
                if let Some(m) = msg {
                    handle_msg(model, m)
                } else {
                    Ok(Update::empty())
                }
            } else {
                Ok(Update::empty())
            }
        }
    }
}

pub fn handle_library_artist(
    model: &mut Model,
    msg: Message,
) -> Result<Update> {
    match msg {
        Message::Direction(Dirs::Vert(d)) => {
            handle_vertical(d, &mut model.library);
            Ok(Update::CURRENT_ARTIST)
        }
        Message::Direction(Dirs::Horiz(Horizontal::Right)) => {
            model.library.active = TrackSelector;
            model
                .library
                .selected_item_mut()
                .and_then(|f| Some(f.init()));
            Ok(Update::empty())
        }
        Message::Enter => {
            if let Some(artist) = model.library.selected_item() {
                model.conn.findadd(Query::new().and(
                    Term::Tag(Borrowed("AlbumArtist")),
                    artist.name.clone(),
                ))?;
            }
            Ok(Update::STATUS | Update::QUEUE | Update::START_PLAYING)
        }
        _ => Ok(Update::empty()),
    }
}

pub fn add_item(model: &mut Model) -> Result<Update> {
    if let Some(artist) = model.library.selected_item_mut() {
        match artist.selected_item().map(|i| i.item) {
            Some(Album(album)) => model.conn.findadd(
                Query::new()
                    .and(
                        Term::Tag(Borrowed("AlbumArtist")),
                        artist.name.clone(),
                    )
                    .and(Term::Tag(Borrowed("Album")), album.name.clone()),
            )?,
            Some(Song(song)) => model
                .conn
                .findadd(Query::new().and(Term::File, song.file.clone()))?,
            None => {}
        }
    }
    Ok(Update::STATUS | Update::QUEUE | Update::START_PLAYING)
}

pub fn handle_library_track(model: &mut Model, msg: Message) -> Result<Update> {
    match msg {
        Message::Direction(Dirs::Vert(d)) => {
            if let Some(art) = model.library.selected_item_mut() {
                handle_vertical(d, art);
            }
            Ok(Update::empty())
        }
        Message::Direction(Dirs::Horiz(Horizontal::Left)) => {
            model.library.active = ArtistSelector;
            Ok(Update::empty())
        }
        Message::Enter => add_item(model),
        Message::Fold | Message::Direction(Dirs::Horiz(Horizontal::Right)) => {
            if let Some(art) = model.library.selected_item_mut() {
                if let Some(album) = art.selected_album_mut() {
                    album.expanded = !album.expanded;
                } else {
                    if let Some(idx) = art.selected() {
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
            }
            Ok(Update::empty())
        }
        _ => Ok(Update::empty()),
    }
}
