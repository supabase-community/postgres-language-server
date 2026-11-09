//! What migration tools replace in a file before they run it, like goose's `ENVSUB`.

pub(crate) mod envsub;
pub(crate) mod interpolate;

use pgls_text_size::TextRange;

/// The text that the analysis reads instead of the file as written, with placeholders where the
/// migration tool replaces parts of it. It has the length of the file, so that the offsets of
/// diagnostics, fixes and editor positions are the same in both.
#[derive(Debug, PartialEq, Eq)]
pub struct Preprocessed {
    pub text: String,
    /// The ranges that the tool replaces when it runs the file, in file order. The text holds a
    /// placeholder there, or the file as written where a placeholder wouldn't parse, like in a
    /// string literal.
    pub placeholders: Vec<TextRange>,
}

impl Preprocessed {
    /// Whether `range` overlaps a placeholder. An empty range touches it if it's inside it.
    pub fn touches_placeholder(&self, range: TextRange) -> bool {
        self.placeholders.iter().any(|placeholder| {
            if range.is_empty() {
                placeholder.start() < range.start() && range.start() < placeholder.end()
            } else {
                placeholder.start() < range.end() && range.start() < placeholder.end()
            }
        })
    }
}
