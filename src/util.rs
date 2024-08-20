use mpd::Song;
use mpd::Status;
use std::time::Duration;

pub fn safe_increment(idx: usize, length: usize) -> usize {
    if length == 0 {
        return idx;
    }
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

pub fn format_progress(s: &Status) -> String {
    if let (Some(e), Some(d)) = (s.elapsed, s.duration) {
        format!("{}/{}", format_time(e), &format_time(d))
    } else {
        String::new()
    }
}
