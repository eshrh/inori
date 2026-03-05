use super::*;
use proto::*;

impl FilterCache {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            order: Vec::new(),
            indices: Vec::new(),
            utfstrings_cache: None,
        }
    }
    pub fn clear_matches(&mut self) {
        self.query.clear();
        self.order.clear();
        self.indices.clear();
    }
}

impl Filter {
    pub fn new() -> Self {
        Self {
            active: false,
            query: String::new(),
            cache: FilterCache::new(),
        }
    }
    pub fn set_on(&mut self) {
        self.active = true;
    }
    pub fn set_off(&mut self) {
        self.active = false;
        self.query.clear();
        self.cache.clear_matches();
    }
    pub fn reset_cache(&mut self) {
        self.cache = FilterCache::new();
    }
}

#[derive(Debug)]
pub enum InfoEntryError {
    MissingArtist,
    TooManyFields(usize),
}

impl std::fmt::Display for InfoEntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfoEntryError::MissingArtist => {
                write!(f, "missing artist field in search entry")
            }
            InfoEntryError::TooManyFields(len) => {
                write!(f, "too many fields in search entry: {}", len)
            }
        }
    }
}

impl std::error::Error for InfoEntryError {}

impl TryFrom<&mut Vec<String>> for InfoEntry {
    type Error = InfoEntryError;

    fn try_from(v: &mut Vec<String>) -> std::result::Result<Self, Self::Error> {
        if v.len() > 4 {
            return Err(InfoEntryError::TooManyFields(v.len()));
        }
        let mut drained = v.drain(..);
        let artist = drained.nth(0).ok_or(InfoEntryError::MissingArtist)?;
        Ok(InfoEntry {
            artist,
            artist_sort: drained.nth(0),
            album: drained.nth(0),
            title: drained.nth(0),
        })
    }
}

impl InfoEntry {
    pub fn is_redundant(&self) -> bool {
        self.album.is_none()
            && self.title.is_none()
            && self.artist_sort.as_ref().is_some_and(|i| *i == self.artist)
    }
    pub fn to_search_string(&self) -> String {
        let mut out: String = self.artist.clone();
        if let Some(artist_sort) = &self.artist_sort {
            if *artist_sort != out {
                out.push_str(&format!(" {}{}{}", "[", artist_sort, "]"));
            }
        }
        if let Some(album) = &self.album {
            out.push('/');
            out.push_str(album);
        }
        if let Some(title) = &self.title {
            out.push('/');
            out.push_str(title);
        }
        out
    }
}

impl Selector for GlobalSearchState {
    fn selector(&self) -> &impl SelectorState {
        &self.results_state
    }
    fn selector_mut(&mut self) -> &mut impl SelectorState {
        &mut self.results_state
    }
    fn len(&self) -> usize {
        self.display_len()
    }
}

impl Searchable<InfoEntry> for GlobalSearchState {
    fn filter(&self) -> &Filter {
        &self.entries.filter
    }
    fn filter_mut(&mut self) -> &mut Filter {
        &mut self.entries.filter
    }
    fn build_utfstrings_cache(&self) -> Option<Vec<Utf32String>> {
        self.loaded.then(|| {
            self.entries
                .items
                .iter()
                .map(|i| Utf32String::from(i.to_search_string()))
                .collect()
        })
    }
    fn storage_len(&self) -> usize {
        self.entries.items.len()
    }
    fn storage_get(&self, idx: usize) -> Option<&InfoEntry> {
        self.entries.items.get(idx)
    }
    fn storage_get_mut(&mut self, idx: usize) -> Option<&mut InfoEntry> {
        self.entries.items.get_mut(idx)
    }
}
