use std::error::Error;

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

impl From<&mut Vec<String>> for InfoEntry {
    fn from(v: &mut Vec<String>) -> Self {
        if v.len() > 4 {
            panic!("too much info given to infoentry");
        } else {
            let mut drained = v.drain(..);
            InfoEntry {
                artist: drained.nth(0).unwrap(),
                artist_sort: drained.nth(0),
                album: drained.nth(0),
                title: drained.nth(0),
            }
        }
    }
}

impl InfoEntry {
    pub fn is_redundant(&self) -> bool {
        self.album.is_none()
            && self.title.is_none()
            && self.artist_sort.as_ref().is_some_and(|i| *i == self.artist)
    }
}

impl ToString for InfoEntry {
    fn to_string(&self) -> String {
        let mut out: String = self.artist.clone();
        if let Some(artist_sort) = &self.artist_sort {
            if *artist_sort != out {
                out.push('/');
                out.push_str(artist_sort);
            }
        }
        if let Some(album) = &self.album {
            out.push('/');
            out.push_str(album);
        }
        if let Some(title) = &self.title {
            out.push('/');
            out.push_str(title);
        }
        out
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

impl Searchable<InfoEntry> for GlobalSearchState {
    fn filter(&self) -> &Filter {
        &self.search
    }
    fn filter_mut(&mut self) -> &mut Filter {
        &mut self.search
    }
    fn contents(&self) -> Box<dyn Iterator<Item = &InfoEntry> + '_> {
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
    fn selected_item_mut(&mut self) -> Option<&mut InfoEntry> {
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
                    Utf32Str::new(&i.to_string(), &mut Vec::new()),
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
