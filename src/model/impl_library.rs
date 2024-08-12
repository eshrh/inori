use super::selector_state::*;
use super::{ArtistData, LibraryState};

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
