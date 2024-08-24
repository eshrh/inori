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

impl From<&mut Vec<String>> for InfoEntry {
    fn from(v: &mut Vec<String>) -> Self {
        if v.len() > 4 {
            panic!("too much info given to infoentry");
        } else {
            let mut drained = v.drain(..);
            InfoEntry {
                artist: drained.nth(0).unwrap(),
                artist_sort: drained.nth(0),
                album: drained.nth(0),
                title: drained.nth(0),
            }
        }
    }
}

impl InfoEntry {
    pub fn is_redundant(&self) -> bool {
        self.album.is_none()
            && self.title.is_none()
            && self.artist_sort.as_ref().is_some_and(|i| *i == self.artist)
    }
}

impl ToString for InfoEntry {
    fn to_string(&self) -> String {
        let mut out: String = self.artist.clone();
        if let Some(artist_sort) = &self.artist_sort {
            if *artist_sort != out {
                out.push('/');
                out.push_str(artist_sort);
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
            self.filter().cache.order.len()
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
        unimplemented!();
    }

    fn update_filter_cache(&mut self, matcher: &mut Matcher) {
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
                    .map(|i| Utf32String::from(i.to_string()))
                    .collect(),
            );
        }

        self.filter_mut().cache.query = self.filter().query.clone();
        self.filter_mut().cache.order = compute_orders(
            &self.filter().query,
            self.filter().cache.utfstrings_cache.as_ref().unwrap(),
            matcher,
            0,
        );
    }
}
