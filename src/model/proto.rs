extern crate mpd;
use super::*;

pub trait SelectorState {
    fn selected(&self) -> Option<usize>;
    fn set_selected(&mut self, s: Option<usize>);
}

impl SelectorState for ListState {
    fn selected(&self) -> Option<usize> {
        self.selected()
    }
    fn set_selected(&mut self, s: Option<usize>) {
        *self.selected_mut() = s;
    }
}

impl SelectorState for TableState {
    fn selected(&self) -> Option<usize> {
        self.selected()
    }
    fn set_selected(&mut self, s: Option<usize>) {
        *self.selected_mut() = s;
    }
}

pub trait Selector {
    fn selector(&self) -> &impl SelectorState;
    fn selector_mut(&mut self) -> &mut impl SelectorState;
    fn len(&self) -> usize;

    fn selected(&self) -> Option<usize> {
        self.selector().selected()
    }
    fn set_selected(&mut self, val: Option<usize>) {
        self.selector_mut().set_selected(val);
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
    fn contents(&self) -> Box<dyn Iterator<Item = &T> + '_>;
    fn selected_item_mut(&mut self) -> Option<&mut T>;
    fn update_filter_cache(
        &mut self,
        matcher: &mut Matcher,
        top_k: Option<usize>,
    );
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
