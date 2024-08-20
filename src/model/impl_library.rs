use std::iter::zip;

use super::selector_state::*;
use super::*;
use crate::model::TrackSelItem;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use nucleo_matcher::{Utf32Str, Utf32String};

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
    pub fn get_ordering(&self) -> Vec<Option<usize>> {
        let mut matcher = Matcher::new(Config::DEFAULT);
        let pattern = Pattern::new(
            self.search.query.as_str(),
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );
        let scores = self
            .contents
            .iter()
            .map(|i| {
                pattern.score(
                    Utf32Str::new(&i.name, &mut Vec::new()),
                    &mut matcher,
                )
            })
            .collect::<Vec<Option<u32>>>();
        let mut order_iter = scores
            .into_iter()
            .enumerate()
            .collect::<Vec<(usize, Option<u32>)>>();
        order_iter.sort_by(|a, b| b.1.unwrap_or(0).cmp(&a.1.unwrap_or(0)));

        // include the index only if the score is Some(nonzero)
        order_iter
            .iter()
            .map(|i| {
                if i.1.is_some_and(|score| score > 0) {
                    Some(i.0)
                } else {
                    None
                }
            })
            .collect()
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
    fn contents(&self) -> Box<dyn Iterator<Item = &ArtistData> + '_> {
        if self.filter().active {
            let order_iter = self.get_ordering();
            Box::new(
                order_iter
                    .into_iter()
                    .filter_map(|idx| idx.map(|i| &self.contents[i])),
            )
        } else {
            Box::new(self.contents.iter())
        }
    }
    fn selected_item_mut(&mut self) -> Option<&mut ArtistData> {
        if self.filter().active {
            let order_iter = self.get_ordering();
            self.selector().selected().and_then(|i| {
                order_iter[i].and_then(|j| self.contents.get_mut(j))
            })
        } else {
            self.selector()
                .selected()
                .and_then(|i| self.contents.get_mut(i))
        }
    }
}
