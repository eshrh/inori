use super::proto::*;
use super::*;
use crate::util::song_to_str;
use nucleo_matcher::Matcher;

impl Selector for QueueSelector {
    fn selector(&self) -> &impl SelectorState {
        &self.state
    }
    fn selector_mut(&mut self) -> &mut impl SelectorState {
        &mut self.state
    }
    fn len(&self) -> usize {
        self.contents_vec().len()
    }
}

impl Searchable<Song> for QueueSelector {
    fn filter(&self) -> &Filter {
        &self.search
    }
    fn filter_mut(&mut self) -> &mut Filter {
        &mut self.search
    }
    fn contents(&self) -> Box<dyn Iterator<Item = &Song> + '_> {
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
    fn selected_item_mut(&mut self) -> Option<&mut Song> {
        if self.should_filter() {
            self.selector()
                .selected()
                .and_then(|i| self.filter().cache.order.get(i).cloned())
                .and_then(|i| self.contents.get_mut(i?))
        } else {
            self.selector()
                .selected()
                .and_then(|i| self.contents.get_mut(i))
        }
    }
    fn update_filter_cache(&mut self, matcher: &mut Matcher) {
        if self.filter().cache.query == self.filter().query {
            return;
        }
        if self.filter().cache.utfstrings_cache.is_none() {
            self.filter_mut().cache.utfstrings_cache = Some(
                self.contents
                    .iter()
                    .map(|i| Utf32String::from(song_to_str(i)))
                    .collect(),
            );
        }
        self.filter_mut().cache.order = search_utils::compute_orders(
            &self.filter().query,
            self.filter().cache.utfstrings_cache.as_ref().unwrap(),
            matcher,
            0,
        );
    }
}

impl QueueSelector {
    pub fn new() -> Self {
        Self {
            search: Filter::new(),
            contents: Vec::new(),
            state: TableState::default(),
        }
    }
}
