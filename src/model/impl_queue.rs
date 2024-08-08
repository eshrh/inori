use super::*;
use crate::event_handler::Result;

impl QueueState {
    pub fn get_sel(&self) -> Option<&Song> {
        match self.selection {
            Some(i) => Some(&self.contents[i]),
            None => None,
        }
    }
    pub fn len(&self) -> usize {
        self.contents.len()
    }
}
