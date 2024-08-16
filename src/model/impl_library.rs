use super::selector_state::*;
use super::*;
use crate::model::TrackSelItem;

impl LibraryState {
    pub fn new() -> Self {
        Self {
            search: super::SearchState::new(),
            active: super::LibActiveSelector::ArtistSelector,
            contents: Vec::new(),
            artist_state: ListState::default(),
        }
    }
    pub fn selected_track(&self) -> Option<TrackSelItem> {
        self.selected_item()?.selected_item()
    }
}

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
