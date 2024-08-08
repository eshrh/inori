pub fn safe_increment(idx: usize, length: usize) -> usize {
    (idx + 1) % length
}

pub fn safe_decrement(idx: usize, length: usize) -> usize {
    if idx == 0 {
        return length - 1;
    }
    return idx - 1;
}
