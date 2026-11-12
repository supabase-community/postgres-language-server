use pgls_text_size::{TextRange, TextSize};

/// The range of `old` that differs from `new`, and the text of `new` that replaces it, so that an
/// edit replaces only what changed and the editor keeps the cursor and the folds. `None` if `old`
/// is too large for a `TextSize`.
///
/// A position can't point between the `\r` and the `\n` of a line break, so the range takes in the
/// whole line break when only one half of it changed. The characters it takes in are the same in
/// `old` and `new`.
pub fn changed_range<'a>(old: &str, new: &'a str) -> Option<(TextRange, &'a str)> {
    let prefix = old
        .char_indices()
        .zip(new.chars())
        .find(|((_, a), b)| a != b)
        .map_or(old.len().min(new.len()), |((index, _), _)| index);
    let suffix = old[prefix..]
        .chars()
        .rev()
        .zip(new[prefix..].chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum::<usize>();
    let prefix = if splits_crlf(old, prefix) {
        prefix - 1
    } else {
        prefix
    };
    let suffix = if splits_crlf(old, old.len() - suffix) {
        suffix - 1
    } else {
        suffix
    };
    let start = TextSize::try_from(prefix).ok()?;
    let end = TextSize::try_from(old.len() - suffix).ok()?;
    Some((TextRange::new(start, end), &new[prefix..new.len() - suffix]))
}

/// Whether `offset` is between the `\r` and the `\n` of a line break.
fn splits_crlf(text: &str, offset: usize) -> bool {
    text[..offset].ends_with('\r') && text[offset..].starts_with('\n')
}
