use mpd::Song;

pub fn safe_increment(idx: usize, length: usize) -> usize {
    (idx + 1) % length
}

pub fn safe_decrement(idx: usize, length: usize) -> usize {
    if length == 0 {
        return idx;
    }
    if idx == 0 {
        return length - 1;
    }
    idx - 1
}

pub fn song_album(s: &Song) -> Option<&String> {
    Some(&s.tags.iter().find(|t| t.0 == "Album")?.1)
}
