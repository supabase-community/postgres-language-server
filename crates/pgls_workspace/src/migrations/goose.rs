//! goose: `-- +goose Up`, `-- +goose Down` and the other `-- +goose` annotations, see
//! https://github.com/pressly/goose/blob/v3.29.0/internal/sqlparser/parser.go

use pgls_analyser::{Direction, MigrationSection};
use pgls_text_size::{TextRange, TextSize};

use super::{
    MigrationTool, Preprocessed, SectionMarkerDiagnostic,
    markers::{self, MarkerSyntax, SectionsBuilder, lines},
};

pub(super) struct Goose;

const SYNTAX: MarkerSyntax = MarkerSyntax {
    tool: "goose",
    first_marker: "-- +goose Up",
    is_marker: |line| annotation(line).is_some(),
    look_alike_message: |line| {
        // goose fails on an annotation with leading whitespace instead of ignoring it, see
        // https://github.com/pressly/goose/blob/v3.29.0/internal/sqlparser/parser.go#L321
        if line.contains("+goose") {
            "goose rejects a migration with an annotation that doesn't start at the beginning of the line."
        } else {
            "goose only reads `-- +goose` annotations with `+goose` in lowercase, so this line is a plain comment."
        }
    },
};

impl MigrationTool for Goose {
    fn sections(&self, text: &str) -> Vec<MigrationSection> {
        let mut sections = SectionsBuilder::new(text);
        let mut no_transaction = false;
        // The start of the open `StatementBegin` block.
        let mut statement_begin = None;
        for (range, line) in lines(text) {
            match annotation(line) {
                Some(annotation) if annotation.eq_ignore_ascii_case("up") => {
                    sections.start(range.start(), Direction::Up, true);
                }
                Some(annotation) if annotation.eq_ignore_ascii_case("down") => {
                    sections.start(range.start(), Direction::Down, true);
                }
                Some(annotation) if annotation.eq_ignore_ascii_case("no transaction") => {
                    no_transaction = true;
                }
                Some(annotation) if annotation.eq_ignore_ascii_case("statementbegin") => {
                    statement_begin = Some(range.start());
                }
                Some(annotation) if annotation.eq_ignore_ascii_case("statementend") => {
                    // goose sends the statements between `StatementBegin` and `StatementEnd`
                    // to Postgres as one query.
                    if let (Some(begin), Some(section)) =
                        (statement_begin.take(), sections.current())
                    {
                        section.queries.push(TextRange::new(begin, range.end()));
                    }
                }
                _ => {}
            }
        }

        let mut sections = sections.finish();
        // goose runs both directions of the file without a transaction.
        if no_transaction {
            for section in &mut sections {
                section.transaction = false;
            }
        }
        sections
    }

    fn marker_diagnostics(
        &self,
        text: &str,
        sections: &[MigrationSection],
        statements: &[TextRange],
    ) -> Vec<SectionMarkerDiagnostic> {
        markers::marker_diagnostics(&SYNTAX, text, sections, statements)
    }

    /// Between `-- +goose ENVSUB ON` and `-- +goose ENVSUB OFF`, goose replaces `${NAME}`,
    /// `${NAME:-default}` and the other braced forms of [interpolate] with environment variables
    /// before it runs the SQL, see
    /// https://github.com/pressly/goose/blob/v3.29.0/internal/sqlparser/parser.go#L211
    ///
    /// We don't know their values, so each braced expansion becomes the named parameter `$NAME`,
    /// padded with spaces to the same length. Named parameters are analysed like psql
    /// variables: as values, or as identifiers that aren't typechecked. `$NAME` is a named
    /// parameter already.
    ///
    /// [interpolate]: https://github.com/mfridman/interpolate/blob/v0.0.2/parser.go
    fn preprocess(&self, text: &str) -> Option<Preprocessed> {
        if !text.contains("+goose") {
            return None;
        }
        let mut substituted = String::with_capacity(text.len());
        let mut placeholders = Vec::new();
        let mut envsub = false;
        for (range, line) in lines(text) {
            let annotation = annotation(line);
            match annotation {
                Some(annotation) if annotation.eq_ignore_ascii_case("envsub on") => envsub = true,
                Some(annotation) if annotation.eq_ignore_ascii_case("envsub off") => envsub = false,
                _ => {}
            }
            if envsub && annotation.is_none() {
                substitute_line(
                    &text[range],
                    range.start(),
                    &mut substituted,
                    &mut placeholders,
                );
            } else {
                substituted.push_str(&text[range]);
            }
        }
        debug_assert_eq!(substituted.len(), text.len());
        debug_assert!(placeholders.iter().all(|placeholder| {
            text.get(usize::from(placeholder.start())..usize::from(placeholder.end()))
                .is_some()
        }));
        (!placeholders.is_empty()).then_some(Preprocessed {
            text: substituted,
            placeholders,
        })
    }
}

