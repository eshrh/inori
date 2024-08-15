use super::selector_state::*;
use super::{AlbumData, ArtistData, LibraryState};
use crate::model::TrackSelItem;

impl Selector for LibraryState {
    fn selector(&self) -> &impl SelectorState {
        &self.artist_state
    }
    fn selector_mut(&mut self) -> &mut impl SelectorState {
        &mut self.artist_state
    }
    fn len(&self) -> usize {
        self.contents.len()
    }
}

impl SelectorWithContents<ArtistData> for LibraryState {
    fn contents(&self) -> &Vec<ArtistData> {
        &self.contents
    }
    fn contents_mut(&mut self) -> &mut Vec<ArtistData> {
        &mut self.contents
    }
}

impl Selector for ArtistData {
    fn selector(&self) -> &impl SelectorState {
        &self.track_sel_state
    }
    fn selector_mut(&mut self) -> &mut impl SelectorState {
        &mut self.track_sel_state
    }
    fn len(&self) -> usize {
        self.albums.len()
            + self
                .albums
                .iter()
                .map(|i| if i.expanded { i.tracks.len() } else { 0 })
                .sum::<usize>()
    }
}

impl<'a> ArtistData {
    pub fn contents(&'a self) -> Vec<TrackSelItem> {
        let mut new: Vec<TrackSelItem> = vec![];
        for album in &self.albums {
            new.push(TrackSelItem::Album(album));
            if album.expanded {
                for track in &album.tracks {
                    new.push(TrackSelItem::Song(track))
                }
            }
        }
        new
    }
    pub fn selected_item(&self) -> Option<TrackSelItem> {
        // TODO: refactor with performant fp
        let sel_idx = self.selector().selected()?;
        let mut i = 0;
        for album in &self.albums {
            if sel_idx == i {
                return Some(TrackSelItem::Album(album));
            }
            i += 1;
            if album.expanded {
                let al_len = album.tracks.len();
                if (sel_idx - i) < al_len {
                    return Some(TrackSelItem::Song(
                        &album.tracks[sel_idx - i],
                    ));
                }
                i += al_len;
            }
        }
        None
    }
    // pub fn test_sel_item(&mut self, idx: usize) {
    //     self.set_selected(Some(idx));
    //     dbg!(self.selected_item().unwrap().to_string());
    // }
}

impl<'a> ToString for TrackSelItem<'a> {
    fn to_string(&self) -> String {
        match self {
            TrackSelItem::Album(a) => a.name.clone(),
            TrackSelItem::Song(s) => s.title.clone().unwrap(),
        }
    }
}
