use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use pgls_analyse::{AnalysisFilter, RuleFilter};
use pgls_analyser::{AnalysableStatement, Analyser, AnalyserConfig, AnalyserParams, LinterOptions};
use pgls_console::StdDisplay;
use pgls_diagnostics::{Diagnostic, PrintDiagnostic};
use sqlx::{Connection, Executor, PgConnection};

static TYPECHECK_CATALOG: OnceLock<Arc<pgls_catalog::CatalogBase>> = OnceLock::new();

fn builtin_catalog() -> Arc<pgls_catalog::CatalogBase> {
    TYPECHECK_CATALOG
        .get_or_init(|| {
            let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://postgres:postgres@127.0.0.1:5432/postgres".into()
            });
            let runtime = tokio::runtime::Runtime::new().expect("create Tokio runtime");
            let snapshot = runtime.block_on(async {
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .connect(&url)
                    .await
                    .unwrap_or_else(|e| panic!("connect to regression database {url}: {e}"));
                pgls_catalog::Snapshot::load(&pool)
                    .await
                    .expect("load PostgreSQL builtin catalog")
            });
            Arc::new(pgls_catalog::CatalogBase::new(Arc::new(builtin_snapshot(
                snapshot,
            ))))
        })
        .clone()
}

fn builtin_snapshot(mut snapshot: pgls_catalog::Snapshot) -> pgls_catalog::Snapshot {
    let builtin = |schema: &str| matches!(schema, "pg_catalog" | "information_schema");
    snapshot.schemas.retain(|x| builtin(&x.name));
    snapshot.schemas.push(pgls_catalog::Schema {
        name: "public".into(),
        ..Default::default()
    });
    snapshot.tables.retain(|x| builtin(&x.schema));
    snapshot.functions.retain(|x| builtin(&x.schema));
    snapshot.types.retain(|x| builtin(&x.schema));
    snapshot.columns.retain(|x| builtin(&x.schema_name));
    snapshot.policies.retain(|x| builtin(&x.schema_name));
    snapshot.triggers.retain(|x| builtin(&x.table_schema));
    snapshot.indexes.retain(|x| builtin(&x.schema));
    snapshot.sequences.retain(|x| builtin(&x.schema));
    snapshot
        .extensions
        .retain(|x| x.schema.as_deref().is_some_and(builtin));
    snapshot.roles.clear();
    let ids = snapshot.types.iter().map(|x| x.id).collect::<HashSet<_>>();
    snapshot
        .casts
        .retain(|x| ids.contains(&x.source) && ids.contains(&x.target));
    snapshot.operators.retain(|x| builtin(&x.schema));
    snapshot
}

