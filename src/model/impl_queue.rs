use super::selector_state::*;
use super::*;

impl Selector for QueueSelector {
    fn selector(&self) -> &impl SelectorState {
        &self.state
    }
    fn selector_mut(&mut self) -> &mut impl SelectorState {
        &mut self.state
    }
    fn len(&self) -> usize {
        self.contents.len()
    }
}

impl Searchable<Song> for QueueSelector {
    fn filter(&self) -> &Filter {
        &self.search
    }
    fn filter_mut(&mut self) -> &mut Filter {
        &mut self.search
    }
    fn contents(&self) -> Box<dyn Iterator<Item=&Song> + '_> {
        Box::new(self.contents.iter())
    }
    fn contents_mut(&mut self) -> Box<dyn Iterator<Item=&mut Song> + '_> {
        Box::new(self.contents.iter_mut())
    }
}

impl QueueSelector {
    pub fn new() -> Self {
        Self {
            search: Filter::new(),
            contents: vec![],
            state: TableState::default(),
        }
    }
}
