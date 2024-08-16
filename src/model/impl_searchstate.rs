use super::*;

impl Filter {
    pub fn new() -> Self {
        Self {
            active: false,
            query: String::new()
        }
    }
}
