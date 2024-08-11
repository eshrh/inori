use super::*;
use crate::event_handler::Result;

impl QueueState {
    pub fn set_selected(&mut self, val: Option<usize>) {
        *self.state.selected_mut() = val;
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn get_sel_song(&self) -> Option<&Song> {
        match self.state.selected() {
            Some(i) => Some(&self.contents[i]),
            None => None,
        }
    }
    pub fn len(&self) -> usize {
        self.contents.len()
    }
}
