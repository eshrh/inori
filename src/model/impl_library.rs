use super::selector_state::*;
use super::*;
use crate::model::TrackSelItem;

impl LibraryState {
    pub fn new() -> Self {
        Self {
            search: super::Filter::new(),
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
        self.contents_vec().len()
    }
}

impl Searchable<ArtistData> for LibraryState {
    fn filter(&self) -> &Filter {
        &self.search
    }
    fn filter_mut(&mut self) -> &mut Filter {
        &mut self.search
    }
    fn contents(&self) -> Box<dyn Iterator<Item=&ArtistData> + '_> {
        if self.filter().active {
            Box::new(
                self.contents.iter()
                    .filter(|i| i.name.contains(&self.filter().query))
            )
        } else {
            Box::new(self.contents.iter())
        }
    }
    fn contents_mut(&mut self) -> Box<dyn Iterator<Item=&mut ArtistData> + '_> {
        if self.search.active {
            Box::new(
                self.contents.iter_mut()
                    .filter(|i| i.name.contains(&self.search.query))
            )
        } else {
            Box::new(self.contents.iter_mut())
        }

    }
}
