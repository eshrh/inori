use super::*;
use nucleo_matcher::Matcher;
use proto::*;
use search_utils::*;

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
        if self.filter().active {
            self.filter()
                .cache
                .order
                .iter()
                .take_while(|i| i.is_some())
                .count()
        } else {
            match &self.contents {
                Some(v) => v.len(),
                None => 0,
            }
        }
    }
}

impl Searchable<InfoEntry> for GlobalSearchState {
    fn filter(&self) -> &Filter {
        &self.search
    }
    fn filter_mut(&mut self) -> &mut Filter {
        &mut self.search
    }
    fn contents(&self) -> Box<dyn Iterator<Item = &InfoEntry> + '_> {
        match &self.contents {
            Some(c) => {
                if self.should_filter() {
                    Box::new(
                        self.filter()
                            .cache
                            .order
                            .iter()
                            .filter_map(|idx| idx.map(|i| &c[i])),
                    )
                } else {
                    Box::new(c.iter())
                }
            }
            None => Box::new(std::iter::empty()),
        }
    }

    fn selected_item_mut(&mut self) -> Option<&mut InfoEntry> {
        let selected = self.selector().selected()?;
        let use_filter = self.should_filter();
        if use_filter {
            let idx = self.filter().cache.order.get(selected).cloned()??;
            self.contents.as_mut()?.get_mut(idx)
        } else {
            self.contents.as_mut()?.get_mut(selected)
        }
    }

    fn update_filter_cache(
        &mut self,
        matcher: &mut Matcher,
        top_k: Option<usize>,
    ) {
        if self.search.query == self.search.cache.query {
            return;
        }
        if self.contents.is_none() {
            self.search.cache.order = Vec::new();
            self.search.cache.indices = Vec::new();
            return;
        }
        if self.filter().cache.utfstrings_cache.is_none() {
            self.filter_mut().cache.utfstrings_cache = Some(
                self.contents
                    .iter()
                    .flatten()
                    .map(|i| Utf32String::from(i.to_search_string()))
                    .collect(),
            );
        }
        let query = self.filter().query.clone();
        let order = {
            let Some(cache) = self.filter().cache.utfstrings_cache.as_ref()
            else {
                return;
            };
            compute_orders(&query, cache, matcher, 0)
        };
        let strings: Vec<&Utf32String> = {
            let Some(cache) = self.filter().cache.utfstrings_cache.as_ref()
            else {
                return;
            };
            let strings_iterator = order
                .iter()
                .take_while(|i| i.is_some())
                .filter_map(|i| i.and_then(|idx| cache.get(idx)));
            if let Some(k) = top_k {
                strings_iterator.take(k).collect()
            } else {
                strings_iterator.collect()
            }
        };
        let indices = compute_indices(&query, strings, matcher);

        self.filter_mut().cache.query = query;
        self.filter_mut().cache.order = order;
        self.filter_mut().cache.indices = indices;
    }
}
