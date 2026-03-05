use super::proto::*;
use super::*;

impl LibraryState {
    pub fn new() -> Self {
        Self {
            artist_search: super::Filter::new(),
            global_search: GlobalSearchState {
                contents: None,
                results_state: ListState::default(),
                search: Filter::new(),
            },
            active: super::LibActiveSelector::ArtistSelector,
            contents: Vec::new(),
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
        if self.should_filter() {
            self.filter()
                .cache
                .order
                .iter()
                .filter(|i| i.is_some())
                .count()
        } else {
            self.contents.len()
        }
    }
}

impl Searchable<ArtistData> for LibraryState {
    fn filter(&self) -> &Filter {
        &self.artist_search
    }
    fn filter_mut(&mut self) -> &mut Filter {
        &mut self.artist_search
    }
    fn build_utfstrings_cache(&self) -> Option<Vec<Utf32String>> {
        Some(
            self.contents
                .iter()
                .map(|i| Utf32String::from(i.to_fuzzy_find_str()))
                .collect(),
        )
    }
    fn contents(&self) -> Box<dyn Iterator<Item = &ArtistData> + '_> {
        if self.should_filter() {
            Box::new(
                self.filter()
                    .cache
                    .order
                    .iter()
                    .filter_map(|idx| idx.and_then(|i| self.contents.get(i))),
            )
        } else {
            Box::new(self.contents.iter())
        }
    }
    fn selected_item_mut(&mut self) -> Option<&mut ArtistData> {
        if self.should_filter() {
            self.selector().selected().and_then(|i| {
                self.artist_search
                    .cache
                    .order
                    .get(i)
                    .cloned()
                    .flatten()
                    .and_then(|j| self.contents.get_mut(j))
            })
        } else {
            self.selector()
                .selected()
                .and_then(|i| self.contents.get_mut(i))
        }
    }
}
