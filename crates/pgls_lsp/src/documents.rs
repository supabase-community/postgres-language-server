use pgls_line_index::LineIndex;

use crate::activity::Activity;

/// Represents an open [`textDocument`]. The session keeps each state of the document in its own
/// `Arc`, see [`Session::is_current_document`].
///
/// [`textDocument`]: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocumentItem
/// [`Session::is_current_document`]: crate::session::Session::is_current_document
pub(crate) struct Document {
    /// The client's version of the document. It is only sent back to the client, e.g. with the
    /// diagnostics: it can repeat when the document is closed and opened again, so it doesn't
    /// tell whether a result is outdated.
    pub(crate) version: i32,
    pub(crate) line_index: LineIndex,
}

impl Document {
    /// The document of `text`, whose line index `activity` counts.
    pub(crate) fn new(version: i32, text: &str, activity: &Activity) -> Self {
        activity.line_index_built();
        Self {
            version,
            line_index: LineIndex::new(text),
        }
    }
}
