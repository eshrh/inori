extern crate mpd;
use ratatui::widgets::{ListState, TableState};

pub trait SelectorState {
    fn selected(&self) -> Option<usize>;
    fn set_selected(&mut self, s: Option<usize>);
    fn offset(&self) -> usize;
    fn set_offset(&mut self, o: usize);
}
impl SelectorState for ListState {
    fn selected(&self) -> Option<usize> {
        self.selected()
    }
    fn set_selected(&mut self, s: Option<usize>) {
        *self.selected_mut() = s;
    }
    fn offset(&self) -> usize {
        self.offset()
    }
    fn set_offset(&mut self, o: usize) {
        *self.offset_mut() = o;
    }
}

impl SelectorState for TableState {
    fn selected(&self) -> Option<usize> {
        self.selected()
    }
    fn set_selected(&mut self, s: Option<usize>) {
        *self.selected_mut() = s;
    }
    fn offset(&self) -> usize {
        self.offset()
    }
    fn set_offset(&mut self, o: usize) {
        *self.offset_mut() = o;
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
}
pub trait SelectorWithContents<T>: Selector {
    fn contents(&self) -> &Vec<T>;
    fn contents_mut(&mut self) -> &mut Vec<T>;
    fn selected_item_mut(&mut self) -> Option<&mut T> {
        match self.selector().selected() {
            Some(i) => Some(&mut self.contents_mut()[i]),
            None => None,
        }
    }
    fn selected_item(&self) -> Option<&T> {
        match self.selector().selected() {
            Some(i) => Some(&self.contents()[i]),
            None => None,
        }
    }
}
