use super::proto::*;
use super::*;
use crate::model::TrackSelItem;
use nucleo_matcher::Matcher;
use search_utils::{compute_indices, compute_orders};

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
        &self.artist_search
    }
    fn filter_mut(&mut self) -> &mut Filter {
        &mut self.artist_search
    }
    fn contents(&self) -> Box<dyn Iterator<Item = &ArtistData> + '_> {
        if self.should_filter() {
            Box::new(
                self.filter()
                    .cache
                    .order
                    .iter()
                    .filter_map(|idx| idx.map(|i| &self.contents[i])),
            )
        } else {
            Box::new(self.contents.iter())
        }
    }
    fn selected_item_mut(&mut self) -> Option<&mut ArtistData> {
        if self.should_filter() {
            self.selector().selected().and_then(|i| {
                self.artist_search.cache.order[i]
                    .and_then(|j| self.contents.get_mut(j))
            })
        } else {
            self.selector()
                .selected()
                .and_then(|i| self.contents.get_mut(i))
        }
    }
    fn update_filter_cache(&mut self, matcher: &mut Matcher) {
        if self.filter().cache.query == self.artist_search.query {
            return;
        }
        if self.filter().cache.utfstrings_cache.is_none() {
            self.filter_mut().cache.utfstrings_cache = Some(
                self.contents
                    .iter()
                    .map(|i| Utf32String::from(i.to_fuzzy_find_str()))
                    .collect(),
            );
        }
        self.filter_mut().cache.order = compute_orders(
            &self.filter().query,
            self.filter().cache.utfstrings_cache.as_ref().unwrap(),
            matcher,
            0,
        );

        let strings_for_indices: Vec<&Utf32String> = self
            .filter()
            .cache
            .order
            .iter()
            .take_while(|i| i.is_some())
            .map(|i| {
                &self.artist_search.cache.utfstrings_cache.as_ref().unwrap()
                    [i.unwrap()]
            })
            .collect();
        self.filter_mut().cache.indices =
            compute_indices(&self.filter().query, strings_for_indices, matcher);
    }
}
