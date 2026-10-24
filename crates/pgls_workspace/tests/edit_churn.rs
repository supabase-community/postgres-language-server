//! The caches of a document during a long editing session: they follow the statements that the
//! document has now, and don't grow with the number of edits.
//!
//! Counts every allocation of the test binary with the allocator in `common`, so the test runs
//! on its own in this file. `CHURN_STATEMENTS` and `CHURN_EDITS` change the size of the workload.

mod common;

use common::{current, mib};
use pgls_analyse::RuleCategories;
use pgls_configuration::{Merge, PartialConfiguration, database::PartialDatabaseConfiguration};
use pgls_fs::PgLSPath;
use pgls_text_size::{TextRange, TextSize};
use pgls_workspace::{
    Workspace,
    features::{
        code_actions::CodeActionsParams, diagnostics::PullFileDiagnosticsParams,
        inlay_hints::PullInlayHintsParams, on_hover::OnHoverParams,
    },
    workspace::{
        ChangeFileParams, CloseFileParams, OpenFileParams, RegisterProjectFolderParams,
        UpdateSettingsParams, server::WorkspaceServer,
    },
};
use sqlx::PgPool;

/// The statement edits applied by each change of the file.
const EDITS_PER_CHANGE: usize = 4;

fn env(name: &str, default: usize) -> usize {
    std::env::var(name).map_or(default, |value| value.parse().unwrap())
}

/// A small deterministic random number generator (xorshift), so that a failure reproduces.
struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Table,
    Index,
    Function,
    Query,
}

/// A statement of the document. `id` makes its text unique, and `version` changes the body of a
/// SQL function.
#[derive(Clone, Copy)]
struct Statement {
    kind: Kind,
    id: usize,
    version: usize,
}

impl Statement {
    fn sql(&self) -> String {
        let Self { id, version, .. } = self;
        match self.kind {
            Kind::Table => format!(
                "create table t{id} (id bigint primary key, name text not null, created_at timestamp);"
            ),
            Kind::Index => format!("create index t{id}_name_idx on t{id} (name);"),
            Kind::Function => format!(
                "create function f{id}(a int, b text) returns int language sql stable as $$ select count(*)::int + {version} from pg_class where relname = b and relnatts = a $$;"
            ),
            Kind::Query => format!(
                "select c.relname, f{id}(1, 'x') from pg_class c where c.relnatts = {id} order by c.relname;"
            ),
        }
    }
}

struct Document {
    statements: Vec<Statement>,
    next_id: usize,
}

impl Document {
    fn new(len: usize) -> Self {
        let mut document = Self {
            statements: Vec::new(),
            next_id: 0,
        };
        for _ in 0..len {
            let statement = document.new_statement(document.statements.len());
            document.statements.push(statement);
        }
        document
    }

    fn new_statement(&mut self, seed: usize) -> Statement {
        self.next_id += 1;
        let kind = match seed % 4 {
            0 => Kind::Table,
            1 => Kind::Index,
            2 => Kind::Function,
            _ => Kind::Query,
        };
        Statement {
            kind,
            id: self.next_id,
            version: 0,
        }
    }

    /// The text, and the offset of each statement in it.
    fn render(&self) -> (String, Vec<usize>) {
        let mut sql = String::new();
        let mut offsets = Vec::new();
        for statement in &self.statements {
            offsets.push(sql.len());
            sql.push_str(&statement.sql());
            sql.push('\n');
        }
        (sql, offsets)
    }

    /// Rewrites, inserts or deletes a statement, or changes the body of a SQL function, and keeps
    /// the length of the document within 10% of `len`.
    fn edit(&mut self, rng: &mut Rng, len: usize) {
        let at = rng.below(self.statements.len());
        let operation = if self.statements.len() < len * 9 / 10 {
            1
        } else if self.statements.len() > len * 11 / 10 {
            2
        } else {
            rng.below(4)
        };
        match operation {
            0 => self.statements[at] = self.new_statement(rng.below(4)),
            1 => {
                let statement = self.new_statement(rng.below(4));
                self.statements.insert(at, statement);
            }
            2 => {
                self.statements.remove(at);
            }
            _ => {
                let function = (at..self.statements.len())
                    .chain(0..at)
                    .find(|i| matches!(self.statements[*i].kind, Kind::Function));
                if let Some(function) = function {
                    self.statements[function].version += 1;
                }
            }
        }
    }
}

