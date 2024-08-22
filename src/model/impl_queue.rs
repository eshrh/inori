use nucleo_matcher::Matcher;

use super::proto::*;
use super::*;

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
        Box::new(self.contents.iter())
    }
    fn selected_item_mut(&mut self) -> Option<&mut Song> {
        self.selector()
            .selected()
            .and_then(|i| self.contents.get_mut(i))
    }
    fn update_filter_cache(&mut self, matcher: &mut Matcher) {
        unimplemented!();
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
