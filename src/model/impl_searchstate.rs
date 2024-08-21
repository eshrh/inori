use super::*;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use nucleo_matcher::{Utf32Str, Utf32String};
use selector_state::*;

impl FilterCache {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            order: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl Filter {
    pub fn new() -> Self {
        Self {
            active: false,
            query: String::new(),
            cache: FilterCache::new(),
        }
    }
}

impl Selector for GlobalSearchState {
    fn selector(&self) -> &impl SelectorState {
        &self.results_state
    }
    fn selector_mut(&mut self) -> &mut impl SelectorState {
        &mut self.results_state
    }
    fn len(&self) -> usize {
        if self.filter().active {
            self.filter().cache.order.len()
        } else {
            match &self.contents {
                Some(v) => v.len(),
                None => 0,
            }
        }
    }
}

impl Searchable<Vec<String>> for GlobalSearchState {
    fn filter(&self) -> &Filter {
        &self.search
    }
    fn filter_mut(&mut self) -> &mut Filter {
        &mut self.search
    }
    fn contents(&self) -> Box<dyn Iterator<Item = &Vec<String>> + '_> {
        match &self.contents {
            Some(c) => {
                if self.filter().active {
                    Box::new(
                        self.search
                            .cache
                            .order
                            .iter()
                            .filter_map(|idx| idx.map(|i| &c[i])),
                    )
                } else {
                    Box::new(c.iter())
                }
            }
            None => Box::new(std::iter::empty()),
        }
    }
    fn selected_item_mut(&mut self) -> Option<&mut Vec<String>> {
        // if self.filter().active {
        //     self.selector().selected().and_then(|i| {
        //         self.search.cache.order[i]
        //             .and_then(|j| self.contents_get_mut(j))
        //     })
        // } else {
        //     self.selector()
        //         .selected()
        //         .and_then(|i| self.contents_get_mut(i))
        // }
        unimplemented!();
    }
    fn update_filter_cache(&mut self) {
        if self.search.query == self.search.cache.query {
            return;
        }
        if self.contents.is_none() {
            self.search.cache.order = Vec::new();
            self.search.cache.indices = Vec::new();
            return;
        }
        let pattern = Pattern::new(
            self.search.query.as_str(),
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );
        let scores = self
            .contents
            .as_ref()
            .unwrap()
            .iter()
            .map(|i| {
                pattern.score(
                    Utf32Str::new(&i.join("/"), &mut Vec::new()),
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
        self.filter_mut().cache.query = self.filter().query.clone();
        self.filter_mut().cache.order = order;
    }
}
