use core::slice;
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    fs::read_to_string,
    path::Path,
    sync::{Arc, OnceLock},
};

use pgls_analyse::{AnalysisFilter, RuleFilter};
use pgls_analyser::{
    AnalysableStatement, Analyser, AnalyserConfig, AnalyserParams, LinterDiagnostic, LinterOptions,
};
use pgls_console::StdDisplay;
use pgls_diagnostics::PrintDiagnostic;

static TYPECHECK_CATALOG: OnceLock<Arc<pgls_catalog::CatalogBase>> = OnceLock::new();

fn typecheck_catalog() -> Arc<pgls_catalog::CatalogBase> {
    TYPECHECK_CATALOG
        .get_or_init(|| {
            let default_url = "postgresql://postgres:postgres@127.0.0.1:5432/postgres";
            let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| default_url.into());
            let runtime = tokio::runtime::Runtime::new().expect("failed to create test database runtime");
            let snapshot = runtime.block_on(async {
                let pool = sqlx::postgres::PgPoolOptions::new()
                    .connect(&database_url)
                    .await
                    .unwrap_or_else(|error| {
                        panic!(
                            "typecheck specs need the test database at {database_url}; run `docker compose up -d` ({error})"
                        )
                    });
                pgls_catalog::Snapshot::load(&pool).await.unwrap_or_else(|error| {
                    panic!(
                        "typecheck specs need the test database at {database_url}; run `docker compose up -d` ({error})"
                    )
                })
            });
            Arc::new(pgls_catalog::CatalogBase::new(Arc::new(builtin_snapshot(snapshot))))
        })
        .clone()
}

fn builtin_snapshot(mut snapshot: pgls_catalog::Snapshot) -> pgls_catalog::Snapshot {
    let is_builtin = |schema: &str| matches!(schema, "pg_catalog" | "information_schema");

    snapshot.schemas.retain(|schema| is_builtin(&schema.name));
    snapshot.schemas.push(pgls_catalog::Schema {
        name: "public".into(),
        ..Default::default()
    });
    snapshot.tables.retain(|table| is_builtin(&table.schema));
    snapshot
        .functions
        .retain(|function| is_builtin(&function.schema));
    snapshot.types.retain(|ty| is_builtin(&ty.schema));
    snapshot
        .columns
        .retain(|column| is_builtin(&column.schema_name));
    snapshot
        .policies
        .retain(|policy| is_builtin(&policy.schema_name));
    snapshot
        .triggers
        .retain(|trigger| is_builtin(&trigger.table_schema));
    snapshot.indexes.retain(|index| is_builtin(&index.schema));
    snapshot
        .sequences
        .retain(|sequence| is_builtin(&sequence.schema));
    snapshot
        .extensions
        .retain(|extension| extension.schema.as_deref().is_some_and(is_builtin));
    snapshot.roles.clear();
    let builtin_type_ids = snapshot
        .types
        .iter()
        .map(|ty| ty.id)
        .collect::<HashSet<_>>();
    // Casts are global catalog entries; retain only casts whose endpoint types are built-ins.
    snapshot.casts.retain(|cast| {
        builtin_type_ids.contains(&cast.source) && builtin_type_ids.contains(&cast.target)
    });
    snapshot
        .operators
        .retain(|operator| is_builtin(&operator.schema));
    snapshot
}

pgls_test_macros::gen_tests! {
  "tests/specs/**/*.sql",
  crate::rule_test
}

