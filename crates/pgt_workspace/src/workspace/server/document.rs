use pgt_diagnostics::{Diagnostic, DiagnosticExt, Severity, serde::Diagnostic as SDiagnostic};
use pgt_text_size::{TextRange, TextSize};

use super::statement_identifier::{IdGenerator, StatementId};

type StatementPos = (StatementId, TextRange);

pub(crate) struct Document {
    pub(crate) content: String,
    pub(crate) version: i32,

    pub(super) diagnostics: Vec<SDiagnostic>,
    /// List of statements sorted by range.start()
    pub(super) positions: Vec<StatementPos>,

    pub(super) id_generator: IdGenerator,
}

impl Document {
    pub(crate) fn new(content: String, version: i32) -> Self {
        let mut id_generator = IdGenerator::new();

        let (ranges, diagnostics) = split_with_diagnostics(&content, None);

        Self {
            positions: ranges
                .into_iter()
                .map(|range| (id_generator.next(), range))
                .collect(),
            content,
            version,
            diagnostics,
            id_generator,
        }
    }

    pub fn diagnostics(&self) -> &[SDiagnostic] {
        &self.diagnostics
    }

    /// Returns true if there is at least one fatal error in the diagnostics
    ///
    /// A fatal error is a scan error that prevents the document from being used
    pub(super) fn has_fatal_error(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity() == Severity::Fatal)
    }

    pub fn iter<'a>(&'a self) -> StatementIterator<'a> {
        StatementIterator::new(self)
    }
}

/// Helper function that wraps the statement splitter and returns the ranges with unified
/// diagnostics
pub(crate) fn split_with_diagnostics(
    content: &str,
    offset: Option<TextSize>,
) -> (Vec<TextRange>, Vec<SDiagnostic>) {
    let o = offset.unwrap_or_else(|| 0.into());
    match pgt_statement_splitter::split(content) {
        Ok(parse) => (
            parse.ranges,
            parse
                .errors
                .into_iter()
                .map(|err| {
                    SDiagnostic::new(
                        err.clone()
                            .with_file_span(err.location().span.map(|r| r + o)),
                    )
                })
                .collect(),
        ),
        Err(errs) => (
            vec![],
            errs.into_iter()
                .map(|err| {
                    SDiagnostic::new(
                        err.clone()
                            .with_file_span(err.location().span.map(|r| r + o)),
                    )
                })
                .collect(),
        ),
    }
}

pub trait StatementMapper<'a> {
    type Output;

    fn map(&self, doc: &'a Document, id: &'a StatementId, range: &'a TextRange) -> Self::Output;
}

pub struct StatementIterator<'a> {
    document: &'a Document,
    positions: std::slice::Iter<'a, StatementPos>,
}

impl<'a> StatementIterator<'a> {
    pub fn new(document: &'a Document) -> Self {
        Self {
            document,
            positions: document.positions.iter(),
        }
    }
}

impl<'a> Iterator for StatementIterator<'a> {
    type Item = (StatementId, TextRange, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.positions.next().map(|(id, range)| {
            let range = range.clone();
            let doc = self.document;
            let id = id.clone();
            (id, range, &doc.content[range.clone()])
        })
    }
}
