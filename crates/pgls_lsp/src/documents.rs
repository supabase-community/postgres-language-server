use std::sync::atomic::{AtomicU64, Ordering};

use crate::adapters::line_index::LineIndex;

/// Represents an open [`textDocument`]. Can be cheaply cloned.
///
/// [`textDocument`]: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocumentItem
#[derive(Clone)]
pub(crate) struct Document {
    pub(crate) version: i32,
    /// Which version of which document this is. Unlike `version`, it is unique: it changes on
    /// every change, and also when the document is closed and opened again with the same
    /// version. A result computed for a revision is outdated once the document has another.
    pub(crate) revision: u64,
    pub(crate) line_index: LineIndex,
}

impl Document {
    pub(crate) fn new(version: i32, text: &str) -> Self {
        static REVISIONS: AtomicU64 = AtomicU64::new(0);
        Self {
            version,
            revision: REVISIONS.fetch_add(1, Ordering::Relaxed),
            line_index: LineIndex::new(text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Document;

    #[test]
    fn reopened_documents_have_another_revision() {
        let opened = Document::new(1, "select 1;");
        let reopened = Document::new(1, "select 1;");
        assert_ne!(opened.revision, reopened.revision);
    }
}
