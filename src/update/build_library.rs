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

    for chunk in artists.chunk_by(|a, _b| a.0 == "AlbumArtist") {
        // println!("{:?}", chunk);
        let albumartist = chunk[0].1.clone();
        if !model.library.contents.contains_key(&albumartist) {
            model.library.contents.insert(
                albumartist.clone(),
                ArtistData {
                    fetched: false,
                    albums: vec![],
                    sort_names: vec![],
                    contents: None,
                },
            );
        }

        for sort_name in chunk.iter().skip(1) {
            model
                .library
                .contents
                .get_mut(&albumartist)
                .unwrap()
                .sort_names
                .push(sort_name.1.clone());
        }
    }
    model.library.artists = model.library.contents.keys().cloned().collect();
    model.library.artists.sort();
    Ok(())
}

pub fn get_tracks(
    model: &mut Model,
    artist_name: &String,
) -> Result<Option<(Vec<String>, HashMap<String, AlbumData>)>> {
    let song_data = model.conn.find(
        Query::new().and(Term::Tag(Borrowed("AlbumArtist")), artist_name),
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
    Ok(Some((album_names, albums)))
}
