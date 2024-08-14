extern crate mpd;
use crate::event_handler::Result;
use crate::model::selector_state::Selector;
use crate::model::{AlbumData, ArtistData, Model};
use mpd::{Query, Song, Term};
use ratatui::widgets::ListState;
use std::borrow::Cow::Borrowed;

pub fn build_library(model: &mut Model) -> Result<()> {
    let artists = model
        .conn
        .list_group_2(("albumartistsort".into(), "albumartist".into()))?;

    for chunk in artists.chunk_by(|_a, b| b.0 == "AlbumArtistSort") {
        let albumartist = chunk[0].1.clone();

        model.library.contents.push(ArtistData {
            name: albumartist.clone(),
            fetched: false,
            albums: vec![],
            sort_names: chunk.iter().skip(1).map(|i| i.1.clone()).collect(),
            track_sel_state: ListState::default()
        });
    }
    model.library.contents.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(())
}

pub fn add_tracks(model: &mut Model) -> Result<()> {
    let song_data = model.conn.find(
        Query::new().and(
            Term::Tag(Borrowed("AlbumArtist")),
            model.library.selected_item_mut().unwrap().name.clone(),
        ),
        None,
    )?;
    let mut albums: Vec<AlbumData> = vec![];
    for album in song_data.chunk_by(|a, b| {
        a.tags.iter().find(|t| t.0 == "Album")
            == b.tags.iter().find(|t| t.0 == "Album")
    }) {
        albums.push(AlbumData {
            name: album
                .first()
                .unwrap()
                .tags
                .iter()
                .find(|t| t.0 == "Album")
                .unwrap()
                .clone()
                .1,
            tracks: album.iter().cloned().collect(),
        });
    }

    model.library.selected_item_mut().unwrap().albums = albums;
    model.library.selected_item_mut().unwrap().fetched = true;
    Ok(())
}
