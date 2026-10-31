//! goose: `-- +goose Up`, `-- +goose Down` and the other `-- +goose` annotations, see
//! https://github.com/pressly/goose/blob/v3.29.0/internal/sqlparser/parser.go

mod interpolate;

use pgls_analyser::{Direction, MigrationSection};
use pgls_lexer::SyntaxKind;
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

    /// Between `-- +goose ENVSUB ON` and `-- +goose ENVSUB OFF`, goose replaces the expansions
    /// of [interpolate], like `$NAME`, `${NAME}` and `${NAME:-default}`, with environment
    /// variables before it runs the SQL, see
    /// https://github.com/pressly/goose/blob/v3.29.0/internal/sqlparser/parser.go#L211
    ///
    /// We don't know their values, so an identifier with expansions, like `${SCHEMA}`,
    /// `${PREFIX}_events` or `"${SCHEMA}"`, becomes the named parameter `$SCHEMA`,
    /// `$PREFIX_events` or `$SCHEMA`, padded with `_` to the same length, so that the whole
    /// identifier is one token with room for the index of a parameter. Named parameters are
    /// analysed like psql variables: as values, or as identifiers that aren't typechecked.
    /// Expansions in string literals and comments stay as written.
    ///
    /// [interpolate]: https://github.com/mfridman/interpolate/blob/v0.0.2/parser.go
    fn preprocess(&self, text: &str) -> Option<Preprocessed> {
        if !text.contains("+goose") {
            return None;
        }
        let mut expansions = Vec::new();
        let mut envsub = false;
        for (range, line) in lines(text) {
            let annotation = annotation(line);
            match annotation {
                Some(annotation) if annotation.eq_ignore_ascii_case("envsub on") => envsub = true,
                Some(annotation) if annotation.eq_ignore_ascii_case("envsub off") => envsub = false,
                _ => {}
            }
            if !envsub || annotation.is_some() {
                continue;
            }
            // goose fails on a line that interpolate rejects, so it stays as written.
            for expansion in interpolate::expansions(line).unwrap_or_default() {
                expansions.push(Expansion {
                    range: TextRange::new(
                        range.start() + TextSize::try_from(expansion.range.start).unwrap(),
                        range.start() + TextSize::try_from(expansion.range.end).unwrap(),
                    ),
                    name: expansion.name,
                });
            }
        }
        (!expansions.is_empty()).then(|| substitute(text, &expansions))
    }
}

/// An `ENVSUB` expansion in the file.
struct Expansion<'a> {
    range: TextRange,
    name: &'a str,
}

/// Replaces the identifiers with `expansions` with named parameters, see [`Goose::preprocess`].
fn substitute(text: &str, expansions: &[Expansion]) -> Preprocessed {
    // Lex the text with each expansion as an identifier of the same length, to find the tokens
    // that the expansions are part of.
    let mut filled = String::with_capacity(text.len());
    let mut end = 0;
    for expansion in expansions {
        let range = std::ops::Range::<usize>::from(expansion.range);
        filled.push_str(&text[end..range.start]);
        filled.extend(std::iter::repeat_n('x', range.len()));
        end = range.end;
    }
    filled.push_str(&text[end..]);
    let lexed = pgls_lexer::lex(&filled);

    // The identifiers with expansions, extended to whole expansions.
    let mut identifiers: Vec<TextRange> = Vec::new();
    for index in 0..lexed.len() {
        if lexed.kind(index) != SyntaxKind::IDENT {
            continue;
        }
        let token = lexed.range(index);
        let first = expansions.partition_point(|expansion| expansion.range.end() <= token.start());
        let Some(range) = expansions[first..]
            .iter()
            .take_while(|expansion| expansion.range.start() < token.end())
            .fold(None, |range: Option<TextRange>, expansion| {
                Some(range.unwrap_or(token).cover(expansion.range))
            })
        else {
            continue;
        };
        match identifiers.last_mut() {
            Some(last) if last.end() > range.start() => *last = last.cover(range),
            _ => identifiers.push(range),
        }
    }

    let mut substituted = String::with_capacity(text.len());
    let mut placeholders = Vec::new();
    let mut end = TextSize::from(0);
    let mut expansions = expansions.iter().peekable();
    for identifier in identifiers {
        substituted.push_str(&text[TextRange::new(end, identifier.start())]);
        // Expansions outside of identifiers stay as written.
        while let Some(expansion) =
            expansions.next_if(|expansion| expansion.range.start() < identifier.start())
        {
            placeholders.push(expansion.range);
        }
        let mut name = String::new();
        let mut written = identifier.start();
        while let Some(expansion) =
            expansions.next_if(|expansion| expansion.range.start() < identifier.end())
        {
            name.extend(name_chars(
                &text[TextRange::new(written, expansion.range.start())],
            ));
            name.extend(name_chars(expansion.name));
            written = expansion.range.end();
        }
        name.extend(name_chars(&text[TextRange::new(written, identifier.end())]));
        if !name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
            name.insert(0, 'e');
        }
        let mut parameter = format!("${name}");
        let len = usize::from(identifier.len());
        parameter.truncate(len);
        substituted.push_str(&parameter);
        substituted.extend(std::iter::repeat_n('_', len - parameter.len()));
        placeholders.push(identifier);
        end = identifier.end();
    }
    substituted.push_str(&text[usize::from(end)..]);
    placeholders.extend(expansions.map(|expansion| expansion.range));

    debug_assert_eq!(substituted.len(), text.len());
    Preprocessed {
        text: substituted,
        placeholders,
    }
}

