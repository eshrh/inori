use super::proto::*;
use super::*;
use crate::util::song_to_search_str;

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
                .map(|song| {
                    Utf32String::from(song_to_search_str(
                        song,
                        self.aliases.title_ref_for_path(&song.file),
                        self.aliases.album_ref_for_song(song),
                    ))
                })
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
            aliases: AliasMaps::default(),
        }
    }

    pub fn set_aliases(&mut self, aliases: &AliasMaps) {
        self.aliases = aliases.clone();
        self.songs.filter.cache.query.clear();
        self.songs.filter.cache.utfstrings_cache = None;
    }

    pub fn id_at(&self, index: usize) -> Option<mpd::Id> {
        self.display_get(index)?.place.map(|place| place.id)
    }

    pub fn selected_id(&self) -> Option<mpd::Id> {
        self.selected().and_then(|index| self.id_at(index))
    }

    pub fn replace_songs(
        &mut self,
        songs: Vec<Song>,
        matcher: &mut Matcher,
        top_k: usize,
    ) {
        let selected_id = self.selected_id();
        let selected_index = self.selected();
        if self.songs.items != songs {
            self.songs.replace_items(songs);
        }
        if self.songs.filter.active {
            self.update_filter_cache(matcher, Some(top_k));
        }

        let selected = selected_id
            .and_then(|id| {
                let index =
                    self.songs.items.iter().position(|song| {
                        song.place.is_some_and(|p| p.id == id)
                    })?;
                self.storage_to_display(index)
            })
            .or_else(|| {
                let len = self.display_len();
                selected_index
                    .filter(|_| len != 0)
                    .map(|index| index.min(len - 1))
            });
        self.set_selected(selected);
    }
}
