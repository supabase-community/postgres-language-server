use std::path::Path;

use pgls_analyser::{Direction, MigrationSection};
use pgls_configuration::migrations::SectionMarkers;
use pgls_text_size::{TextRange, TextSize};

#[derive(Debug)]
pub(crate) struct Migration {
    pub(crate) sequence_number: u64,
    #[allow(unused)]
    pub(crate) name: String,
}

/// Whether the path is inside the migrations directory.
pub(crate) fn is_in_migrations_dir(path: &Path, migrations_dir: &Path) -> bool {
    if path.starts_with(migrations_dir) {
        return true;
    }
    path.canonicalize()
        .ok()
        .and_then(|canonical_child| {
            migrations_dir
                .canonicalize()
                .ok()
                .map(|canonical_dir| canonical_child.starts_with(&canonical_dir))
        })
        .unwrap_or(false)
}

/// Get the migration associated with a path, if it is a migration file
pub(crate) fn get_migration(path: &Path, migrations_dir: &Path) -> Option<Migration> {
    if !is_in_migrations_dir(path, migrations_dir) {
        return None;
    }

    // we are trying to match patterns used by popular migration tools

    // in the "root" pattern, all files are directly within the migrations directory
    // and their names follow <timestamp>_<name>.sql.
    // this is used by supabase
    let root_migration = path
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .and_then(parse_migration_name);

    if root_migration.is_some() {
        return root_migration;
    }

    // in the "subdirectory" pattern, each migration is in a subdirectory named <timestamp>_<name>
    // this is used by prisma and drizzle
    path.parent()
        .and_then(|parent| parent.file_name())
        .and_then(|os_str| os_str.to_str())
        .and_then(parse_migration_name)
}

fn parse_migration_name(name: &str) -> Option<Migration> {
    let mut parts = name.splitn(2, '_');
    // remove leading zeros to support numeric
    let sequence_number: u64 = parts.next()?.trim_start_matches('0').parse().ok()?;
    let full_name = parts.next()?;
    let name = full_name
        .strip_suffix(".sql")
        .unwrap_or(full_name)
        .to_string();
    Some(Migration {
        sequence_number,
        name,
    })
}

/// The sections of a migration file, in file order. Without markers, there are none.
///
/// Markers are found line by line, like the migration tools do, without looking at the SQL.
pub(crate) fn migration_sections(text: &str, markers: SectionMarkers) -> Vec<MigrationSection> {
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

/// The text that the analysis reads for a file with goose's `-- +goose ENVSUB ON`, and the
/// ranges of the replaced expansions, or `None` if nothing is replaced.
///
/// Between `ENVSUB ON` and `ENVSUB OFF`, goose replaces `${NAME}`, `${NAME:-default}` and the
/// other braced forms of [interpolate] with environment variables before it runs the SQL. We
/// don't know their values, so each braced expansion becomes the named parameter `$NAME`,
/// padded with spaces to the same length so that all offsets stay the same. Named parameters
/// are analysed like psql variables: as values, or as identifiers that aren't typechecked.
/// `$NAME` is a named parameter already. See
/// https://github.com/pressly/goose/blob/v3.29.0/internal/sqlparser/parser.go#L211
///
/// [interpolate]: https://github.com/mfridman/interpolate/blob/v0.0.2/parser.go
pub(crate) fn goose_envsub(text: &str) -> Option<(String, Vec<TextRange>)> {
    if !text.contains("+goose") {
        return None;
    }
    let mut substituted = String::with_capacity(text.len());
    let mut expansions = Vec::new();
    let mut envsub = false;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let annotation = goose_annotation(line.trim_end_matches(['\n', '\r']));
        match annotation {
            Some(annotation) if annotation.eq_ignore_ascii_case("envsub on") => envsub = true,
            Some(annotation) if annotation.eq_ignore_ascii_case("envsub off") => envsub = false,
            _ if envsub && annotation.is_none() => {
                substitute_line(line, start, &mut substituted, &mut expansions);
                continue;
            }
            _ => {}
        }
        substituted.push_str(line);
    }
    (!expansions.is_empty()).then_some((substituted, expansions))
}

