use super::selector_state::*;
use super::*;

impl Selector<Song> for QueueSelector {
    fn selector(&self) -> &impl SelectorState {
        &self.state
    }
    fn selector_mut(&mut self) -> &mut impl SelectorState {
        &mut self.state
    }
    fn contents(&self) -> &Vec<Song> {
        &self.contents
    }
    fn contents_mut(&mut self) -> &mut Vec<Song> {
        &mut self.contents
    }
}