fn workspace(test_db: &PgPool) -> WorkspaceServer {
    let mut conf = PartialConfiguration::init();
    conf.merge_with(PartialConfiguration {
        db: Some(PartialDatabaseConfiguration {
            database: Some(
                test_db
                    .connect_options()
                    .get_database()
                    .unwrap()
                    .to_string(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    });
    let workspace = WorkspaceServer::new();
    workspace
        .register_project_folder(RegisterProjectFolderParams {
            path: None,
            set_as_current_workspace: true,
        })
        .unwrap();
    workspace
        .update_settings(UpdateSettingsParams {
            configuration: conf,
            gitignore_matches: vec![],
            vcs_base_path: None,
            workspace_directory: None,
        })
        .unwrap();
    workspace
}

fn diagnostics_and_code_actions(workspace: &WorkspaceServer, path: &PgLSPath, at: TextSize) {
    workspace
        .pull_file_diagnostics(PullFileDiagnosticsParams {
            path: path.clone(),
            categories: RuleCategories::all(),
            max_diagnostics: u32::MAX,
            only: vec![],
            skip: vec![],
        })
        .unwrap();
    workspace
        .pull_code_actions(CodeActionsParams {
            path: path.clone(),
            cursor_position: at,
            only: vec![],
            skip: vec![],
        })
        .unwrap();
}

/// Hover on the statement at `offset`, and inlay hints around it.
fn hover_and_inlay_hints(workspace: &WorkspaceServer, path: &PgLSPath, sql: &str, offset: usize) {
    let at = TextSize::try_from(offset + 9).unwrap();
    workspace
        .on_hover(OnHoverParams {
            path: path.clone(),
            position: at,
        })
        .unwrap();
    let end = TextSize::try_from((offset + 2_000).min(sql.len())).unwrap();
    workspace
        .pull_inlay_hints(PullInlayHintsParams {
            path: path.clone(),
            range: TextRange::new(TextSize::try_from(offset).unwrap(), end),
        })
        .unwrap();
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn caches_follow_the_document_through_many_edits(test_db: PgPool) {
    let len = env("CHURN_STATEMENTS", 1_000);
    let edits = env("CHURN_EDITS", 2_000);
    let workspace = workspace(&test_db);
    let path = PgLSPath::new("churn.sql");
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    let mut document = Document::new(len);

    // A full pass over the document as it is opened.
    let baseline = current();
    let (sql, offsets) = document.render();
    workspace
        .open_file(OpenFileParams {
            path: path.clone(),
            content: sql.clone(),
            version: 1,
        })
        .unwrap();
    let last = *offsets.last().unwrap();
    diagnostics_and_code_actions(&workspace, &path, TextSize::try_from(last + 9).unwrap());
    hover_and_inlay_hints(&workspace, &path, &sql, last);
    let first_pass = current() - baseline;

    // Each change of the file applies a few edits, like typing a few characters between two
    // requests: every change re-splits the whole document, which takes most of the time of a
    // debug build.
    let changes = edits / EDITS_PER_CHANGE;
    for change in 0..changes {
        for _ in 0..EDITS_PER_CHANGE {
            document.edit(&mut rng, len);
        }
        let (sql, offsets) = document.render();
        workspace
            .change_file(ChangeFileParams {
                path: path.clone(),
                content: sql.clone(),
                version: i32::try_from(change).unwrap() + 2,
            })
            .unwrap();
        let offset = offsets[rng.below(offsets.len())];
        if change % 5 == 0 {
            hover_and_inlay_hints(&workspace, &path, &sql, offset);
        }
        if change % 50 == 0 {
            diagnostics_and_code_actions(
                &workspace,
                &path,
                TextSize::try_from(offset + 9).unwrap(),
            );
        }
    }

    // The same full pass at the end.
    let (sql, offsets) = document.render();
    let last = *offsets.last().unwrap();
    diagnostics_and_code_actions(&workspace, &path, TextSize::try_from(last + 9).unwrap());
    hover_and_inlay_hints(&workspace, &path, &sql, last);
    let after_edits = current() - baseline;

    let stats = workspace.document_cache_stats(&path).unwrap();
    println!(
        "{len} statements, {edits} edits in {changes} changes: retained {:.1} MiB after the first pass, {:.1} MiB after \
         the edits; {stats:?}",
        mib(first_pass),
        mib(after_edits)
    );

    // Every statement of the document, including the function bodies, has a parse result and a
    // tree, and nothing else is cached.
    assert_eq!(stats.parse_results, stats.statements, "{stats:?}");
    assert_eq!(stats.trees, stats.statements, "{stats:?}");
    assert!(stats.analysis);
    // The document stays within 10% of its length, so the memory doesn't grow with the edits.
    assert!(
        after_edits <= first_pass * 3 / 2,
        "{:.1} MiB retained after {edits} edits, {:.1} MiB after the first pass",
        mib(after_edits),
        mib(first_pass)
    );

    workspace.close_file(CloseFileParams { path }).unwrap();
}
