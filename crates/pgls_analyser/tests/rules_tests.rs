use core::slice;
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    fs::read_to_string,
    path::Path,
    sync::{Arc, OnceLock},
};

use pgls_analyse::{AnalysisFilter, Applicability, RuleFilter};
use pgls_analyser::{
    AnalysableStatement, Analyser, AnalyserConfig, AnalyserParams, Direction, LinterDiagnostic,
    LinterOptions, MigrationSection, RuleFix, UnparsableStatement,
};
use pgls_console::StdDisplay;
use pgls_diagnostics::PrintDiagnostic;
use pgls_text_size::{TextRange, TextSize};

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
    let is_typecheck = group == pgls_analyser::TYPECHECK_GROUP;

    let results = analyse(&analyser, &query, is_typecheck);

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

    assert_fixes_converge(&analyser, &query, is_typecheck);
}

fn analyse(analyser: &Analyser, query: &str, is_typecheck: bool) -> Vec<LinterDiagnostic> {
    let split = pgls_statement_splitter::split(query);

    // Statements that don't parse are passed on as such, like the workspace does. The body of a
    // SQL function follows the function, with its parameters in scope.
    let mut stmts = Vec::new();
    let mut unparsable = Vec::new();
    for r in &split.ranges {
        let text = &query[*r];
        match pgls_query::parse(text) {
            Ok(ast) => {
                let root = ast.into_root().expect("Failed to convert AST to root node");
                let body = sql_function_body(&root, text, *r);
                stmts.push(AnalysableStatement::new(root, *r).with_sql(text));
                stmts.extend(body);
            }
            Err(_) => unparsable.push(UnparsableStatement {
                range: *r,
                sql: text.to_owned(),
            }),
        }
    }

    // Typecheck rules use the test database's built-in catalog; user schemas are filtered out.
    analyser.run(AnalyserParams {
        stmts,
        unparsable,
        catalog_base: is_typecheck.then(typecheck_catalog),
        search_path: vec!["public".into()],
        typecheck: is_typecheck,
        sections: migration_sections(query),
        ..Default::default()
    })
}

/// Applies all fixes, safe and unsafe, until none is left, like `check --fix --unsafe`. That must
/// end: a fixed statement must not get the same fix again.
fn assert_fixes_converge(analyser: &Analyser, query: &str, is_typecheck: bool) {
    let mut code = query.to_string();
    for _ in 0..10 {
        let diagnostics = analyse(analyser, &code, is_typecheck);
        let mut fixes: Vec<&RuleFix> = diagnostics
            .iter()
            .filter_map(LinterDiagnostic::get_fix)
            .collect();
        fixes.sort_by_key(|fix| fix.range().map(TextRange::start));
        let mut combined = RuleFix::new("");
        for fix in fixes {
            let overlaps = combined
                .range()
                .zip(fix.range())
                .is_some_and(|(previous, range)| range.start() <= previous.end());
            if !overlaps {
                combined.edits.extend(fix.edits.iter().cloned());
            }
        }
        if combined.edits.is_empty() {
            return;
        }
        code = combined.apply(&code);
    }
    panic!("the fixes still change the input after 10 passes:\n{code}");
}

/// The statement in the body of a `LANGUAGE sql` function, as the workspace extracts it.
fn sql_function_body(
    root: &pgls_query::NodeEnum,
    sql: &str,
    range: TextRange,
) -> Option<AnalysableStatement> {
    let pgls_query::NodeEnum::CreateFunctionStmt(create_fn) = root else {
        return None;
    };
    let body = pgls_query_ext::utils::sql_function_body(root, sql)?;
    let body_root = pgls_query::parse(&body.body).ok()?.into_root()?;
    Some(
        AnalysableStatement::new(body_root, body.range + range.start())
            .with_sql(body.body)
            .with_function(pgls_catalog::resolve::FunctionContext::from_create_function(create_fn)),
    )
}

/// The migration sections of the file, each starting at a `-- section: up` or
/// `-- section: down transaction:false` line. Like dbmate, each section is sent to Postgres as
/// one query, unless `queries:separate` follows, like goose without `StatementBegin`.
fn migration_sections(query: &str) -> Vec<MigrationSection> {
    let mut sections: Vec<MigrationSection> = Vec::new();
    let mut one_query = Vec::new();
    let mut offset = 0;
    for line in query.split_inclusive('\n') {
        let start = TextSize::from(offset as u32);
        offset += line.len();
        let Some(section) = line.trim_end().strip_prefix("-- section:") else {
            continue;
        };
        let mut words = section.split_whitespace();
        let direction = match words.next() {
            Some("up") => Direction::Up,
            Some("down") => Direction::Down,
            other => panic!("Unknown section direction `{other:?}`"),
        };
        let mut transaction = true;
        let mut separate = false;
        for option in words {
            match option {
                "transaction:false" => transaction = false,
                "queries:separate" => separate = true,
                other => panic!("Unknown section option `{other}`"),
            }
        }
        if let Some(previous) = sections.last_mut() {
            previous.range = TextRange::new(previous.range.start(), start);
        }
        sections.push(MigrationSection {
            range: TextRange::new(start, TextSize::from(query.len() as u32)),
            direction,
            transaction,
            queries: Vec::new(),
        });
        one_query.push(!separate);
    }
    for (section, one_query) in sections.iter_mut().zip(one_query) {
        if one_query {
            section.queries = vec![section.range];
        }
    }
    sections
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

            if let Some(fix) = diagnostic.get_fix() {
                write_fix(snapshot, query, fix);
            }
        }
    }
}

/// Writes the input with the fix applied. A fix must keep the input valid SQL.
fn write_fix(snapshot: &mut String, query: &str, fix: &RuleFix) {
    let applicability = match fix.applicability {
        Applicability::Always => "safe",
        Applicability::MaybeIncorrect => "unsafe",
    };
    let fixed = fix.apply(query);
    if pgls_query::parse(query).is_ok() {
        assert!(
            pgls_query::parse(&fixed).is_ok(),
            "the fix `{}` produces invalid SQL:\n{fixed}",
            fix.message
        );
    }

    writeln!(snapshot, "## Fix ({applicability}): {}", fix.message).unwrap();
    writeln!(snapshot, "```").unwrap();
    writeln!(snapshot, "{fixed}").unwrap();
    writeln!(snapshot, "```").unwrap();
    writeln!(snapshot).unwrap();
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
