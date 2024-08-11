use super::{ArtistData, LibraryState};

impl LibraryState {
    pub fn artist_selected_pos(&self) -> Option<usize> {
        self.artist_state.selected()
    }

    pub fn artist_selected_name(&self) -> Option<&String> {
        Some(&self.contents[self.artist_selected_pos()?].name)
    }

    pub fn artist_selected(&self) -> Option<&ArtistData> {
        Some(&self.contents[self.artist_selected_pos()?])
    }

    pub fn artist_selected_mut(&mut self) -> Option<&mut ArtistData> {
        let pos = self.artist_selected_pos()?;
        self.contents.get_mut(pos)
    }

    pub fn set_artist_selected(&mut self, val: Option<usize>) {
        *self.artist_state.selected_mut() = val;
    }
}
