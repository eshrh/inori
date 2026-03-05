extern crate mpd;
use super::search_utils::{compute_indices, compute_orders};
use super::*;

pub trait SelectorState {
    fn selected(&self) -> Option<usize>;
    fn set_selected(&mut self, s: Option<usize>);
    fn offset(&self) -> usize;
    fn set_offset(&mut self, s: usize);
}

impl SelectorState for ListState {
    fn selected(&self) -> Option<usize> {
        self.selected()
    }
    fn offset(&self) -> usize {
        self.offset()
    }
    fn set_selected(&mut self, s: Option<usize>) {
        *self.selected_mut() = s;
    }
    fn set_offset(&mut self, s: usize) {
        *self.offset_mut() = s;
    }
}

impl SelectorState for TableState {
    fn selected(&self) -> Option<usize> {
        self.selected()
    }
    fn offset(&self) -> usize {
        self.offset()
    }
    fn set_selected(&mut self, s: Option<usize>) {
        *self.selected_mut() = s;
    }
    fn set_offset(&mut self, s: usize) {
        *self.offset_mut() = s;
    }
}

pub trait Selector {
    fn selector(&self) -> &impl SelectorState;
    fn selector_mut(&mut self) -> &mut impl SelectorState;
    fn len(&self) -> usize;

    fn selected(&self) -> Option<usize> {
        self.selector().selected()
    }
    fn offset(&self) -> usize {
        self.selector().offset()
    }
    fn set_selected(&mut self, val: Option<usize>) {
        self.selector_mut().set_selected(val);
    }
    fn set_offset(&mut self, val: usize) {
        self.selector_mut().set_offset(val);
    }
    fn init(&mut self) {
        // idempotent
        if self.len() != 0 && self.selected().is_none() {
            self.set_selected(Some(0));
        }
    }
    fn watch_oob(&mut self) {
        if self.len() == 0 || self.selected().is_some_and(|f| f >= self.len()) {
            self.set_selected(None)
        }
    }
}

pub trait Searchable<T>: Selector {
    fn filter(&self) -> &Filter;
    fn filter_mut(&mut self) -> &mut Filter;
    fn build_utfstrings_cache(&self) -> Option<Vec<Utf32String>>;
    fn contents(&self) -> Box<dyn Iterator<Item = &T> + '_>;
    fn selected_item_mut(&mut self) -> Option<&mut T>;
    fn update_filter_cache(
        &mut self,
        matcher: &mut Matcher,
        top_k: Option<usize>,
    ) {
        if self.filter().cache.query == self.filter().query {
            return;
        }

        if self.filter().cache.utfstrings_cache.is_none() {
            let Some(cache) = self.build_utfstrings_cache() else {
                self.filter_mut().cache.order.clear();
                self.filter_mut().cache.indices.clear();
                return;
            };
            self.filter_mut().cache.utfstrings_cache = Some(cache);
        }

        let query = self.filter().query.clone();
        let order = {
            let Some(cache) = self.filter().cache.utfstrings_cache.as_ref()
            else {
                return;
            };
            compute_orders(&query, cache, matcher, 0)
        };
        let strings: Vec<&Utf32String> = {
            let Some(cache) = self.filter().cache.utfstrings_cache.as_ref()
            else {
                return;
            };
            let strings_iterator = order
                .iter()
                .take_while(|i| i.is_some())
                .filter_map(|i| i.and_then(|idx| cache.get(idx)));
            if let Some(k) = top_k {
                strings_iterator.take(k).collect()
            } else {
                strings_iterator.collect()
            }
        };
        let indices = compute_indices(&query, strings, matcher);

        self.filter_mut().cache.query = query;
        self.filter_mut().cache.order = order;
        self.filter_mut().cache.indices = indices;
    }
    fn selected_item(&self) -> Option<&T> {
        self.selector()
            .selected()
            .and_then(|i| self.contents().nth(i))
    }
    fn contents_vec(&self) -> Vec<&T> {
        self.contents().collect()
    }
    fn should_filter(&self) -> bool {
        self.filter().active
            && self.filter().cache.order.iter().any(|i| i.is_some())
            && !self.filter().query.is_empty()
    }
}
