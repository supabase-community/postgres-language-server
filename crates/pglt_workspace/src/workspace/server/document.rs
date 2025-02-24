use pglt_diagnostics::{serde::Diagnostic as SDiagnostic, Diagnostic, DiagnosticExt, Severity};
use pglt_fs::PgLTPath;
use text_size::{TextRange, TextSize};

/// Global unique identifier for a statement
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub(crate) struct Statement {
    /// Path of the document
    pub(crate) path: PgLTPath,
    /// Unique id within the document
    pub(crate) id: StatementId,
}

pub type StatementId = usize;

type StatementPos = (StatementId, TextRange);

pub(crate) struct Document {
    pub(crate) path: PgLTPath,
    pub(crate) content: String,
    pub(crate) version: i32,

    pub(super) diagnostics: Vec<SDiagnostic>,
    /// List of statements sorted by range.start()
    pub(super) positions: Vec<StatementPos>,

    pub(super) id_generator: IdGenerator,
}

impl Document {
    pub(crate) fn new(path: PgLTPath, content: String, version: i32) -> Self {
        let mut id_generator = IdGenerator::new();

        let (ranges, diagnostics) = split_with_diagnostics(&content, None);

        Self {
            path,
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

    pub fn iter_statements(&self) -> impl Iterator<Item = Statement> + '_ {
        self.positions.iter().map(move |(id, _)| Statement {
            id: *id,
            path: self.path.clone(),
        })
    }

    pub fn iter_statements_with_text(&self) -> impl Iterator<Item = (Statement, &str)> + '_ {
        self.positions.iter().map(move |(id, range)| {
            let statement = Statement {
                id: *id,
                path: self.path.clone(),
            };
            let text = &self.content[range.start().into()..range.end().into()];
            (statement, text)
        })
    }

    pub fn iter_statements_with_range(&self) -> impl Iterator<Item = (Statement, &TextRange)> + '_ {
        self.positions.iter().map(move |(id, range)| {
            let statement = Statement {
                id: *id,
                path: self.path.clone(),
            };
            (statement, range)
        })
    }

    pub fn iter_statements_with_text_and_range(
        &self,
    ) -> impl Iterator<Item = (Statement, &TextRange, &str)> + '_ {
        self.positions.iter().map(move |(id, range)| {
            let statement = Statement {
                id: *id,
                path: self.path.clone(),
            };
            (
                statement,
                range,
                &self.content[range.start().into()..range.end().into()],
            )
        })
    }
}

pub(crate) struct IdGenerator {
    pub(super) next_id: usize,
}

impl IdGenerator {
    fn new() -> Self {
        Self { next_id: 0 }
    }

    pub(super) fn next(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

/// Helper function that wraps the statement splitter and returns the ranges with unified
/// diagnostics
pub(crate) fn split_with_diagnostics(
    content: &str,
    offset: Option<TextSize>,
) -> (Vec<TextRange>, Vec<SDiagnostic>) {
    let o = offset.unwrap_or_else(|| 0.into());
    match pglt_statement_splitter::split(content) {
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