/// Replaces the braced expansions of a line, see [`goose_envsub`].
fn substitute_line(line: &str, start: usize, out: &mut String, expansions: &mut Vec<TextRange>) {
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
        let expansion_start = start + (line.len() - from.len());
        expansions.push(TextRange::at(
            TextSize::from(expansion_start as u32),
            TextSize::of(expansion),
        ));
        rest = &from[expansion.len()..];
    }
    out.push_str(rest);
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
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    #[test]
    fn test_get_migration_root_pattern() {
        let temp_dir = setup();
        let migrations_dir = temp_dir.path().to_path_buf();
        let path = migrations_dir.join("1234567890_create_users.sql");
        fs::write(&path, "").unwrap();

        let migration = get_migration(&path, &migrations_dir);

        assert!(migration.is_some());
        let migration = migration.unwrap();
        assert_eq!(migration.sequence_number, 1234567890);
        assert_eq!(migration.name, "create_users");
    }

    #[test]
    fn test_get_migration_subdirectory_pattern() {
        let temp_dir = setup();
        let migrations_dir = temp_dir.path().to_path_buf();
        let subdir = migrations_dir.join("1234567890_create_users");
        fs::create_dir(&subdir).unwrap();
        let path = subdir.join("up.sql");
        fs::write(&path, "").unwrap();

        let migration = get_migration(&path, &migrations_dir);

        assert!(migration.is_some());
        let migration = migration.unwrap();
        assert_eq!(migration.sequence_number, 1234567890);
        assert_eq!(migration.name, "create_users");
    }

    #[test]
    fn test_get_migration_prefix_number() {
        let temp_dir = setup();
        let migrations_dir = temp_dir.path().to_path_buf();
        let path = migrations_dir.join("000201_a_migration.sql");
        fs::write(&path, "").unwrap();

        let migration = get_migration(&path, &migrations_dir);

        assert!(migration.is_some());
        let migration = migration.unwrap();
        assert_eq!(migration.sequence_number, 201);
        assert_eq!(migration.name, "a_migration");
    }

    #[test]
    fn test_get_migration_not_timestamp_in_filename() {
        let migrations_dir = PathBuf::from("/tmp/migrations");
        let path = migrations_dir.join("not_a_migration.sql");

        let migration = get_migration(&path, &migrations_dir);

        assert!(migration.is_none());
    }

    #[test]
    fn test_get_migration_outside_migrations_dir() {
        let migrations_dir = PathBuf::from("/tmp/migrations");
        let path = PathBuf::from("/tmp/other/1234567890_create_users.sql");

        let migration = get_migration(&path, &migrations_dir);

        assert!(migration.is_none());
    }

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
    fn goose_envsub_replaces_braced_expansions_with_named_parameters() {
        let text = "-- +goose Up\nselect '${A}';\n-- +goose ENVSUB ON\ncreate schema ${SCHEMA};\ngrant usage on schema ${SCHEMA:-${FALLBACK}} to $ROLE, \\${KEPT}, $${KEPT};\n-- +goose ENVSUB OFF\nselect ${B};\n";
        let (substituted, expansions) = goose_envsub(text).unwrap();
        assert_eq!(substituted.len(), text.len());
        assert_eq!(
            substituted,
            "-- +goose Up\nselect '${A}';\n-- +goose ENVSUB ON\ncreate schema $SCHEMA  ;\ngrant usage on schema $SCHEMA                to $ROLE, \\${KEPT}, $${KEPT};\n-- +goose ENVSUB OFF\nselect ${B};\n"
        );
        assert_eq!(
            expansions
                .iter()
                .map(|range| &text[*range])
                .collect::<Vec<_>>(),
            ["${SCHEMA}", "${SCHEMA:-${FALLBACK}}"]
        );
        assert!(goose_envsub("-- +goose Up\nselect ${A};\n").is_none());
        assert!(goose_envsub("select 1;").is_none());
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
