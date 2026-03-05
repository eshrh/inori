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
        self.display_len()
    }
}

impl Searchable<Song> for QueueSelector {
    fn filter(&self) -> &Filter {
        &self.songs.filter
    }
    fn filter_mut(&mut self) -> &mut Filter {
        &mut self.songs.filter
    }
    fn build_utfstrings_cache(&self) -> Option<Vec<Utf32String>> {
        Some(
            self.songs
                .items
                .iter()
                .map(|i| Utf32String::from(song_to_str(i)))
                .collect(),
        )
    }
    fn storage_len(&self) -> usize {
        self.songs.items.len()
    }
    fn storage_get(&self, idx: usize) -> Option<&Song> {
        self.songs.items.get(idx)
    }
    fn storage_get_mut(&mut self, idx: usize) -> Option<&mut Song> {
        self.songs.items.get_mut(idx)
    }
}

impl QueueSelector {
    pub fn new() -> Self {
        Self {
            songs: FilteredView::new(),
            state: TableState::default(),
        }
    }
}
