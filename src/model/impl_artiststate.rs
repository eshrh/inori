use super::*;
use crate::model::search_utils::compute_orders;
use proto::*;
use search_utils::compute_indices;

impl Selector for ArtistData {
    fn selector(&self) -> &impl SelectorState {
        &self.track_sel_state
    }
    fn selector_mut(&mut self) -> &mut impl SelectorState {
        &mut self.track_sel_state
    }
    fn len(&self) -> usize {
        self.albums.len()
            + self
                .albums
                .iter()
                .map(|i| if i.expanded { i.tracks.len() } else { 0 })
                .sum::<usize>()
    }
}

impl<'a> ArtistData {
    fn find_rank(&self, idx: usize) -> Option<usize> {
        self.search
            .cache
            .order
            .iter()
            .take_while(|i| i.is_some())
            .position(|i| *i == Some(idx))
    }
    pub fn expand_all(&mut self) {
        for album in &mut self.albums {
            if !album.expanded {
                album.expanded = true;
            }
        }
    }
    pub fn contents(&'a self) -> Vec<TrackSelItem> {
        let mut new: Vec<TrackSelItem> = Vec::new();
        let mut i = 0; // full index
        for album in &self.albums {
            if self.search.active {
                new.push(TrackSelItem::from(album).rank(self.find_rank(i)))
            } else {
                new.push(album.into());
            }
            i += 1;
            if album.expanded {
                for track in &album.tracks {
                    if self.search.active {
                        new.push(
                            TrackSelItem::from(track).rank(self.find_rank(i)),
                        )
                    } else {
                        new.push(track.into());
                    }
                    i += 1;
                }
            } else {
                i += album.tracks.len();
            }
        }
        // if self.search.active && self.search.query.len() > 0 {
        //     println!("{:?}", new.iter().map(|i| i.rank).collect::<Vec<Option<usize>>>());
        //     // println!("{:?}", self.search.cache.order);
        //     panic!();
        // }
        new
    }
    pub fn selected_item(&self) -> Option<TrackSelItem> {
        let sel_idx = self.selector().selected()?;
        let mut i = 0; // keeps track of index with folding
        let mut full_index = 0; // keeps track of index without considering folding
        for album in &self.albums {
            if sel_idx == i {
                if self.search.active {
                    return Some(
                        TrackSelItem::from(album)
                            .rank(self.find_rank(full_index)),
                    );
                } else {
                    return Some(album.into());
                }
            }
            i += 1;
            full_index += 1;
            let al_len = album.tracks.len();
            if album.expanded {
                if (sel_idx - i) < al_len {
                    if self.search.active {
                        return album.tracks.get(sel_idx - i).map(|song| {
                            TrackSelItem::from(song).rank(
                                self.find_rank(full_index + (sel_idx - i)),
                            )
                        });
                    } else {
                        return album
                            .tracks
                            .get(sel_idx - i)
                            .map(|song| song.into());
                    }
                }
                i += al_len;
            }
            full_index += al_len;
        }
        None
    }
    pub fn selected_album_mut(&mut self) -> Option<&mut AlbumData> {
        // assumption: order in self.albums is the same as in the viewer.
        // NOTE: can't use TrackSelItem enum since references are immutable.
        // Tried this and it's busted.
        let sel_idx = self.selector().selected()?;
        let mut i = 0;
        let mut album_i = 0;
        for album in &self.albums {
            if sel_idx == i {
                return self.albums.get_mut(album_i);
            }
            album_i += 1;
            i += 1;
            if album.expanded {
                i += album.tracks.len()
            }
        }
        None
    }

    pub fn update_search(&mut self, matcher: &mut Matcher) {
        if self.search.cache.query == self.search.query {
            return;
        }
        if self.search.cache.utfstrings_cache.is_none() {
            let mut tmp: Vec<Utf32String> = Vec::new();
            for album in &self.albums {
                tmp.push(Utf32String::from(album.name.clone()));
                for track in &album.tracks {
                    tmp.push(Utf32String::from(
                        track.title.clone().unwrap_or("".into()),
                    ));
                }
            }
            self.search.cache.utfstrings_cache = Some(tmp);
        }
        self.search.cache.order = compute_orders(
            &self.search.query,
            self.search.cache.utfstrings_cache.as_ref().unwrap(),
            matcher,
            0,
        );
        let strings_for_indices: Vec<&Utf32String> = self
            .search
            .cache
            .order
            .iter()
            .take_while(|i| i.is_some())
            .map(|i| {
                &self.search.cache.utfstrings_cache.as_ref().unwrap()
                    [i.unwrap()]
            })
            .collect();

        self.search.cache.indices =
            compute_indices(&self.search.query, strings_for_indices, matcher);
        self.search.cache.query = self.search.query.clone();

        if self.search.cache.order.iter().any(|i| i.is_some()) {
            let mut i = 0;
            for album in self.albums.iter_mut() {
                album.expanded = false;
                if self.search.cache.order.contains(&Some(i)) {
                    album.expanded = true;
                }
                i += 1;
                for _ in &album.tracks {
                    if self.search.cache.order.contains(&Some(i)) {
                        album.expanded = true;
                    }
                    i += 1
                }
            }
        }
        let mut top_idx: Option<usize> = None;
        for (i, item) in self.contents().iter().enumerate() {
            if let Some(0) = item.rank {
                top_idx = Some(i);
            }
        }
        self.set_selected(top_idx);
    }
}

impl ArtistData {
    pub fn from_names(name: String, sort_names: Vec<String>) -> Self {
        Self {
            name,
            fetched: false,
            albums: Vec::new(),
            sort_names,
            track_sel_state: TableState::default(),
            search: Filter::new(),
        }
    }
    pub fn to_fuzzy_find_str(&self) -> String {
        if self.sort_names.first().is_some_and(|n| *n == self.name) {
            self.name.clone()
        } else {
            format!("{} [{}]", self.name, self.sort_names.join(", "))
        }
    }
}

impl<'a> From<&'a AlbumData> for TrackSelItem<'a> {
    fn from(value: &'a AlbumData) -> Self {
        Self {
            item: ItemRef::Album(value),
            rank: None,
        }
    }
}

impl<'a> From<&'a Song> for TrackSelItem<'a> {
    fn from(value: &'a Song) -> Self {
        Self {
            item: ItemRef::Song(value),
            rank: None,
        }
    }
}

impl<'a> TrackSelItem<'a> {
    pub fn rank(mut self, val: Option<usize>) -> Self {
        self.rank = val;
        self
    }
}

impl<'a> ToString for TrackSelItem<'a> {
    fn to_string(&self) -> String {
        match self.item {
            ItemRef::Album(a) => a.name.clone(),
            ItemRef::Song(s) => {
                s.title.clone().unwrap_or("<SONG TITLE NOT FOUND>".into())
            }
        }
    }
}
