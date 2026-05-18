pub(crate) fn snippet(text: &str, start: usize, end: usize) -> String {
    text.get(start..end)
        .map_or_else(String::new, str::to_string)
}

pub(crate) fn text_size_to_usize(size: ra_ap_syntax::TextSize) -> usize {
    u32::from(size) as usize
}
