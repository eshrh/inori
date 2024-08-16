use super::*;

impl SearchState {
    pub fn new() -> Self {
        Self {
            active: false,
            query: String::new()
        }
    }
}
