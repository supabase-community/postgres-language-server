//! dbmate: `-- migrate:up` and `-- migrate:down`, see
//! https://github.com/amacneil/dbmate/blob/v2.37.0/pkg/dbmate/migration.go
//!
//! Its migrations are `<version>_<description>.sql` files in `db/migrations`, unless
//! `--migrations-dir` says otherwise, see
//! https://github.com/amacneil/dbmate/blob/v2.37.0/README.md#creating-migrations

use std::path::Path;

use pgls_fs::{FileSystem, OpenOptions};

use crate::{
    Detected, Direction, FileInfo, MigrationTool, Root, Script, ToolId, Version, detect,
    directives::{self, lines},
    naming,
    units::UnitsBuilder,
};

/// The folder of dbmate in a project, and its migrations in it.
const DBMATE_DIRECTORY: &str = "db";
const DBMATE_MIGRATIONS_DIR: &str = "db/migrations";

pub struct Dbmate;

impl MigrationTool for Dbmate {
    fn id(&self) -> ToolId {
        ToolId::Dbmate
    }

    fn name(&self) -> &'static str {
        "dbmate"
    }

    fn default_dir(&self) -> Option<&'static str> {
        Some(DBMATE_MIGRATIONS_DIR)
    }

    /// A first migration in `db/migrations` with `-- migrate:up`, which dbmate requires in every
    /// migration. dbmate has no configuration file, and reads the database from `DATABASE_URL`,
    /// so the connection is left alone.
    fn detect(&self, fs: &dyn FileSystem, root: &Root) -> Option<Detected> {
        let migrations_dir = root.path(DBMATE_MIGRATIONS_DIR);
        let first = detect::files(&migrations_dir)
            .into_iter()
            .find(|name| is_migration_file_name(name))?;
        let mut contents = String::new();
        fs.open_with_options(
            &migrations_dir.join(first),
            OpenOptions::default().read(true),
        )
        .and_then(|mut file| file.read_to_string(&mut contents))
        .ok()?;
        lines(&contents)
            .any(|(_, line)| marker(line).is_some_and(|(direction, _)| direction == Direction::Up))
            .then(|| Detected::new(self, root, DBMATE_DIRECTORY))
    }

    /// dbmate has no folder of its own to run `init` in.
    fn is_tool_directory(&self, _fs: &dyn FileSystem, _directory: &Path) -> bool {
        false
    }

    fn file(&self, path: &Path, migrations_dir: &Path) -> FileInfo {
        naming::numbered(path, migrations_dir, |text| self.version(text))
    }

    fn version(&self, text: &str) -> Option<Version> {
        naming::number(text)
    }

    fn read(&self, text: &str) -> Script {
        let mut units = UnitsBuilder::new(text);
        for (range, line) in lines(text) {
            if let Some((direction, transaction)) = marker(line) {
                units.start(range.start(), direction, transaction);
            }
        }
        let mut units = units.finish();
        // dbmate sends each section to Postgres as one query.
        for unit in &mut units {
            unit.queries = vec![unit.range];
        }
        Script {
            units,
            look_alikes: directives::look_alikes(
                text,
                |line| marker(line).is_some(),
                |_| "dbmate only reads `-- migrate:up` and `-- migrate:down` in lowercase and at the start of the line, so this line is a plain comment.",
            ),
            before_first_unit: Some(
                "dbmate rejects a migration with statements before the first `-- migrate:up`."
                    .to_string(),
            ),
        }
    }
}

/// `<version>_<description>.sql`, where the version is all leading digits.
fn is_migration_file_name(name: &str) -> bool {
    let description = name.trim_start_matches(|c: char| c.is_ascii_digit());
    description.len() < name.len() && description.starts_with('_') && name.ends_with(".sql")
}

/// `-- migrate:up` or `-- migrate:down` at the start of the line, with options like
/// `transaction:false`. See
/// https://github.com/amacneil/dbmate/blob/v2.37.0/pkg/dbmate/migration.go#L62
fn marker(line: &str) -> Option<(Direction, bool)> {
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

#[cfg(test)]
mod tests {
    use crate::{
        Direction, ToolId,
        tests::{marker_problems, sections},
    };

    #[test]
    fn migration_file_names() {
        assert!(super::is_migration_file_name(
            "20151127184807_create_users.sql"
        ));
        assert!(super::is_migration_file_name("1_init.sql"));
        assert!(!super::is_migration_file_name("_init.sql"));
        assert!(!super::is_migration_file_name("20151127184807.sql"));
        assert!(!super::is_migration_file_name(
            "20151127184807_create_users.sql.bak"
        ));
        assert!(!super::is_migration_file_name("schema.sql"));
    }

    #[test]
    fn dbmate_sections() {
        let text = "-- migrate:up\ncreate table t (id int);\n\n-- migrate:down transaction:false\ndrop table t;\n";
        assert_eq!(
            sections(text, ToolId::Dbmate),
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
            sections(text, ToolId::Dbmate)
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
    fn queries_sent_at_once() {
        let text =
            "-- migrate:up transaction:false\nselect 1;\nselect 2;\n-- migrate:down\nselect 3;\n";
        let queries = ToolId::Dbmate
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
        // dbmate sends each section as one query.
        assert_eq!(
            queries,
            vec![
                vec!["-- migrate:up transaction:false\nselect 1;\nselect 2;\n"],
                vec!["-- migrate:down\nselect 3;\n"],
            ]
        );
    }

    #[test]
    fn statements_before_the_first_marker() {
        let text =
            "create table t (id int);\n-- migrate:up\nselect 1;\n-- migrate:down\nselect 2;\n";
        assert_eq!(
            marker_problems(text, ToolId::Dbmate),
            vec![(
                "create table t (id int);",
                "dbmate rejects a migration with statements before the first `-- migrate:up`."
                    .to_string()
            )]
        );
        // Without markers, the file is one migration.
        assert!(marker_problems("select 1;\n", ToolId::Dbmate).is_empty());
    }

    #[test]
    fn lines_that_look_like_markers() {
        let text = "-- migrate:up\nselect 1;\n-- Migrate:Down\n  -- migrate:down\nselect 2;\n-- migrate:upgrade\n";
        let problems = marker_problems(text, ToolId::Dbmate);
        assert_eq!(
            problems.iter().map(|(line, _)| *line).collect::<Vec<_>>(),
            ["-- Migrate:Down", "  -- migrate:down"]
        );
        assert!(
            problems[0]
                .1
                .starts_with("dbmate only reads `-- migrate:up`")
        );
    }
}
