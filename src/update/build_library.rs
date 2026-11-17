extern crate mpd;
use crate::event_handler::Result;
use crate::model::proto::*;
use crate::model::{AlbumData, ArtistData, ItemRef, Model};
use mpd::{Query, Term};
use std::borrow::Cow::Borrowed;
use std::collections::HashMap;

enum TrackSelection {
    Album(String),
    Song(String),
}

pub fn build_library(model: &mut Model) -> Result<()> {
    let artists = model
        .conn
        .list_group_2(("albumartistsort".into(), "albumartist".into()))?;

    let selected_name = model.library.selected_item().map(|a| a.name.clone());
    let mut previous: HashMap<_, _> =
        std::mem::take(&mut model.library.artists.items)
            .into_iter()
            .map(|artist| (artist.name.clone(), artist))
            .collect();
    for chunk in artists.chunk_by(|_a, b| b.0 == "AlbumArtistSort") {
        if let Some(albumartist) = chunk.first().map(|i| i.1.clone()) {
            let sort_names =
                chunk.iter().skip(1).map(|i| i.1.clone()).collect();
            let mut artist = previous
                .remove(&albumartist)
                .unwrap_or_else(|| ArtistData::from_names(albumartist, vec![]));
            artist.sort_names = sort_names;
            artist.fetched = false;
            model.library.artists.items.push(artist);
        }
    }
    // sort by sort name
    model.library.artists.items.sort_by(|a, b| {
        let a_name = a.sort_names.first().unwrap_or(&a.name);
        let b_name = b.sort_names.first().unwrap_or(&b.name);
        a_name.to_lowercase().cmp(&b_name.to_lowercase())
    });
    model.library.artists.items.shrink_to_fit();
    model.library.artists.filter.reset_cache();
    model.library.update_filter_cache(
        &mut model.matcher,
        Some(model.frame_size.height.into()),
    );
    let selected = selected_name.and_then(|name| {
        let storage_idx = model
            .library
            .artists
            .items
            .iter()
            .position(|artist| artist.name == name)?;
        if model.library.should_filter() {
            model
                .library
                .artists
                .filter
                .cache
                .order
                .iter()
                .position(|idx| *idx == Some(storage_idx))
        } else {
            Some(storage_idx)
        }
    });
    model.library.set_selected(selected);
    model.library.init();
    Ok(())
}

pub fn add_tracks(model: &mut Model) -> Result<()> {
    let Some(artist) = model.library.selected_item() else {
        return Ok(());
    };
    if artist.fetched {
        return Ok(());
    }
    let selected = artist.selected_item().map(|item| match item.item {
        ItemRef::Album(album) => TrackSelection::Album(album.name.clone()),
        ItemRef::Song(song) => TrackSelection::Song(song.file.clone()),
    });
    let song_data = model.conn.find(
        Query::new()
            .and(Term::Tag(Borrowed("AlbumArtist")), artist.name.clone()),
        None,
    )?;
    let mut albums: Vec<AlbumData> = Vec::new();

    // chunks have album field invariant!
    for album in song_data.chunk_by(|a, b| {
        a.tags.iter().find(|t| t.0 == "Album")
            == b.tags.iter().find(|t| t.0 == "Album")
    }) {
        if let Some(track) = album.first() {
            albums.push(AlbumData {
                name: track
                    .tags
                    .iter()
                    .find(|t| t.0 == "Album")
                    .cloned()
                    .map(|i| i.1)
                    .unwrap_or("<ALBUM NOT FOUND>".into()),
                alias: track
                    .tags
                    .iter()
                    .find_map(|(k, v)| {
                        (k == "Album")
                            .then(|| model.aliases.album_ref_for_name(v))
                            .flatten()
                    })
                    .map(|r| r.alias.to_string()),
                alias_variants: track
                    .tags
                    .iter()
                    .find_map(|(k, v)| {
                        (k == "Album")
                            .then(|| model.aliases.album_ref_for_name(v))
                            .flatten()
                    })
                    .map_or_else(Vec::new, |r| r.variants.to_vec()),
                tracks: album.to_vec(),
                expanded: true,
            });
        }
    }
    if let Some(item) = model.library.selected_item_mut() {
        let expanded: HashMap<_, _> = item
            .albums
            .iter()
            .map(|album| (album.name.clone(), album.expanded))
            .collect();
        let unchanged = item.albums.len() == albums.len()
            && item.albums.iter().zip(&albums).all(|(a, b)| {
                a.name == b.name
                    && a.alias == b.alias
                    && a.alias_variants == b.alias_variants
                    && a.tracks == b.tracks
            });
        if !unchanged {
            item.albums = albums;
            item.search.reset_cache();
        }
        item.fetched = true;
        if item.search.active {
            item.update_search(&mut model.matcher, &model.aliases);
        }
        for album in &mut item.albums {
            if let Some(state) = expanded.get(&album.name) {
                album.expanded = *state;
            }
        }
        if let Some(selection) = selected {
            let selected = item.contents().iter().position(|entry| {
                match (&selection, &entry.item) {
                    (TrackSelection::Album(name), ItemRef::Album(album)) => {
                        album.name == *name
                    }
                    (TrackSelection::Song(file), ItemRef::Song(song)) => {
                        song.file == *file
                    }
                    _ => false,
                }
            });
            if selected.is_some() {
                item.set_selected(selected);
            }
        }
        item.watch_oob();
        item.init();
    }
    Ok(())
}