/// Replaces the braced expansions of a line that starts at `start`, see [`Goose::preprocess`].
fn substitute_line(line: &str, start: TextSize, out: &mut String, expansions: &mut Vec<TextRange>) {
    let mut rest = line;
    while let Some(dollar) = rest.find(['$', '\\']) {
        let (before, from) = rest.split_at(dollar);
        out.push_str(before);
        // `\\`, `\$` and `$$` are escapes, and `$(` is left alone.
        let escape = ["\\\\", "\\$", "$$", "$("]
            .iter()
            .find(|escape| from.starts_with(**escape));
        if let Some(escape) = escape {
            out.push_str(escape);
            rest = &from[escape.len()..];
            continue;
        }
        let Some(body) = from.strip_prefix("${") else {
            out.push_str(&from[..1]);
            rest = &from[1..];
            continue;
        };
        // The expansion ends at the matching brace, defaults can hold expansions too.
        let mut depth = 1;
        let end = body.char_indices().find_map(|(index, c)| {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
            (depth == 0).then_some(index)
        });
        let Some(end) = end else {
            out.push_str(from);
            return;
        };
        let expansion = &from[..end + 3];
        let name: String = body
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        let name = if name.starts_with(|c: char| c.is_alphabetic()) {
            name
        } else {
            "e".to_string()
        };
        let placeholder = format!("${name}");
        out.push_str(&placeholder);
        out.push_str(&" ".repeat(expansion.len().saturating_sub(placeholder.len())));
        let expansion_start = start + TextSize::of(&line[..line.len() - from.len()]);
        expansions.push(TextRange::at(expansion_start, TextSize::of(expansion)));
        rest = &from[expansion.len()..];
    }
    out.push_str(rest);
}

/// The annotation of a `-- +goose <annotation>` line. See
/// https://github.com/pressly/goose/blob/v3.29.0/internal/sqlparser/parser.go#L319
fn annotation(line: &str) -> Option<&str> {
    let annotation = line
        .strip_prefix("--")?
        .trim_start()
        .strip_prefix("+goose")?;
    Some(annotation.trim())
}

#[cfg(test)]
mod tests {
    use pgls_analyser::Direction;
    use pgls_configuration::migrations::MigrationTool::{self as Tool};

    use crate::migrations::{
        migration_tool,
        tests::{marker_problems, sections},
    };

    #[test]
    fn goose_sections() {
        let text = "-- +goose Up\n-- +goose StatementBegin\nselect 1;\n-- +goose StatementEnd\n-- +goose down\nselect 2;";
        assert_eq!(
            sections(text, Tool::Goose),
            vec![
                (
                    "-- +goose Up\n-- +goose StatementBegin\nselect 1;\n-- +goose StatementEnd\n",
                    Direction::Up,
                    true
                ),
                ("-- +goose down\nselect 2;", Direction::Down, true),
            ]
        );
    }

    #[test]
    fn queries_sent_at_once() {
        let text = "-- +goose NO TRANSACTION\n-- +goose Up\nselect 1;\n-- +goose StatementBegin\nselect 2;\nselect 3;\n-- +goose StatementEnd\n-- +goose Down\nselect 4;\n";
        let queries = migration_tool(Tool::Goose)
            .map_or_else(Vec::new, |tool| tool.sections(text))
            .into_iter()
            .map(|section| {
                section
                    .queries
                    .iter()
                    .map(|query| &text[*query])
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        // goose sends a `StatementBegin` block as one query, and the other statements one by one.
        assert_eq!(
            queries,
            vec![
                vec!["-- +goose StatementBegin\nselect 2;\nselect 3;\n-- +goose StatementEnd\n"],
                vec![],
            ]
        );
    }

    #[test]
    fn statements_before_the_first_marker() {
        let text = "select 1;\n-- +goose Up\nselect 2;\n";
        assert_eq!(marker_problems(text, Tool::Goose).len(), 1);
    }

    #[test]
    fn lines_that_look_like_markers() {
        let text =
            "-- +goose Up\nselect 1;\n-- +Goose Down\n  -- +goose StatementBegin\nselect 2;\n";
        let problems = marker_problems(text, Tool::Goose);
        assert_eq!(
            problems.iter().map(|(line, _)| *line).collect::<Vec<_>>(),
            ["-- +Goose Down", "  -- +goose StatementBegin"]
        );
        assert!(problems[0].1.ends_with("so this line is a plain comment."));
        assert!(problems[1].1.starts_with("goose rejects a migration"));
    }

    #[test]
    fn goose_envsub_replaces_braced_expansions_with_named_parameters() {
        let goose = migration_tool(Tool::Goose).unwrap();
        let text = "-- +goose Up\nselect '${A}';\n-- +goose ENVSUB ON\ncreate schema ${SCHEMA};\ngrant usage on schema ${SCHEMA:-${FALLBACK}} to $ROLE, \\${KEPT}, $${KEPT};\n-- +goose ENVSUB OFF\nselect ${B};\n";
        let preprocessed = goose.preprocess(text).unwrap();
        assert_eq!(preprocessed.text.len(), text.len());
        assert_eq!(
            preprocessed.text,
            "-- +goose Up\nselect '${A}';\n-- +goose ENVSUB ON\ncreate schema $SCHEMA  ;\ngrant usage on schema $SCHEMA                to $ROLE, \\${KEPT}, $${KEPT};\n-- +goose ENVSUB OFF\nselect ${B};\n"
        );
        assert_eq!(
            preprocessed
                .placeholders
                .iter()
                .map(|range| &text[*range])
                .collect::<Vec<_>>(),
            ["${SCHEMA}", "${SCHEMA:-${FALLBACK}}"]
        );
        assert!(goose.preprocess("-- +goose Up\nselect ${A};\n").is_none());
        assert!(goose.preprocess("select 1;").is_none());
    }

    #[test]
    fn goose_no_transaction_applies_to_the_whole_file() {
        let text = "-- +goose Up\nselect 1;\n-- +goose Down\nselect 2;\n-- +goose NO TRANSACTION\n";
        assert!(
            sections(text, Tool::Goose)
                .iter()
                .all(|(_, _, transaction)| !transaction)
        );
    }
}
