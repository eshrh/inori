use super::proto::*;
use super::*;
use crate::util::song_to_str;

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
    fn build_utfstrings_cache(&self) -> Option<Vec<Utf32String>> {
        Some(
            self.contents
                .iter()
                .map(|i| Utf32String::from(song_to_str(i)))
                .collect(),
        )
    }
    fn contents(&self) -> Box<dyn Iterator<Item = &Song> + '_> {
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
