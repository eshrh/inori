extern crate mpd;
use crate::event_handler::Result;
use crate::model::*;
use mpd::{Query, Term};
use std::borrow::Cow::Borrowed;
use std::collections::BTreeMap;

pub fn build_library(model: &mut Model) -> Result<()> {
    let artists = model
        .conn
        .list_group_2(("albumartistsort".into(), "albumartist".into()))?;
    for chunk in artists.chunk_by(|a, _b| a.0 == "AlbumArtist") {
        let albumartist = chunk[0].1.clone();
        if !model.library.contents.contains_key(&albumartist) {
            model.library.contents.insert(
                albumartist.clone(),
                ArtistData {
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
    Ok(())
}

pub fn get_tracks(
    model: &mut Model,
    artist_name: &String,
) -> Result<Option<BTreeMap<String, AlbumData>>> {
    let song_data = model.conn.find(
        Query::new().and(Term::Tag(Borrowed("Artist")), artist_name),
        None,
    )?;

    let mut albums: BTreeMap<String, AlbumData> = BTreeMap::new();
    for song in song_data {
        let album_name =
            song.tags.iter().find(|t| t.0 == "Album").unwrap().clone().1;
        match albums.get_mut(&album_name) {
            None => {
                let _ =
                    albums.insert(album_name, AlbumData { tracks: vec![song] });
            }
            Some(v) => v.tracks.push(song),
        }
    }
    Ok(Some(albums))
}
