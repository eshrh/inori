extern crate mpd;
use crate::event_handler::Result;
use crate::model::proto::*;
use crate::model::{AlbumData, ArtistData, Model};
use itertools::Itertools;
use mpd::{Query, Term};
use std::borrow::Cow::Borrowed;

pub fn build_library(model: &mut Model) -> Result<()> {
    let artists = model
        .conn
        .list_group_2(("albumartistsort".into(), "albumartist".into()))?;

    for chunk in artists.chunk_by(|_a, b| b.0 == "AlbumArtistSort") {
        if let Some(albumartist) = chunk.first().map(|i| i.1.clone()) {
            model.library.contents.push(ArtistData::from_names(
                albumartist,
                chunk.iter().skip(1).map(|i| i.1.clone()).collect(),
            ));
        }
    }
    model.library.contents.sort_by(|a, b| {
        let a_name = a.sort_names.first().unwrap_or(&a.name);
        let b_name = b.sort_names.first().unwrap_or(&a.name);
        a_name.to_lowercase().cmp(&b_name.to_lowercase())
    });
    Ok(())
}

pub fn add_tracks(model: &mut Model) -> Result<()> {
    let song_data = model.conn.find(
        Query::new().and(
            Term::Tag(Borrowed("AlbumArtist")),
            match model.library.selected_item_mut() {
                Some(a) => a.name.clone(),
                None => return Ok(()),
            },
        ),
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
                    .and_then(|i| Some(i.1))
                    .unwrap_or("<ALBUM NOT FOUND>".into()),
                tracks: album.iter().cloned().collect(),
                expanded: true,
            });
        }
    }
    if let Some(states) = model
        .library
        .selected_item()
        .map(|item| item.albums.iter().map(|i| i.expanded).collect_vec())
    {
        if states.len() == albums.len() {
            for (i, prev) in albums.iter_mut().zip(states) {
                i.expanded = prev;
            }
        }
    }
    model.library.selected_item_mut().map(|i| {
        i.albums = albums;
        i.fetched = true;
    });
    Ok(())
}
