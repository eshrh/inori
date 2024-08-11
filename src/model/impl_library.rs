use super::{ArtistData, LibraryState};

impl LibraryState {
    pub fn artist_selected_pos(&self) -> Option<usize> {
        self.artist_state.selected()
    }

    pub fn artist_selected_name(&self) -> Option<&String> {
        Some(&self.artists[self.artist_selected_pos()?])
    }

    pub fn artist_selected(&self) -> Option<&ArtistData> {
        self.contents.get(self.artist_selected_name()?)
    }

    pub fn artist_selected_mut(&mut self) -> Option<&mut ArtistData> {
        let artist_name = &self.artists[self.artist_selected_pos()?];
        self.contents.get_mut(artist_name)
    }

    pub fn set_artist_selected(&mut self, val: Option<usize>) {
        *self.artist_state.selected_mut() = val;
    }
}