fn rule_test(full_path: &'static str, _: &str, _: &str) {
    let input_file = Path::new(full_path);

    let (group, rule, fname) = parse_test_path(input_file);

    let rule_filter = RuleFilter::Rule(group.as_str(), rule.as_str());
    let filter = AnalysisFilter {
        enabled_rules: Some(slice::from_ref(&rule_filter)),
        ..Default::default()
    };

    let query =
        read_to_string(full_path).unwrap_or_else(|_| panic!("Failed to read file: {full_path} "));

    let options = LinterOptions::default();
    let analyser = Analyser::new(AnalyserConfig {
        options: &options,
        filter,
    });

    let split = pgls_statement_splitter::split(&query);

    let stmts = split
        .ranges
        .iter()
        .map(|r| {
            let text = &query[*r];
            let ast = pgls_query::parse(text).expect("failed to parse SQL");

            AnalysableStatement::new(
                ast.into_root().expect("Failed to convert AST to root node"),
                *r,
            )
            .with_sql(text)
        })
        .collect::<Vec<_>>();

    // Typecheck rules use the test database's built-in catalog; user schemas are filtered out.
    let is_typecheck = group == pgls_analyser::TYPECHECK_GROUP;
    let results = analyser.run(AnalyserParams {
        stmts,
        catalog_base: is_typecheck.then(typecheck_catalog),
        search_path: vec!["public".into()],
        typecheck: is_typecheck,
        ..Default::default()
    });

    let mut snapshot = String::new();
    write_snapshot(&mut snapshot, query.as_str(), results.as_slice());

    insta::with_settings!({
        prepend_module_to_snapshot => false,
        snapshot_path => input_file.parent().unwrap(),
    }, {
        insta::assert_snapshot!(fname, snapshot);
    });

    let expectation = Expectation::from_file(&query);
    expectation.assert(results.as_slice());
}

fn parse_test_path(path: &Path) -> (String, String, String) {
    let mut comps: Vec<&str> = path
        .components()
        .map(|c| c.as_os_str().to_str().unwrap())
        .collect();

    let fname = comps.pop().unwrap();
    let rule = comps.pop().unwrap();
    let group = comps.pop().unwrap();

    (group.into(), rule.into(), fname.into())
}

fn write_snapshot(snapshot: &mut String, query: &str, diagnostics: &[LinterDiagnostic]) {
    writeln!(snapshot, "# Input").unwrap();
    writeln!(snapshot, "```").unwrap();
    writeln!(snapshot, "{query}").unwrap();
    writeln!(snapshot, "```").unwrap();
    writeln!(snapshot).unwrap();

    if !diagnostics.is_empty() {
        writeln!(snapshot, "# Diagnostics").unwrap();
        for diagnostic in diagnostics {
            let printer = PrintDiagnostic::simple(diagnostic);

            writeln!(snapshot, "{}", StdDisplay(printer)).unwrap();
            writeln!(snapshot).unwrap();
        }
    }
}

enum Expectation {
    NoDiagnostics,
    Diagnostics(Vec<(String, usize)>),
}

impl Expectation {
    fn from_file(content: &str) -> Self {
        let mut multiple_of: HashMap<&str, i32> = HashMap::new();
        for line in content.lines() {
            if line.contains("expect_no_diagnostics") {
                if !multiple_of.is_empty() {
                    panic!(
                        "Cannot use both `expect_no_diagnostics` and `expect_` in the same test"
                    );
                }
                return Self::NoDiagnostics;
            }

            if line.contains("expect_") && !line.contains("expect_no_diagnostics") {
                let kind = line
                    .splitn(3, "_")
                    .last()
                    .expect("Use pattern: `-- expect_<category>`")
                    .trim();

                *multiple_of.entry(kind).or_insert(0) += 1;
            }
        }

        if !multiple_of.is_empty() {
            return Self::Diagnostics(
                multiple_of
                    .into_iter()
                    .map(|(k, v)| (k.into(), v as usize))
                    .collect(),
            );
        }

        panic!(
            "No expectation found in the test file. Use `-- expect_no_diagnostics` or `-- expect_<category>`"
        );
    }

    fn assert(&self, diagnostics: &[LinterDiagnostic]) {
        match self {
            Self::NoDiagnostics => {
                if !diagnostics.is_empty() {
                    panic!("This test should not have any diagnostics.");
                }
            }
            Self::Diagnostics(expected) => {
                let mut counts: HashMap<&str, usize> = HashMap::new();
                for diag in diagnostics {
                    *counts.entry(diag.get_category_name()).or_insert(0) += 1;
                }

                for (kind, expected_count) in expected {
                    let actual_count = counts.get(kind.as_str()).copied().unwrap_or(0);
                    if actual_count != *expected_count {
                        panic!(
                            "Expected {expected_count} diagnostics of kind `{kind}`, but found {actual_count}."
                        );
                    }
                }
            }
        }
    }
}
