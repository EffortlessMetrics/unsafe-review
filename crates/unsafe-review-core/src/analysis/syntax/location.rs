#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LineColumn {
    pub(crate) line: usize,
    pub(crate) column: usize,
}

pub(crate) fn line_column(text: &str, offset: usize) -> LineColumn {
    let offset = offset.min(text.len());
    let line_start = line_start_before_offset(text, offset);

    LineColumn {
        line: line_number_before_offset(text, offset),
        column: column_number_after_line_start(text, line_start, offset),
    }
}

fn line_number_before_offset(text: &str, offset: usize) -> usize {
    text.char_indices()
        .take_while(|(idx, _ch)| *idx < offset)
        .filter(|(_idx, ch)| *ch == '\n')
        .count()
        + 1
}

fn line_start_before_offset(text: &str, offset: usize) -> usize {
    text.char_indices()
        .take_while(|(idx, _ch)| *idx < offset)
        .filter_map(|(idx, ch)| (ch == '\n').then_some(idx + ch.len_utf8()))
        .last()
        .unwrap_or(0)
}

fn column_number_after_line_start(text: &str, line_start: usize, offset: usize) -> usize {
    text.char_indices()
        .skip_while(|(idx, _ch)| *idx < line_start)
        .take_while(|(idx, _ch)| *idx < offset)
        .count()
        + 1
}
