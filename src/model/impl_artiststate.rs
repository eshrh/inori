use super::*;
use selector_state::*;
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
    pub fn selected_album_mut(&mut self) -> Option<&mut AlbumData> {
        // assumption: order in self.albums is the same as in the viewer.
        // NOTE: can't use TrackSelItem enum since references are immutable.
        // Tried this and it's busted.
        let sel_idx = self.selector().selected()?;
        let mut i = 0;
        let mut album_i = 0;
        for album in &self.albums {
            if sel_idx == i {
                return Some(&mut self.albums[album_i]);
            }
            album_i += 1;
            i += 1;
            if album.expanded {
                i += album.tracks.len()
            }
        }
        None
    }
}

impl ArtistData {
    pub fn from_names(name: String, sort_names: Vec<String>) -> Self {
        Self {
            name,
            fetched: false,
            albums: vec![],
            sort_names,
            track_sel_state: ListState::default(),
        }
    }
}

impl<'a> ToString for TrackSelItem<'a> {
    fn to_string(&self) -> String {
        match self {
            TrackSelItem::Album(a) => a.name.clone(),
            TrackSelItem::Song(s) => s.title.clone().unwrap(),
        }
    }
}
