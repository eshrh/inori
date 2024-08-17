use super::*;
use std::time::Duration;

impl AlbumData {
    pub fn total_time(&self) -> Duration {
        self.tracks
            .iter()
            .map(|i| i.duration.unwrap_or(Duration::from_secs(0)))
            .sum()
    }
}
