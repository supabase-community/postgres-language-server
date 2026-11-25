use std::ops::Range;

use pgls_text_size::{TextRange, TextSize};

/// Replaces the text in `range` with `new_text`, e.g. an edit of a fix or a code action. Unlike
/// `TextEdit`, which is a diff of a whole text, it only knows the range it replaces.
///
/// The ranges come from rules and frontends, so every application checks them, in release builds
/// too: an edit out of order, or one that doesn't lie on char boundaries, is rejected instead of
/// panicking or changing the wrong text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeEdit {
    pub range: TextRange,
    pub new_text: String,
}

impl RangeEdit {
    pub fn new(range: TextRange, new_text: impl Into<String>) -> Self {
        Self {
            range,
            new_text: new_text.into(),
        }
    }

    /// Applies the edit to `text` in place. `false`, and `text` unchanged, if the range doesn't
    /// lie on char boundaries of `text`.
    pub fn apply(&self, text: &mut String) -> bool {
        let range = Range::<usize>::from(self.range);
        if text.get(range.clone()).is_none() {
            return false;
        }
        text.replace_range(range, &self.new_text);
        true
    }

    /// `text` with `edits` applied, where `text` starts at offset `start` of the text that the
    /// ranges are relative to. `None` unless the edits are sorted, don't overlap, and lie on char
    /// boundaries of `text`.
    ///
    /// Cost: one copy of `text`; the check is part of the same pass.
    pub fn apply_all(text: &str, start: TextSize, edits: &[RangeEdit]) -> Option<String> {
        let mut result = String::with_capacity(text.len());
        let mut last = 0;
        for edit in edits {
            let range = Range::<usize>::from(edit.range.checked_sub(start)?);
            if range.start < last {
                return None;
            }
            text.get(range.clone())?;
            result.push_str(text.get(last..range.start)?);
            result.push_str(&edit.new_text);
            last = range.end;
        }
        result.push_str(text.get(last..)?);
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use pgls_text_size::{TextRange, TextSize};

    use super::RangeEdit;

    fn edit(start: u32, end: u32, new_text: &str) -> RangeEdit {
        RangeEdit::new(TextRange::new(start.into(), end.into()), new_text)
    }

    #[test]
    fn applies_sorted_edits() {
        let edits = [edit(0, 6, "SELECT"), edit(7, 7, "a, "), edit(8, 9, "")];
        assert_eq!(
            RangeEdit::apply_all("select 1;", TextSize::from(0), &edits).as_deref(),
            Some("SELECT a, 1")
        );
        // The same edits on the text from offset 10 of a file.
        let edits = edits.map(|mut edit| {
            edit.range += TextSize::from(10);
            edit
        });
        assert_eq!(
            RangeEdit::apply_all("select 1;", TextSize::from(10), &edits).as_deref(),
            Some("SELECT a, 1")
        );
        assert_eq!(
            RangeEdit::apply_all("", TextSize::from(0), &[]).as_deref(),
            Some("")
        );
    }

    #[test]
    fn rejects_edits_that_dont_fit() {
        let apply = |text, edits: &[RangeEdit]| RangeEdit::apply_all(text, 0.into(), edits);
        // Out of order, overlapping, past the end, inside `é` (bytes 8..10).
        assert_eq!(apply("select 1;", &[edit(7, 8, "2"), edit(0, 6, "")]), None);
        assert_eq!(apply("select 1;", &[edit(0, 6, ""), edit(5, 8, "")]), None);
        assert_eq!(apply("select 1;", &[edit(9, 10, "")]), None);
        assert_eq!(apply("select 'é';", &[edit(9, 10, "e")]), None);
        // Before the start of the text.
        assert_eq!(
            RangeEdit::apply_all("select 1;", 10.into(), &[edit(0, 1, "")]),
            None
        );

        let mut text = "select 'é';".to_string();
        assert!(!edit(9, 10, "e").apply(&mut text));
        assert_eq!(text, "select 'é';");
        assert!(edit(8, 10, "e").apply(&mut text));
        assert_eq!(text, "select 'e';");
    }
}
