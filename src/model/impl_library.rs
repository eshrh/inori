use super::selector_state::*;
use super::*;
use crate::model::TrackSelItem;
use crate::update::build_library;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use nucleo_matcher::{Utf32Str, Utf32String};

impl LibraryState {
    pub fn new() -> Self {
        Self {
            artist_search: super::Filter::new(),
            global_search: GlobalSearchState {
                contents: None,
                results_state: ListState::default(),
                search: Filter::new(),
                matcher: Matcher::new(Config::DEFAULT),
            },
            active: super::LibActiveSelector::ArtistSelector,
            contents: Vec::new(),
            artist_state: ListState::default(),
            matcher: {
                let mut default_config = Config::DEFAULT;
                default_config.prefer_prefix = true;
                Matcher::new(default_config)
            },
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
        if self.filter().active {
            Box::new(
                self.artist_search
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
        if self.filter().active {
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
    fn update_filter_cache(&mut self) {
        if self.filter().cache.query != self.artist_search.query {
            let pattern = Pattern::new(
                self.artist_search.query.as_str(),
                CaseMatching::Ignore,
                Normalization::Smart,
                AtomKind::Fuzzy,
            );

            let scores = self
                .contents
                .iter()
                .map(|i| {
                    pattern.score(
                        Utf32Str::new(&i.to_fuzzy_find_str(), &mut Vec::new()),
                        &mut self.matcher,
                    )
                })
                .collect::<Vec<Option<u32>>>();
            let mut order = scores
                .into_iter()
                .enumerate()
                .collect::<Vec<(usize, Option<u32>)>>();
            order.sort_by(|a, b| b.1.unwrap_or(0).cmp(&a.1.unwrap_or(0)));
            // include the index only if the score is Some(nonzero)
            let order = order
                .iter()
                .map(|i| {
                    if i.1.is_some_and(|score| score > 0) {
                        Some(i.0)
                    } else {
                        None
                    }
                })
                .collect::<Vec<Option<usize>>>();

            // generate indices:
            let mut indices: Vec<Option<Vec<u32>>> = Vec::new();
            for idx in order.iter().take_while(|i| i.is_some()) {
                let mut tmp: Vec<u32> = Vec::new();
                pattern.indices(
                    Utf32Str::new(
                        &self.contents[idx.unwrap()].to_fuzzy_find_str(),
                        &mut Vec::new(),
                    ),
                    &mut self.matcher,
                    &mut tmp,
                );
                indices.push(Some(tmp));
            }
            self.filter_mut().cache.query = self.filter().query.clone();
            self.filter_mut().cache.order = order;
            self.filter_mut().cache.indices = indices;
        }
    }
}
