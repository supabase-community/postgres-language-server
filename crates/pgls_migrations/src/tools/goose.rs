//! goose: `-- +goose Up`, `-- +goose Down` and the other `-- +goose` annotations, see
//! https://github.com/pressly/goose/blob/v3.29.0/internal/sqlparser/parser.go
//!
//! goose has no configuration file and no migrations folder of its own: it runs the migrations
//! in the folder of `-dir` or `GOOSE_MIGRATION_DIR`, `.` by default. So its projects aren't
//! detected.

use std::path::Path;

use pgls_fs::FileSystem;
use pgls_text_size::TextRange;

use crate::{
    Detected, Direction, FileInfo, MigrationTool, Root, Script, ToolId, Version,
    directives::{self, lines},
    naming,
    units::UnitsBuilder,
};

pub struct Goose;

impl MigrationTool for Goose {
    fn id(&self) -> ToolId {
        ToolId::Goose
    }

    fn name(&self) -> &'static str {
        "goose"
    }

    /// goose has no sign of its projects, see above.
    fn detect(&self, _fs: &dyn FileSystem, _root: &Root) -> Option<Detected> {
        None
    }

    fn is_tool_directory(&self, _fs: &dyn FileSystem, _directory: &Path) -> bool {
        false
    }

    fn default_dir(&self) -> Option<&'static str> {
        None
    }

    fn file(&self, path: &Path, migrations_dir: &Path) -> FileInfo {
        naming::numbered(path, migrations_dir, |text| self.version(text))
    }

    fn version(&self, text: &str) -> Option<Version> {
        naming::number(text)
    }

    fn read(&self, text: &str) -> Script {
        let mut sections = UnitsBuilder::new(text);
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

        let mut units = sections.finish();
        // goose runs both directions of the file without a transaction.
        if no_transaction {
            for unit in &mut units {
                unit.transaction = false;
            }
        }
        Script {
            units,
            look_alikes: directives::look_alikes(
                text,
                |line| annotation(line).is_some(),
                |line| {
                    // goose fails on an annotation with leading whitespace instead of ignoring
                    // it, see
                    // https://github.com/pressly/goose/blob/v3.29.0/internal/sqlparser/parser.go#L321
                    if line.contains("+goose") {
                        "goose rejects a migration with an annotation that doesn't start at the beginning of the line."
                    } else {
                        "goose only reads `-- +goose` annotations with `+goose` in lowercase, so this line is a plain comment."
                    }
                },
            ),
            before_first_unit: Some(
                "goose rejects a migration with statements before the first `-- +goose Up`."
                    .to_string(),
            ),
        }
    }
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
    use crate::{
        Direction, ToolId,
        tests::{marker_problems, sections},
    };

    #[test]
    fn goose_sections() {
        let text = "-- +goose Up\n-- +goose StatementBegin\nselect 1;\n-- +goose StatementEnd\n-- +goose down\nselect 2;";
        assert_eq!(
            sections(text, ToolId::Goose),
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
        let queries = ToolId::Goose
            .tool()
            .read(text)
            .units
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
        assert_eq!(marker_problems(text, ToolId::Goose).len(), 1);
    }

    #[test]
    fn lines_that_look_like_markers() {
        let text =
            "-- +goose Up\nselect 1;\n-- +Goose Down\n  -- +goose StatementBegin\nselect 2;\n";
        let problems = marker_problems(text, ToolId::Goose);
        assert_eq!(
            problems.iter().map(|(line, _)| *line).collect::<Vec<_>>(),
            ["-- +Goose Down", "  -- +goose StatementBegin"]
        );
        assert!(problems[0].1.ends_with("so this line is a plain comment."));
        assert!(problems[1].1.starts_with("goose rejects a migration"));
    }

    #[test]
    fn goose_no_transaction_applies_to_the_whole_file() {
        let text = "-- +goose Up\nselect 1;\n-- +goose Down\nselect 2;\n-- +goose NO TRANSACTION\n";
        assert!(
            sections(text, ToolId::Goose)
                .iter()
                .all(|(_, _, transaction)| !transaction)
        );
    }
}
