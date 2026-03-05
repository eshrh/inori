use mpd::Song;
use mpd::Status;
use std::time::Duration;

/// Returns lhs + rhs, but keeps the value between 0 (inclusive) and max_value (exclusive).
/// Returns lhs with no change if max_value == 0.
pub fn safe_add(lhs: usize, rhs: usize, max_value: usize) -> usize {
    if max_value == 0 {
        return lhs;
    }
    if lhs + rhs >= max_value {
        return max_value - 1;
    }
    lhs + rhs
}

/// Returns lhs - rhs, but returns 0 if the result would be lower than 0.
/// Returns lhs with no change if max_value == 0.
pub fn safe_subtract(lhs: usize, rhs: usize, max_value: usize) -> usize {
    if max_value == 0 || lhs == 0 {
        return lhs;
    }
    if rhs >= lhs {
        return 0;
    }
    lhs - rhs
}

pub fn song_album(s: &Song) -> Option<&String> {
    Some(&s.tags.iter().find(|t| t.0 == "Album")?.1)
}

/// Formats a duration as HH:MM:SS or MM:SS as needed.
pub fn format_time(d: Duration) -> String {
    let total = d.as_secs();
    let m = total / 60;
    let s = total % 60;
    if m > 59 {
        format!("{}:{:02}:{:02}", m / 60, m % 60, s)
    } else {
        format!("{}:{:02}", m, s)
    }
}

/// Progress on the currently playing song, in the format {elapsed}/{duration}. 
pub fn format_progress(s: &Status) -> String {
    if let (Some(e), Some(d)) = (s.elapsed, s.duration) {
        format!("{}/{}", format_time(e), &format_time(d))
    } else {
        String::new()
    }
}

/// String representation of a song in the queue.
pub fn song_to_str(song: &Song) -> String {
    let mut out = String::new();
    if let Some(title) = &song.title {
        out.push_str(title);
    }
    if let Some(artist) = &song.artist {
        out.push(' ');
        out.push_str(artist);
    }
    if let Some(album) = song_album(song) {
        out.push(' ');
        out.push_str(album);
    }
    out
}
