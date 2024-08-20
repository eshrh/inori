use super::*;

impl FilterCache {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            order: Vec::new(),
            indices: Vec::new(),
        }
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
}
