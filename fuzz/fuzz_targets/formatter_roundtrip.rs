//! The formatter on any text that parses as one statement: `format_statement` doesn't panic,
//! doesn't refuse because its output doesn't parse back to the same AST, and its output formats to
//! itself (`FormatResult::formats_to_itself`). The same oracle as the corpus test.
//!
//! An input with a `%` or a `{` is also formatted as a Go printf and a Rust fmt template, through
//! the workspace like `format` does, see [`TemplateFormat`].

#![no_main]

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use pgls_configuration::PartialConfiguration;
use pgls_fs::PgLSPath;
use pgls_pretty_print::{
    FormatConfig, FormatError, FormatStatementParams, NormalizedStatements, format_statement,
};
use pgls_templating::Template;
use pgls_workspace::{
    Workspace,
    features::format::PullFileFormattingParams,
    workspace::{
        OpenFileParams, ProjectKey, RegisterProjectFolderParams, UpdateSettingsParams,
        server::WorkspaceServer,
    },
};

/// Longer inputs mostly cost time; a statement's layout doesn't need them.
const INPUT_LEN_MAX: usize = 2000;

fuzz_target!(|data: &[u8]| {
    let Ok(sql) = std::str::from_utf8(data) else {
        return;
    };
    if sql.len() > INPUT_LEN_MAX {
        return;
    }
    if sql.contains(['%', '{']) {
        TemplateFormat::GO_PRINTF.check(sql);
        TemplateFormat::RUST_FMT.check(sql);
    }
    let Some(ast) = pgls_query::parse(sql).ok().and_then(|x| x.into_root()) else {
        return;
    };
    let config = FormatConfig {
        line_width: 80,
        ..Default::default()
    };
    match format_statement(FormatStatementParams::new(&ast, sql, &config)) {
        Ok(result) => assert!(
            result.formats_to_itself(&config),
            "formatter_roundtrip: formatting isn't idempotent\n{sql}\n---\n{}",
            result.formatted
        ),
        Err(e @ (FormatError::ParseError { .. } | FormatError::BetaUnsupported { .. })) => {
            panic!("formatter_roundtrip: the output doesn't round-trip: {e}\n{sql}")
        }
        Err(_) => {}
    }
});

/// A template syntax, and the workspace that formats its files. An input that has directives of
/// the template and renders to SQL that parses (`Template::render`) is formatted like `format`
/// formats a file of the template, and the output, rendered with the same sample values, must
/// have the same statements (`NormalizedStatements`): the formatter didn't change what the host
/// sends to Postgres. A statement that it can't restore stays as written, which renders the same.
struct TemplateFormat {
    template: Template,
    workspace: &'static OnceLock<(WorkspaceServer, ProjectKey)>,
}

impl TemplateFormat {
    const GO_PRINTF: Self = Self {
        template: Template::GoPrintf,
        workspace: &GO_PRINTF_WORKSPACE,
    };
    const RUST_FMT: Self = Self {
        template: Template::RustFmt,
        workspace: &RUST_FMT_WORKSPACE,
    };

    fn check(&self, sql: &str) {
        if self.template.read(sql).is_none() {
            return;
        }
        let rendered = self.template.render(sql);
        let Some(expected) = NormalizedStatements::of(&rendered) else {
            return;
        };
        // Where the splitter cuts a statement that Postgres reads as one, like before a `SET`
        // after a `;` in a comment, the formatter ends it with a `;` of its own, with or without
        // a template. `splitter_vs_parser` checks that the two agree.
        let statements = pgls_query::parse(&rendered).map_or(0, |parsed| parsed.stmts().len());
        if pgls_statement_splitter::split(&rendered).ranges.len() != statements {
            return;
        }
        let formatted = self.format(sql);
        assert_eq!(
            NormalizedStatements::of(&self.template.render(&formatted)).as_ref(),
            Some(&expected),
            "formatter_roundtrip: formatting the {:?} template changed the program\n{sql}\n---\n{formatted}",
            self.template
        );
    }

    /// `sql` formatted as a file of the template.
    fn format(&self, sql: &str) -> String {
        let (workspace, project_key) = self.workspace.get_or_init(|| {
            let workspace = WorkspaceServer::new();
            let project_key = workspace
                .register_project_folder(RegisterProjectFolderParams {
                    path: None,
                    real_path: None,
                })
                .expect("a project");
            let syntax = match self.template {
                Template::GoPrintf => "go-printf",
                Template::RustFmt => "rust-fmt",
                Template::None => "none",
            };
            let configuration: PartialConfiguration = serde_json::from_str(&format!(
                r#"{{ "db": {{ "disableConnection": true }}, "format": {{ "enabled": true, "lineWidth": 80 }}, "templating": {{ "syntax": "{syntax}" }} }}"#
            ))
            .expect("a configuration");
            workspace
                .update_settings(UpdateSettingsParams {
                    project_key,
                    configuration,
                    gitignore_matches: vec![],
                    vcs_base_path: None,
                    workspace_directory: None,
                    replace: false,
                })
                .expect("settings");
            (workspace, project_key)
        });
        let path = PgLSPath::new("template.sql");
        // Opening the file again replaces its content.
        workspace
            .open_file(OpenFileParams {
                project_key: *project_key,
                path: path.clone(),
                content: sql.to_string(),
                version: 1,
            })
            .expect("an open file");
        workspace
            .pull_file_formatting(PullFileFormattingParams {
                project_key: *project_key,
                path,
                range: None,
            })
            .expect("formatting")
            .formatted
    }
}

static GO_PRINTF_WORKSPACE: OnceLock<(WorkspaceServer, ProjectKey)> = OnceLock::new();
static RUST_FMT_WORKSPACE: OnceLock<(WorkspaceServer, ProjectKey)> = OnceLock::new();
