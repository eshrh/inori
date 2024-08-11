extern crate mpd;
use crate::event_handler::Result;
use crate::model::*;
use mpd::{Query, Term};
use std::borrow::Cow::Borrowed;
use std::collections::HashMap;

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
            contents: None,
        });
    }
    model.library.contents.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(())
}

pub fn add_tracks(model: &mut Model, artist_id: usize) -> Result<()> {
    let song_data = model.conn.find(
        Query::new().and(
            Term::Tag(Borrowed("AlbumArtist")),
            model.library.contents.get(artist_id).unwrap().name.clone(),
        ),
        None,
    )?;
    let mut albums: HashMap<String, AlbumData> = HashMap::new();
    for song in song_data {
        let album_name =
            song.tags.iter().find(|t| t.0 == "Album").unwrap().clone().1;
        match albums.get_mut(&album_name) {
            None => {
                albums.insert(album_name, AlbumData { tracks: vec![song] });
            }
            Some(v) => v.tracks.push(song),
        }
    }
    let mut album_names: Vec<String> =
        albums.keys().into_iter().cloned().collect();
    album_names.sort();

    model.library.artist_selected_mut().unwrap().albums = album_names;
    model.library.artist_selected_mut().unwrap().contents = Some(albums);
    model.library.artist_selected_mut().unwrap().fetched = true;
    Ok(())
}
