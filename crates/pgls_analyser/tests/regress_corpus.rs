use std::time::Duration;
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
    let catalog = tokio::task::spawn_blocking(builtin_catalog)
        .await
        .expect("load builtin catalog");
    let options = LinterOptions::default();
    let typecheck_rules = [RuleFilter::Group(pgls_analyser::TYPECHECK_GROUP)];
    let analyser = Analyser::new(AnalyserConfig {
        options: &options,
        filter: AnalysisFilter::from_enabled_rules(&typecheck_rules),
    });
    let mut report = String::new();
    let mut fp: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut tp: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut artefacts: Vec<String> = Vec::new();
    let mut tp_details: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    let mut total = 0usize;
    let mut analysed = 0usize;
    let mut skipped = 0usize;

    for (file_no, path) in files.iter().enumerate() {
        let db = format!("regress_{file_no}");
        // A database left over from an interrupted run.
        sqlx::query(&format!("DROP DATABASE IF EXISTS {db} WITH (FORCE)"))
            .execute(&mut admin)
            .await
            .unwrap_or_else(|e| panic!("drop {db}: {e}"));
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
        // Set when a statement hangs on the client side; the connection is unusable from then on.
        let mut stuck = false;
        for range in &split.ranges {
            let sql = cleaned[*range].trim();
            if sql.is_empty() {
                status.push(PgStatus::Skipped);
                continue;
            }
            total += 1;
            let outcome = if stuck || transaction_control(sql) || copy_with_client(sql) {
                PgStatus::Skipped
            } else {
                let execute = sqlx::raw_sql(sql).execute(&mut conn);
                match tokio::time::timeout(Duration::from_secs(30), execute).await {
                    Err(_) => {
                        stuck = true;
                        PgStatus::Skipped
                    }
                    Ok(Ok(_)) => PgStatus::Accepted,
                    Ok(Err(e)) => {
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
            if std::env::var_os("REGRESS_DEBUG").is_some() {
                let state = match &outcome {
                    PgStatus::Accepted => "accepted".to_owned(),
                    PgStatus::Rejected(state) => state.clone(),
                    PgStatus::Skipped => "skipped".to_owned(),
                };
                eprintln!("[{state}] {}", truncate(sql, 120));
            }
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
                Some(PgStatus::Accepted) => {
                    let file = relative(path, &root);
                    let entry = format!(
                        "{file}: {}\n  {}\n  PostgreSQL accepted",
                        truncate(sql, 300),
                        message.replace('\n', " ")
                    );
                    match known_artefact(&file, sql) {
                        Some(reason) => artefacts.push(format!("{entry}\n  {reason}")),
                        None => fp.entry(rule).or_default().push(entry),
                    }
                }
                Some(PgStatus::Rejected(state)) => {
                    tp_details
                        .entry((rule.clone(), state.clone()))
                        .or_default()
                        .push(format!(
                            "{}: {}\n  {}",
                            relative(path, &root),
                            truncate(sql, 300),
                            message.replace('\n', " ")
                        ));
                    *tp.entry((rule, state.clone())).or_default() += 1
                }
                Some(PgStatus::Skipped) | None => {}
            }
        }
        conn.close().await.ok();
        // Roles are shared by all databases.
        let roles = regress_roles(&mut admin).await;
        if !roles.is_empty() {
            if let Ok(mut conn) = PgConnection::connect(&url).await {
                drop_owned(&mut conn, &roles).await;
                conn.close().await.ok();
            }
        }
        sqlx::query(&format!("DROP DATABASE {db} WITH (FORCE)"))
            .execute(&mut admin)
            .await
            .unwrap_or_else(|e| panic!("drop {db}: {e}"));
        drop_roles(&mut admin, &roles).await;
    }
    let fp_total = fp.values().map(Vec::len).sum::<usize>();
    let tp_total = tp.values().sum::<usize>();
    report.push_str(&format!(
        "Statements submitted: {total}\nStatements analysed: {analysed}\nStatements skipped on PostgreSQL side: {skipped}\nFalse positives: {fp_total}\nTrue positives: {tp_total}\n\nHarness caveats: transaction-control statements, COPY to or from the client, and statements after one that hangs are skipped; COPY payloads are removed during preprocessing, so data-dependent statements may differ from the regression suite.\n\n"
    ));
    report.push_str("FALSE POSITIVES BY RULE\n");
    for (rule, entries) in &fp {
        report.push_str(&format!("\n{rule} ({})\n", entries.len()));
        for entry in entries {
            report.push_str(&format!("{entry}\n"));
        }
    }
    report.push_str(&format!(
        "\nKNOWN HARNESS ARTEFACTS ({})\n",
        artefacts.len()
    ));
    for entry in &artefacts {
        report.push_str(&format!("{entry}\n"));
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
    report.push_str("\nTRUE POSITIVES\n");
    for ((rule, state), entries) in &tp_details {
        report.push_str(&format!("\n{rule} × {state}\n"));
        for entry in entries {
            report.push_str(&format!("{entry}\n"));
        }
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

/// Findings on statements Postgres accepted only because an earlier statement of the file
/// failed. The linter assumes every statement succeeds, like a migration that stops at the
/// first error.
const KNOWN_ARTEFACTS: &[(&str, &str, &str)] = &[(
    "transactions.sql",
    "writetest",
    "`DROP TABLE writetest` fails in a read-only session, so the table still exists",
)];

fn known_artefact(file: &str, sql: &str) -> Option<&'static str> {
    KNOWN_ARTEFACTS
        .iter()
        .find(|(known_file, needle, _)| *known_file == file && sql.contains(needle))
        .map(|(_, _, reason)| *reason)
}

/// The roles the regression tests create; they are all named `regress_*`.
async fn regress_roles(conn: &mut PgConnection) -> Vec<String> {
    sqlx::query_scalar("select quote_ident(rolname) from pg_roles where rolname like 'regress\\_%'")
        .fetch_all(conn)
        .await
        .expect("list regression roles")
}

async fn drop_owned(conn: &mut PgConnection, roles: &[String]) {
    for role in roles {
        sqlx::raw_sql(&format!("DROP OWNED BY {role} CASCADE"))
            .execute(&mut *conn)
            .await
            .ok();
    }
}

async fn drop_roles(admin: &mut PgConnection, roles: &[String]) {
    drop_owned(admin, roles).await;
    for role in roles {
        sqlx::raw_sql(&format!("DROP ROLE IF EXISTS {role}"))
            .execute(&mut *admin)
            .await
            .ok();
    }
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
/// `COPY` to or from the client. Postgres also accepts `FROM STDOUT` and `TO STDIN`.
fn copy_with_client(sql: &str) -> bool {
    let upper = sql.to_ascii_uppercase();
    upper.trim_start().starts_with("COPY ") && (upper.contains("STDIN") || upper.contains("STDOUT"))
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
        if upper.contains("COPY") && (upper.contains("FROM STDIN") || upper.contains("FROM STDOUT"))
        {
            in_copy = true;
        }
    }
    output
}
