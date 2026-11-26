//! psql syntax in the migrations of a tool that runs them without psql, like a `\set` or a
//! `:name`. Postgres receives it as written and rejects it.

use pgls_diagnostics::{Diagnostic, MessageAndDescription};
use pgls_lexer::SyntaxKind;
use pgls_text_size::TextRange;

/// psql syntax in a migration of a tool that runs it without psql.
#[derive(Debug, Diagnostic)]
#[diagnostic(category = "migrations/psqlSyntax", severity = Warning)]
pub struct PsqlSyntaxDiagnostic {
    #[location(span)]
    pub span: TextRange,
    #[message]
    #[description]
    pub message: MessageAndDescription,
}

impl PsqlSyntaxDiagnostic {
    /// The psql meta-commands and variables in `text`, a migration that `tool` runs without psql,
    /// in text order.
    pub(crate) fn all_in(text: &str, tool: &str) -> Vec<Self> {
        let lexed = pgls_lexer::lex(text);
        // An unquoted `\` is never server SQL: it starts a psql meta-command, or is psql's `\;` or
        // `\:`.
        let meta_commands = (0..lexed.len())
            .filter(|idx| {
                matches!(
                    lexed.kind(*idx),
                    SyntaxKind::META_COMMAND | SyntaxKind::BACKSLASH
                )
            })
            .map(|idx| Self {
                span: lexed.range(idx),
                message: MessageAndDescription::from(format!(
                    "{tool} runs migrations without psql, so Postgres receives this psql meta-command and rejects it."
                )),
            });
        let variables = pgls_lexer::convert_to_positional_params_with_metadata(text)
            .psql_variables
            .into_iter()
            .map(|span| Self {
                span,
                message: MessageAndDescription::from(format!(
                    "{tool} runs migrations without psql, so nothing replaces this psql variable and Postgres rejects it."
                )),
            });
        let mut diagnostics: Vec<Self> = meta_commands.chain(variables).collect();
        diagnostics.sort_by_key(|diagnostic| diagnostic.span.start());
        diagnostics
    }
}
