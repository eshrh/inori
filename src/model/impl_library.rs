use super::selector_state::*;
use super::{ArtistData, LibraryState, AlbumData};

impl Selector<ArtistData> for LibraryState {
    fn selector(&self) -> &impl SelectorState {
        &self.artist_state
    }
    fn selector_mut(&mut self) -> &mut impl SelectorState {
        &mut self.artist_state
    }
    fn contents(&self) -> &Vec<ArtistData> {
        &self.contents
    }
    fn contents_mut(&mut self) -> &mut Vec<ArtistData> {
        &mut self.contents
    }
}

impl Selector<AlbumData> for ArtistData {
    fn selector(&self) -> &impl SelectorState {
        &self.track_sel_state
    }
    fn selector_mut(&mut self) -> &mut impl SelectorState {
        &mut self.track_sel_state
    }
    fn contents(&self) -> &Vec<AlbumData> {
        &self.albums
    }
    fn contents_mut(&mut self) -> &mut Vec<AlbumData> {
        &mut self.albums
    }

}