/// The characters of `text` that a named parameter can hold.
fn name_chars(text: &str) -> impl Iterator<Item = char> + '_ {
    text.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
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
    use pgls_configuration::migrations::SectionMarkers;

    use crate::migrations::{
        migration_tool,
        tests::{marker_problems, sections},
    };

    #[test]
    fn goose_sections() {
        let text = "-- +goose Up\n-- +goose StatementBegin\nselect 1;\n-- +goose StatementEnd\n-- +goose down\nselect 2;";
        assert_eq!(
            sections(text, SectionMarkers::Goose),
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
        let queries = migration_tool(SectionMarkers::Goose)
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
        assert_eq!(marker_problems(text, SectionMarkers::Goose).len(), 1);
    }

    #[test]
    fn lines_that_look_like_markers() {
        let text =
            "-- +goose Up\nselect 1;\n-- +Goose Down\n  -- +goose StatementBegin\nselect 2;\n";
        let problems = marker_problems(text, SectionMarkers::Goose);
        assert_eq!(
            problems.iter().map(|(line, _)| *line).collect::<Vec<_>>(),
            ["-- +Goose Down", "  -- +goose StatementBegin"]
        );
        assert!(problems[0].1.ends_with("so this line is a plain comment."));
        assert!(problems[1].1.starts_with("goose rejects a migration"));
    }

    /// The text that is analysed for `text`, and the parts that goose replaces.
    fn envsub(text: &str) -> Option<(String, Vec<&str>)> {
        let preprocessed = migration_tool(SectionMarkers::Goose)
            .unwrap()
            .preprocess(text)?;
        assert_eq!(preprocessed.text.len(), text.len());
        let placeholders = preprocessed
            .placeholders
            .iter()
            .map(|range| &text[*range])
            .collect();
        Some((preprocessed.text, placeholders))
    }

    #[test]
    fn goose_envsub_replaces_expansions_with_named_parameters() {
        let text = "-- +goose Up\nselect '${A}';\n-- +goose ENVSUB ON\ncreate schema ${SCHEMA};\ngrant usage on schema ${SCHEMA:-${FALLBACK}} to $ROLE, \\${KEPT};\n-- +goose ENVSUB OFF\nselect ${B};\n";
        assert_eq!(
            envsub(text),
            Some((
                "-- +goose Up\nselect '${A}';\n-- +goose ENVSUB ON\ncreate schema $SCHEMA__;\ngrant usage on schema $SCHEMA_______________ to $ROLE, \\${KEPT};\n-- +goose ENVSUB OFF\nselect ${B};\n".to_string(),
                vec!["${SCHEMA}", "${SCHEMA:-${FALLBACK}}", "$ROLE"]
            ))
        );
        assert_eq!(envsub("-- +goose Up\nselect ${A};\n"), None);
        assert_eq!(envsub("select 1;"), None);
    }

    #[test]
    fn goose_envsub_reads_defaults_like_interpolate() {
        // interpolate reads `{` in a default as text, and goose runs `create schema public;`
        // with `SCHEMA=public`.
        assert_eq!(
            envsub("-- +goose Up\n-- +goose ENVSUB ON\ncreate schema ${SCHEMA:-app{test};\n"),
            Some((
                "-- +goose Up\n-- +goose ENVSUB ON\ncreate schema $SCHEMA____________;\n"
                    .to_string(),
                vec!["${SCHEMA:-app{test}"]
            ))
        );
        // interpolate rejects `${A:?}`, so goose fails on the line, and it stays as written.
        assert_eq!(
            envsub("-- +goose Up\n-- +goose ENVSUB ON\nselect ${A:?};\n"),
            None
        );
    }

    #[test]
    fn goose_envsub_replaces_whole_identifiers() {
        // goose runs `create table app_events (id bigint);` with `PREFIX=app`, and
        // `select * from "public".events` with `SCHEMA=public`.
        assert_eq!(
            envsub(
                "-- +goose Up\n-- +goose ENVSUB ON\ncreate table ${PREFIX}_events (id bigint);\nselect * from \"${SCHEMA}\".events, app_$PREFIX.${A}${B};\n"
            ),
            Some((
                "-- +goose Up\n-- +goose ENVSUB ON\ncreate table $PREFIX_events__ (id bigint);\nselect * from $SCHEMA____.events, $app_PREFIX.$AB_____;\n"
                    .to_string(),
                vec![
                    "${PREFIX}_events",
                    "\"${SCHEMA}\"",
                    "app_$PREFIX",
                    "${A}${B}"
                ]
            ))
        );
    }

    #[test]
    fn goose_envsub_keeps_expansions_in_strings_and_comments() {
        let text = "-- +goose Up\n-- +goose ENVSUB ON\ninsert into audit_log (message) values ('${MESSAGE}'); -- ${NOTE}\n";
        assert_eq!(
            envsub(text),
            Some((text.to_string(), vec!["${MESSAGE}", "${NOTE}"]))
        );
    }

    #[test]
    fn goose_no_transaction_applies_to_the_whole_file() {
        let text = "-- +goose Up\nselect 1;\n-- +goose Down\nselect 2;\n-- +goose NO TRANSACTION\n";
        assert!(
            sections(text, SectionMarkers::Goose)
                .iter()
                .all(|(_, _, transaction)| !transaction)
        );
    }
}
