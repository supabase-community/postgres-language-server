//! dbmate: `-- migrate:up` and `-- migrate:down`, see
//! https://github.com/amacneil/dbmate/blob/v2.37.0/pkg/dbmate/migration.go

use pgls_analyser::{Direction, MigrationSection};
use pgls_text_size::TextRange;

use super::{
    MigrationTool, SectionMarkerDiagnostic,
    markers::{self, MarkerSyntax, SectionsBuilder, lines},
};

pub(super) struct Dbmate;

const SYNTAX: MarkerSyntax = MarkerSyntax {
    tool: "dbmate",
    first_marker: "-- migrate:up",
    is_marker: |line| marker(line).is_some(),
    look_alike_message: |_| "dbmate only reads `-- migrate:up` and `-- migrate:down` in lowercase and at the start of the line, so this line is a plain comment.",
};

impl MigrationTool for Dbmate {
    fn sections(&self, text: &str) -> Vec<MigrationSection> {
        let mut sections = SectionsBuilder::new(text);
        for (range, line) in lines(text) {
            if let Some((direction, transaction)) = marker(line) {
                sections.start(range.start(), direction, transaction);
            }
        }
        let mut sections = sections.finish();
        // dbmate sends each section to Postgres as one query.
        for section in &mut sections {
            section.queries = vec![section.range];
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
    use pgls_analyser::Direction;
    use pgls_configuration::migrations::SectionMarkers;

    use crate::migrations::{
        migration_tool,
        tests::{marker_problems, sections},
    };

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
    fn queries_sent_at_once() {
        let text =
            "-- migrate:up transaction:false\nselect 1;\nselect 2;\n-- migrate:down\nselect 3;\n";
        let queries = migration_tool(SectionMarkers::Dbmate)
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
            marker_problems(text, SectionMarkers::Dbmate),
            vec![(
                "create table t (id int);",
                "dbmate rejects a migration with statements before the first `-- migrate:up`."
                    .to_string()
            )]
        );
        // Without markers, the file is one migration.
        assert!(marker_problems("select 1;\n", SectionMarkers::Dbmate).is_empty());
    }

    #[test]
    fn lines_that_look_like_markers() {
        let text = "-- migrate:up\nselect 1;\n-- Migrate:Down\n  -- migrate:down\nselect 2;\n-- migrate:upgrade\n";
        let problems = marker_problems(text, SectionMarkers::Dbmate);
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