#[tokio::test]
#[ignore = "requires PostgreSQL 15 and PG_REGRESS_DIR"]
async fn postgres_regression_typecheck_false_positives() {
    let Some(root) = std::env::var_os("PG_REGRESS_DIR").map(PathBuf::from) else {
        eprintln!("skipping regression corpus: PG_REGRESS_DIR is unset");
        return;
    };
    assert!(
        root.is_dir(),
        "PG_REGRESS_DIR is not a directory: {}",
        root.display()
    );
    let mut files = fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("read {}: {e}", root.display()))
        .map(|entry| entry.expect("read directory entry").path())
        .filter(|path| path.extension().is_some_and(|x| x == "sql"))
        .collect::<Vec<_>>();
    files.sort();
    let base_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@127.0.0.1:5432/postgres".into());
    let admin_url = base_url
        .rsplit_once('/')
        .map(|(x, _)| format!("{x}/postgres"))
        .unwrap_or_else(|| "postgresql://postgres:postgres@127.0.0.1:5432/postgres".into());
    let mut admin = PgConnection::connect(&admin_url)
        .await
        .expect("connect to postgres admin db");
    let catalog = builtin_catalog();
    let options = LinterOptions::default();
    let typecheck_rules = [RuleFilter::Group(pgls_analyser::TYPECHECK_GROUP)];
    let analyser = Analyser::new(AnalyserConfig {
        options: &options,
        filter: AnalysisFilter::from_enabled_rules(&typecheck_rules),
    });
    let mut report = String::new();
    let mut fp: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut tp: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut total = 0usize;
    let mut analysed = 0usize;
    let mut skipped = 0usize;

    for (file_no, path) in files.iter().enumerate() {
        let db = format!("regress_{file_no}");
        sqlx::query(&format!("CREATE DATABASE {db}"))
            .execute(&mut admin)
            .await
            .unwrap_or_else(|e| panic!("create {db}: {e}"));
        let url = database_url(&base_url, &db);
        let mut conn = PgConnection::connect(&url)
            .await
            .unwrap_or_else(|e| panic!("connect to {db}: {e}"));
        conn.execute("SET statement_timeout = '5s'")
            .await
            .expect("set statement timeout");
        let source =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let cleaned = preprocess(&source);
        let split = pgls_statement_splitter::split(&cleaned);
        let mut statements = Vec::new();
        let mut status = Vec::new();
        for range in &split.ranges {
            let sql = cleaned[*range].trim();
            if sql.is_empty() {
                status.push(PgStatus::Skipped);
                continue;
            }
            total += 1;
            let outcome = if transaction_control(sql) || copy_from_stdin(sql) {
                PgStatus::Skipped
            } else {
                match sqlx::query(sql).execute(&mut conn).await {
                    Ok(_) => PgStatus::Accepted,
                    Err(e) => {
                        let code = e
                            .as_database_error()
                            .and_then(|x| x.code())
                            .map(|x| x.into_owned());
                        match code.as_deref() {
                            Some("57014") => PgStatus::Skipped,
                            Some(state) => PgStatus::Rejected(state.to_string()),
                            None => PgStatus::Rejected("CLIENT_ERROR".into()),
                        }
                    }
                }
            };
            if matches!(&outcome, PgStatus::Skipped) {
                skipped += 1;
            }
            status.push(outcome);
            if let Ok(ast) = pgls_query::parse(sql) {
                if let Some(root) = ast.into_root() {
                    statements.push(AnalysableStatement::new(root, *range).with_sql(sql));
                }
            }
        }
        analysed += statements.len();
        let diagnostics = analyser.run(AnalyserParams {
            stmts: statements,
            catalog_base: Some(catalog.clone()),
            search_path: vec!["public".into()],
            typecheck: true,
            ..Default::default()
        });
        for diagnostic in &diagnostics {
            let Some(span) = diagnostic.location().span else {
                continue;
            };
            let index = split
                .ranges
                .iter()
                .position(|r| {
                    usize::from(r.start()) <= usize::from(span.start())
                        && usize::from(span.start()) < usize::from(r.end())
                })
                .unwrap_or(0);
            let sql = split
                .ranges
                .get(index)
                .map(|r| cleaned[*r].trim())
                .unwrap_or("");
            let rule = diagnostic.get_category_name().to_string();
            let message = format!("{}", StdDisplay(PrintDiagnostic::simple(diagnostic)));
            match status.get(index) {
                Some(PgStatus::Accepted) => fp.entry(rule).or_default().push(format!(
                    "{}: {}\n  {}\n  PostgreSQL accepted",
                    relative(path, &root),
                    truncate(sql, 300),
                    message.replace('\n', " ")
                )),
                Some(PgStatus::Rejected(state)) => {
                    *tp.entry((rule, state.clone())).or_default() += 1
                }
                Some(PgStatus::Skipped) | None => {}
            }
        }
        conn.close().await.ok();
        sqlx::query(&format!("DROP DATABASE {db} WITH (FORCE)"))
            .execute(&mut admin)
            .await
            .unwrap_or_else(|e| panic!("drop {db}: {e}"));
    }
    let fp_total = fp.values().map(Vec::len).sum::<usize>();
    let tp_total = tp.values().sum::<usize>();
    report.push_str(&format!(
        "Statements submitted: {total}\nStatements analysed: {analysed}\nStatements skipped on PostgreSQL side: {skipped}\nFalse positives: {fp_total}\nTrue positives: {tp_total}\n\nHarness caveats: transaction-control and COPY FROM STDIN statements are skipped; COPY payloads are removed during preprocessing, so data-dependent statements may differ from the regression suite.\n\n"
    ));
    report.push_str("FALSE POSITIVES BY RULE\n");
    for (rule, entries) in &fp {
        report.push_str(&format!("\n{rule} ({})\n", entries.len()));
        for entry in entries {
            report.push_str(&format!("{entry}\n"));
        }
    }
    report.push_str("\nTRUE POSITIVES BY RULE\n");
    let mut per_rule: BTreeMap<&str, usize> = BTreeMap::new();
    for ((rule, _), count) in &tp {
        *per_rule.entry(rule).or_default() += count;
    }
    for (rule, count) in per_rule {
        report.push_str(&format!("{rule}: {count}\n"));
    }
    report.push_str("\nTRUE POSITIVES BY RULE × SQLSTATE\n");
    for ((rule, state), count) in &tp {
        report.push_str(&format!("{rule} × {state}: {count}\n"));
    }
    fs::create_dir_all("target").expect("create target report directory");
    fs::write("target/regress-report.txt", &report).expect("write regression report");
    eprintln!("{report}");
    assert!(
        fp.is_empty(),
        "{} typecheck false positives; see target/regress-report.txt",
        fp.values().map(Vec::len).sum::<usize>()
    );
}

enum PgStatus {
    Accepted,
    Rejected(String),
    Skipped,
}

fn database_url(base: &str, database: &str) -> String {
    let (prefix, _) = base
        .rsplit_once('/')
        .unwrap_or(("postgresql://postgres:postgres@127.0.0.1:5432", "postgres"));
    format!("{prefix}/{database}")
}

fn relative(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}
fn truncate(sql: &str, max: usize) -> String {
    if sql.chars().count() <= max {
        sql.into()
    } else {
        format!("{}…", sql.chars().take(max).collect::<String>())
    }
}
fn copy_from_stdin(sql: &str) -> bool {
    let upper = sql.to_ascii_uppercase();
    upper.trim_start().starts_with("COPY ") && upper.contains("FROM STDIN")
}

fn transaction_control(sql: &str) -> bool {
    let normalized = sql.trim_start().to_ascii_lowercase();
    [
        "begin",
        "start transaction",
        "commit",
        "end",
        "rollback",
        "savepoint",
        "release savepoint",
        "prepare transaction",
    ]
    .iter()
    .any(|x| normalized.starts_with(x))
}

fn preprocess(source: &str) -> String {
    let mut output = String::new();
    let mut in_copy = false;
    for line in source.lines() {
        if in_copy {
            if line.trim() == "\\." {
                in_copy = false;
            }
            continue;
        }
        if line.trim_start().starts_with('\\') {
            continue;
        }
        output.push_str(line);
        output.push('\n');
        let upper = line.to_ascii_uppercase();
        if upper.contains("COPY") && upper.contains("FROM STDIN") {
            in_copy = true;
        }
    }
    output
}
