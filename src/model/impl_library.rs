use super::proto::*;
use super::*;

impl LibraryState {
    pub fn new() -> Self {
        Self {
            artists: FilteredView::new(),
            global_search: GlobalSearchState {
                entries: FilteredView::new(),
                loaded: false,
                results_state: ListState::default(),
            },
            active: super::LibActiveSelector::ArtistSelector,
            artist_state: ListState::default(),
        }
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
        self.display_len()
    }
}

impl Searchable<ArtistData> for LibraryState {
    fn filter(&self) -> &Filter {
        &self.artists.filter
    }
    fn filter_mut(&mut self) -> &mut Filter {
        &mut self.artists.filter
    }
    fn build_utfstrings_cache(&self) -> Option<Vec<Utf32String>> {
        Some(
            self.artists
                .items
                .iter()
                .map(|i| Utf32String::from(i.to_fuzzy_find_str()))
                .collect(),
        )
    }
    fn storage_len(&self) -> usize {
        self.artists.items.len()
    }
    fn storage_get(&self, idx: usize) -> Option<&ArtistData> {
        self.artists.items.get(idx)
    }
    fn storage_get_mut(&mut self, idx: usize) -> Option<&mut ArtistData> {
        self.artists.items.get_mut(idx)
    }
}
