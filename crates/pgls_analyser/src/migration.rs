//! Migration sections. Tools like dbmate and goose put the up and the down migration in the same
//! file, separated by marker comments, and run each of them on its own.

use pgls_catalog::TransactionMode;
use pgls_text_size::{TextRange, TextSize};

/// The marker comments that split a migration file into sections.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SectionMarkers {
    /// The whole file is one migration.
    #[default]
    None,
    /// [dbmate](https://github.com/amacneil/dbmate): `-- migrate:up` and `-- migrate:down`,
    /// optionally followed by `transaction:false`.
    Dbmate,
    /// [goose](https://github.com/pressly/goose): `-- +goose Up` and `-- +goose Down`, and
    /// `-- +goose NO TRANSACTION` for the whole file.
    Goose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

/// A part of a migration file that the migration tool runs on its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationSection {
    /// From the marker to the next marker, or to the end of the file.
    pub range: TextRange,
    pub direction: Direction,
    /// Whether the migration tool runs the section in a transaction.
    pub transaction: bool,
}

impl MigrationSection {
    pub fn transaction_mode(&self) -> TransactionMode {
        if self.transaction {
            TransactionMode::Transaction
        } else {
            TransactionMode::Autocommit
        }
    }
}

/// The sections of a migration file, in file order. Without markers, there are none.
///
/// Markers are found line by line, like the migration tools do, without looking at the SQL.
pub fn migration_sections(text: &str, markers: SectionMarkers) -> Vec<MigrationSection> {
    let mut sections: Vec<MigrationSection> = Vec::new();
    let mut no_transaction = false;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let start = TextSize::from(offset as u32);
        offset += line.len();
        let line = line.trim_end_matches(['\n', '\r']);

        let marker = match markers {
            SectionMarkers::None => None,
            SectionMarkers::Dbmate => dbmate_marker(line),
            SectionMarkers::Goose => match goose_annotation(line) {
                Some(annotation) if annotation.eq_ignore_ascii_case("up") => {
                    Some((Direction::Up, true))
                }
                Some(annotation) if annotation.eq_ignore_ascii_case("down") => {
                    Some((Direction::Down, true))
                }
                Some(annotation) if annotation.eq_ignore_ascii_case("no transaction") => {
                    no_transaction = true;
                    None
                }
                _ => None,
            },
        };

        if let Some((direction, transaction)) = marker {
            if let Some(previous) = sections.last_mut() {
                previous.range = TextRange::new(previous.range.start(), start);
            }
            sections.push(MigrationSection {
                range: TextRange::new(start, TextSize::from(text.len() as u32)),
                direction,
                transaction,
            });
        }
    }

    // goose runs both directions of the file without a transaction.
    if no_transaction {
        for section in &mut sections {
            section.transaction = false;
        }
    }

    sections
}

/// `-- migrate:up` or `-- migrate:down` at the start of the line, with options like
/// `transaction:false`. See
/// https://github.com/amacneil/dbmate/blob/v2.37.0/pkg/dbmate/migration.go#L62
fn dbmate_marker(line: &str) -> Option<(Direction, bool)> {
    let directive = line
        .strip_prefix("--")?
        .trim_start()
        .strip_prefix("migrate:")?;
    let (direction, options) = if let Some(options) = directive.strip_prefix("up") {
        (Direction::Up, options)
    } else {
        (Direction::Down, directive.strip_prefix("down")?)
    };
    if !options.is_empty() && !options.starts_with(char::is_whitespace) {
        return None;
    }
    let transaction = options
        .split_whitespace()
        .filter_map(|option| option.split_once(':'))
        .filter(|(_, value)| !value.contains(':'))
        .all(|(key, value)| key != "transaction" || value != "false");
    Some((direction, transaction))
}

/// The annotation of a `-- +goose <annotation>` line. See
/// https://github.com/pressly/goose/blob/v3.29.0/internal/sqlparser/parser.go#L319
fn goose_annotation(line: &str) -> Option<&str> {
    let annotation = line
        .strip_prefix("--")?
        .trim_start()
        .strip_prefix("+goose")?;
    Some(annotation.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sections(text: &str, markers: SectionMarkers) -> Vec<(&str, Direction, bool)> {
        migration_sections(text, markers)
            .into_iter()
            .map(|section| (&text[section.range], section.direction, section.transaction))
            .collect()
    }

    #[test]
    fn dbmate_sections() {
        let text = "-- migrate:up\ncreate table t (id int);\n\n-- migrate:down transaction:false\ndrop table t;\n";
        assert_eq!(
            sections(text, SectionMarkers::Dbmate),
            vec![
                (
                    "-- migrate:up\ncreate table t (id int);\n\n",
                    Direction::Up,
                    true
                ),
                (
                    "-- migrate:down transaction:false\ndrop table t;\n",
                    Direction::Down,
                    false
                ),
            ]
        );
    }

    #[test]
    fn dbmate_options_and_spacing() {
        let directions = |text| {
            sections(text, SectionMarkers::Dbmate)
                .into_iter()
                .map(|(_, direction, transaction)| (direction, transaction))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            directions("--migrate:up foo:bar transaction:false\r\nselect 1;"),
            vec![(Direction::Up, false)]
        );
        assert_eq!(
            directions("-- migrate:up transaction:true"),
            vec![(Direction::Up, true)]
        );
        // Not markers: indented, a longer word, or a different tool.
        assert!(directions(" -- migrate:up\n-- migrate:upgrade\n-- +goose Up\n").is_empty());
    }

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
    fn goose_no_transaction_applies_to_the_whole_file() {
        let text = "-- +goose Up\nselect 1;\n-- +goose Down\nselect 2;\n-- +goose NO TRANSACTION\n";
        assert!(
            sections(text, SectionMarkers::Goose)
                .iter()
                .all(|(_, _, transaction)| !transaction)
        );
    }

    #[test]
    fn no_markers() {
        let text = "-- migrate:up\nselect 1;\n-- +goose Down\n";
        assert!(sections(text, SectionMarkers::None).is_empty());
        assert!(sections("select 1;", SectionMarkers::Dbmate).is_empty());
    }
}
