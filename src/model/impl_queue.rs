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

impl SelectorWithContents<Song> for QueueSelector {
    fn contents(&self) -> &Vec<Song> {
        &self.contents
    }
    fn contents_mut(&mut self) -> &mut Vec<Song> {
        &mut self.contents
    }
}
