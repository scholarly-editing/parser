pub fn char_slice(s: &str, start: usize, end: usize) -> &str {
    let start_byte = s
        .char_indices()
        .nth(start)
        .map(|(i, _)| i)
        .unwrap_or_else(|| s.len());
    let end_byte = s
        .char_indices()
        .nth(end)
        .map(|(i, _)| i)
        .unwrap_or_else(|| s.len());
    &s[start_byte..end_byte]
}
