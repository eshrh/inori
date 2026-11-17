extern crate mpd;
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
    fn storage_len(&self) -> usize;
    fn storage_get(&self, idx: usize) -> Option<&T>;
    fn storage_get_mut(&mut self, idx: usize) -> Option<&mut T>;
    fn update_filter_cache(
        &mut self,
        matcher: &mut Matcher,
        top_k: Option<usize>,
    ) {
        self.update_filter(matcher, top_k, false);
    }

    fn update_filter_query(
        &mut self,
        matcher: &mut Matcher,
        top_k: Option<usize>,
    ) {
        self.update_filter(matcher, top_k, true);
    }

    fn update_filter(
        &mut self,
        matcher: &mut Matcher,
        top_k: Option<usize>,
        query_edit: bool,
    ) {
        if self.filter().cache.query == self.filter().query {
            return;
        }
        if self.filter().cache.utfstrings_cache.is_none() {
            let Some(cache) = self.build_utfstrings_cache() else {
                self.filter_mut().cache.clear_matches();
                return;
            };
            self.filter_mut().cache.utfstrings_cache = Some(cache.into());
        }
        self.filter_mut().update_matches(matcher, top_k, query_edit);
    }

    fn display_to_storage(&self, display_idx: usize) -> Option<usize> {
        if self.should_filter() {
            self.filter().cache.order.get(display_idx).copied()
        } else {
            Some(display_idx)
        }
    }

    fn storage_to_display(&self, storage_idx: usize) -> Option<usize> {
        if self.should_filter() {
            self.filter()
                .cache
                .ranks
                .get(storage_idx)
                .copied()
                .flatten()
        } else {
            (storage_idx < self.storage_len()).then_some(storage_idx)
        }
    }

    fn display_len(&self) -> usize {
        if self.should_filter() {
            self.filter().cache.order.len()
        } else {
            self.storage_len()
        }
    }

    fn display_get(&self, display_idx: usize) -> Option<&T> {
        let storage_idx = self.display_to_storage(display_idx)?;
        self.storage_get(storage_idx)
    }

    fn display_get_mut(&mut self, display_idx: usize) -> Option<&mut T> {
        let storage_idx = self.display_to_storage(display_idx)?;
        self.storage_get_mut(storage_idx)
    }

    fn selected_item(&self) -> Option<&T> {
        self.selected().and_then(|i| self.display_get(i))
    }
    fn selected_item_mut(&mut self) -> Option<&mut T> {
        let selected = self.selected()?;
        self.display_get_mut(selected)
    }
    fn should_filter(&self) -> bool {
        self.filter().active
            && !self.filter().cache.order.is_empty()
            && !self.filter().query.is_empty()
    }
}
