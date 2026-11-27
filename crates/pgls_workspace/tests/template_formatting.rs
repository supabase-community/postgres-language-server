//! Formatting the files of a host, like Go printf and Rust fmt templates and goose's `ENVSUB`,
//! through the workspace, like `format` does: one snapshot per fixture in `tests/data/<syntax>/`
//! and line width, with the diagnostics of the formatter. For every fixture, the output
//! formats to itself, has the placeholders of the input in the same order, and is the same
//! program to the host: both texts, with the same sample values written in place of the
//! placeholders, parse to the same tree. A statement that the formatter can't restore stays as
//! written. A new fixture needs no new code.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

use std::fmt::Write;

use dir_test::{Fixture, dir_test};
use insta::{assert_snapshot, with_settings};
use pgls_configuration::PartialConfiguration;
use pgls_fs::PgLSPath;
use pgls_migrations::MigrationTool;
use pgls_templating::{Preprocessed, Template};
use pgls_text_size::TextRange;
use pgls_workspace::{
    Workspace,
    features::format::PullFileFormattingParams,
    workspace::{
        OpenFileParams, RegisterProjectFolderParams, UpdateSettingsParams, server::WorkspaceServer,
    },
};

/// The line widths of every fixture, like the fixtures of the printer.
const LINE_WIDTHS: [usize; 2] = [80, 100];

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/data/go-printf/", glob: "*.sql")]
fn go_printf(fixture: Fixture<&str>) {
    FormatFixture::check(&fixture, Syntax::GoPrintf);
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/data/rust-fmt/", glob: "*.sql")]
fn rust_fmt(fixture: Fixture<&str>) {
    FormatFixture::check(&fixture, Syntax::RustFmt);
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/data/envsub/", glob: "*.sql")]
fn envsub(fixture: Fixture<&str>) {
    FormatFixture::check(&fixture, Syntax::Envsub);
}

/// The host of a folder of fixtures.
#[derive(Clone, Copy)]
enum Syntax {
    GoPrintf,
    RustFmt,
    /// goose's `ENVSUB`, so its fixtures have the markers of goose.
    Envsub,
}

impl Syntax {
    fn folder(self) -> &'static str {
        match self {
            Self::GoPrintf => "go-printf",
            Self::RustFmt => "rust-fmt",
            Self::Envsub => "envsub",
        }
    }

    /// The configuration that makes a file a file of the host, formatted at `width`.
    fn configuration(self, width: usize) -> PartialConfiguration {
        let host = match self {
            Self::GoPrintf => r#""templating": { "syntax": "go-printf" }"#,
            Self::RustFmt => r#""templating": { "syntax": "rust-fmt" }"#,
            Self::Envsub => r#""migrations": { "tool": "goose" }"#,
        };
        serde_json::from_str(&format!(
            r#"{{ "db": {{ "disableConnection": true }}, "format": {{ "enabled": true, "lineWidth": {width} }}, {host} }}"#
        ))
        .unwrap()
    }

    fn read(self, text: &str) -> Option<Preprocessed> {
        match self {
            Self::GoPrintf => Template::GoPrintf.read(text),
            Self::RustFmt => Template::RustFmt.read(text),
            Self::Envsub => MigrationTool::Goose.read(text).preprocessed,
        }
    }

    /// `text` as the host would write it, with sample values.
    fn render(self, text: &str) -> String {
        match self {
            Self::GoPrintf => Template::GoPrintf.render(text),
            Self::RustFmt => Template::RustFmt.render(text),
            Self::Envsub => MigrationTool::Goose.render(text),
        }
    }

    /// The text of each placeholder of `text`, in order, also those in strings and comments.
    fn placeholders(self, text: &str) -> Vec<String> {
        self.read(text)
            .map(|preprocessed| {
                preprocessed
                    .placeholders
                    .iter()
                    .map(|placeholder| text[*placeholder].to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The normalized trees of the statements of `text` as the host would write it, like the
    /// formatter compares them.
    fn rendered_trees(self, text: &str) -> Vec<pgls_query::NodeEnum> {
        let rendered = self.render(text);
        let parsed = pgls_query::parse(&rendered)
            .unwrap_or_else(|error| panic!("{rendered}\ndoesn't parse: {error}"));
        parsed
            .stmts()
            .into_iter()
            .map(|statement| {
                let mut statement = statement.clone();
                pgls_pretty_print::normalize_ast(&mut statement);
                statement
            })
            .collect()
    }

    /// `text` formatted at `width` through the workspace.
    fn format(self, width: usize, text: &str) -> Formatted {
        let workspace = WorkspaceServer::new();
        let project_key = workspace
            .register_project_folder(RegisterProjectFolderParams {
                path: None,
                real_path: None,
            })
            .unwrap();
        workspace
            .update_settings(UpdateSettingsParams {
                project_key,
                configuration: self.configuration(width),
                gitignore_matches: vec![],
                vcs_base_path: None,
                workspace_directory: None,
                replace: false,
            })
            .unwrap();
        let path = PgLSPath::new("fixture.sql");
        workspace
            .open_file(OpenFileParams {
                project_key,
                path: path.clone(),
                content: text.to_string(),
                version: 1,
            })
            .unwrap();
        let result = workspace
            .pull_file_formatting(PullFileFormattingParams {
                project_key,
                path,
                range: None,
            })
            .unwrap();
        Formatted {
            output: result.formatted,
            diagnostics: result
                .diagnostics
                .iter()
                .map(|diagnostic| (diagnostic.span(), diagnostic.description_text().to_string()))
                .collect(),
        }
    }
}

/// What the formatter made of a text.
struct Formatted {
    output: String,
    /// The span and the description of each diagnostic.
    diagnostics: Vec<(Option<TextRange>, String)>,
}

impl Formatted {
    /// The descriptions of the diagnostics.
    fn messages(&self) -> impl Iterator<Item = &str> {
        self.diagnostics.iter().map(|(_, message)| message.as_str())
    }
}

/// A fixture of a host.
struct FormatFixture<'a> {
    syntax: Syntax,
    name: &'a str,
    input: &'a str,
}

impl<'a> FormatFixture<'a> {
    /// Formats the fixture at every line width, checks the oracles and snapshots the output.
    fn check(fixture: &'a Fixture<&'a str>, syntax: Syntax) {
        let fixture = Self {
            syntax,
            name: std::path::Path::new(fixture.path())
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap(),
            input: fixture.content(),
        };
        let placeholders = syntax.placeholders(fixture.input);
        let trees = syntax.rendered_trees(fixture.input);
        for width in LINE_WIDTHS {
            fixture.check_width(width, &placeholders, &trees);
        }
    }

    /// Formats the fixture at `width`, given the placeholders of the input and its trees as the
    /// host would write it.
    fn check_width(&self, width: usize, placeholders: &[String], trees: &[pgls_query::NodeEnum]) {
        let Self {
            syntax,
            name,
            input,
        } = *self;
        let formatted = syntax.format(width, input);
        let output = &formatted.output;

        // The output formats to itself, with the same diagnostics, which may have moved.
        let again = syntax.format(width, output);
        assert_eq!(&again.output, output, "{name} at {width} isn't idempotent");
        assert_eq!(
            again.messages().collect::<Vec<_>>(),
            formatted.messages().collect::<Vec<_>>(),
            "{name} at {width}: the diagnostics changed"
        );
        // The placeholders, also those in strings and comments, are those of the input, in the
        // same order: Go and Rust take their arguments by position.
        assert_eq!(
            syntax.placeholders(output),
            placeholders,
            "{name} at {width}: the placeholders changed"
        );
        // The host sends the same program to Postgres.
        assert_eq!(
            syntax.rendered_trees(output),
            trees,
            "{name} at {width}: the rendered program changed"
        );
        // A statement that couldn't be restored stays as written.
        for (span, message) in &formatted.diagnostics {
            if message.contains("could not be restored") {
                let statement = &input[span.expect("a span")];
                assert!(
                    output.contains(statement),
                    "{name} at {width}: `{statement}` changed"
                );
            }
        }

        let mut snapshot = output.clone();
        if !formatted.diagnostics.is_empty() {
            snapshot.push_str("\n---\ndiagnostics:\n");
        }
        for (span, message) in &formatted.diagnostics {
            let at = span.map_or(String::new(), |span| format!("`{}`: ", &input[span]));
            writeln!(snapshot, "  {at}{message}").unwrap();
        }
        with_settings!({
            omit_expression => true,
            snapshot_path => format!("snapshots/{}", syntax.folder()),
            prepend_module_to_snapshot => false,
        }, {
            assert_snapshot!(format!("{name}_{width}"), snapshot);
        });
    }
}
