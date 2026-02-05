//! Integration tests for pglinter diagnostics
//!
//! These tests configure pglinter thresholds to 0% so rules fire deterministically.
//!
//! Note: These tests require the pglinter extension to be installed, which is only
//! available on Linux (via Docker) and macOS. Windows CI does not have pglinter.

#![cfg(not(target_os = "windows"))]

use pgls_analyse::AnalysisFilter;
use pgls_console::fmt::{Formatter, HTML};
use pgls_diagnostics::{Diagnostic, LogCategory, Visit};
use pgls_pglinter::{PglinterCache, PglinterParams, run_pglinter};
use pgls_schema_cache::SchemaCache;
use sqlx::PgPool;
use std::fmt::Write;
use std::io;

struct TestVisitor {
    logs: Vec<String>,
}

impl TestVisitor {
    fn new() -> Self {
        Self { logs: Vec::new() }
    }

    fn into_string(self) -> String {
        self.logs.join("\n")
    }
}

impl Visit for TestVisitor {
    fn record_log(
        &mut self,
        category: LogCategory,
        text: &dyn pgls_console::fmt::Display,
    ) -> io::Result<()> {
        let prefix = match category {
            LogCategory::None => "",
            LogCategory::Info => "[Info] ",
            LogCategory::Warn => "[Warn] ",
            LogCategory::Error => "[Error] ",
        };

        let mut buffer = vec![];
        let mut writer = HTML::new(&mut buffer);
        let mut formatter = Formatter::new(&mut writer);
        text.fmt(&mut formatter)?;

        let text_str = String::from_utf8(buffer).unwrap();
        self.logs.push(format!("{prefix}{text_str}"));
        Ok(())
    }
}

/// Configure pglinter for deterministic testing:
/// - Set all thresholds to 0% warning, 1% error so any violation triggers
/// - Disable cluster-level rules that depend on pg_hba.conf
async fn configure_pglinter_for_tests(pool: &PgPool) {
    // Set thresholds to 0% warning for deterministic behavior
    let rules_to_configure = [
        "B001", "B002", "B003", "B004", "B005", "B006", "B007", "B008", "B009", "B010", "B011",
        "B012", "S001", "S002", "S003", "S004", "S005",
    ];

    for rule in rules_to_configure {
        let _ = sqlx::query("SELECT pglinter.update_rule_levels($1, 0, 1)")
            .bind(rule)
            .execute(pool)
            .await;
    }

    // Disable cluster-level rules (depend on pg_hba.conf, not deterministic)
    for rule in ["C001", "C002", "C003"] {
        let _ = sqlx::query("SELECT pglinter.disable_rule($1)")
            .bind(rule)
            .execute(pool)
            .await;
    }
}

struct TestSetup<'a> {
    name: &'a str,
    setup: &'a str,
    test_db: &'a PgPool,
    /// Only include rules matching these prefixes (e.g., ["B001", "B005"])
    /// Empty means include all non-cluster rules
    rule_filter: Vec<&'a str>,
}

impl TestSetup<'_> {
    async fn test(self) {
        // Create required extensions (pglinter may depend on plpgsql_check)
        sqlx::raw_sql("CREATE EXTENSION IF NOT EXISTS plpgsql_check")
            .execute(self.test_db)
            .await
            .expect("plpgsql_check extension not available");

        sqlx::raw_sql("CREATE EXTENSION IF NOT EXISTS pglinter")
            .execute(self.test_db)
            .await
            .expect("pglinter extension not available");

        configure_pglinter_for_tests(self.test_db).await;

        sqlx::raw_sql(self.setup)
            .execute(self.test_db)
            .await
            .expect("Failed to setup test database");

        let schema_cache = SchemaCache::load(self.test_db)
            .await
            .expect("Failed to load schema cache");

        let cache = PglinterCache::load(self.test_db, &schema_cache)
            .await
            .expect("Failed to load pglinter cache");

        let filter = AnalysisFilter::default();
        let diagnostics = run_pglinter(
            PglinterParams {
                conn: self.test_db,
                schema_cache: &schema_cache,
            },
            &filter,
            Some(&cache),
        )
        .await
        .expect("Failed to run pglinter checks");

        // Filter diagnostics
        let filtered: Vec<_> = diagnostics
            .iter()
            .filter(|d| {
                let category = d.category().map(|c| c.name()).unwrap_or("");
                // Exclude cluster-level rules
                if category.contains("/cluster/") {
                    return false;
                }
                // Apply rule filter if specified
                if !self.rule_filter.is_empty() {
                    let rule_code = d.advices.rule_code.as_deref().unwrap_or("");
                    return self.rule_filter.contains(&rule_code);
                }
                true
            })
            .collect();

        // Sort by category for deterministic output
        let mut sorted = filtered;
        sorted.sort_by_key(|d| d.category().map(|c| c.name()).unwrap_or("unknown"));

        let content = if sorted.is_empty() {
            String::from("No Diagnostics")
        } else {
            let mut result = String::new();

            for (idx, diagnostic) in sorted.iter().enumerate() {
                if idx > 0 {
                    writeln!(&mut result).unwrap();
                    writeln!(&mut result, "---").unwrap();
                    writeln!(&mut result).unwrap();
                }

                let category_name = diagnostic.category().map(|c| c.name()).unwrap_or("unknown");
                writeln!(&mut result, "Category: {category_name}").unwrap();
                writeln!(&mut result, "Severity: {:?}", diagnostic.severity()).unwrap();

                let mut msg_content = vec![];
                let mut writer = HTML::new(&mut msg_content);
                let mut formatter = Formatter::new(&mut writer);
                diagnostic.message(&mut formatter).unwrap();
                writeln!(
                    &mut result,
                    "Message: {}",
                    String::from_utf8(msg_content).unwrap()
                )
                .unwrap();

                let mut visitor = TestVisitor::new();
                diagnostic.advices(&mut visitor).unwrap();
                let advice_text = visitor.into_string();
                if !advice_text.is_empty() {
                    writeln!(&mut result, "Advices:\n{advice_text}").unwrap();
                }
            }

            result
        };

        insta::with_settings!({
            prepend_module_to_snapshot => false,
        }, {
            insta::assert_snapshot!(self.name, content);
        });
    }
}

/// Test that pglinter extension can be created
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn extension_check(test_db: PgPool) {
    // Create required extensions (pglinter may depend on plpgsql_check)
    sqlx::raw_sql("CREATE EXTENSION IF NOT EXISTS plpgsql_check")
        .execute(&test_db)
        .await
        .expect("plpgsql_check extension not available");

    sqlx::raw_sql("CREATE EXTENSION IF NOT EXISTS pglinter")
        .execute(&test_db)
        .await
        .expect("pglinter extension not available");

    let schema_cache = SchemaCache::load(&test_db)
        .await
        .expect("Failed to load schema cache");

    assert!(
        schema_cache.extensions.iter().any(|e| e.name == "pglinter"),
        "pglinter extension not found"
    );
}

/// Test B001: Table without primary key
/// Note: pglinter checks ALL tables in the database globally, not just specific tables.
/// So this test verifies that B001 fires when any table lacks a primary key.
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn table_without_primary_key(test_db: PgPool) {
    TestSetup {
        name: "table_without_primary_key",
        setup: r#"
            CREATE TABLE public.test_no_pk (
                name text,
                value integer
            );
        "#,
        test_db: &test_db,
        rule_filter: vec!["B001"],
    }
    .test()
    .await;
}

/// Test B005: Objects with uppercase names
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn objects_with_uppercase(test_db: PgPool) {
    TestSetup {
        name: "objects_with_uppercase",
        setup: r#"
            CREATE TABLE public."TestTable" (
                id serial PRIMARY KEY,
                "UserName" text
            );
        "#,
        test_db: &test_db,
        rule_filter: vec!["B005"],
    }
    .test()
    .await;
}

/// Test B003: Foreign key without index
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn fk_without_index(test_db: PgPool) {
    TestSetup {
        name: "fk_without_index",
        setup: r#"
            CREATE TABLE public.parent_table (
                id serial PRIMARY KEY,
                name text
            );

            CREATE TABLE public.child_table (
                id serial PRIMARY KEY,
                parent_id integer NOT NULL REFERENCES public.parent_table(id)
            );
        "#,
        test_db: &test_db,
        rule_filter: vec!["B003"],
    }
    .test()
    .await;
}

/// Test multiple issues at once
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn multiple_issues(test_db: PgPool) {
    TestSetup {
        name: "multiple_issues",
        setup: r#"
            -- Table without primary key
            CREATE TABLE public.no_pk (
                name text
            );

            -- Table with uppercase name
            CREATE TABLE public."BadName" (
                id serial PRIMARY KEY
            );

            -- FK without index
            CREATE TABLE public.ref_parent (
                id serial PRIMARY KEY
            );

            CREATE TABLE public.ref_child (
                id serial PRIMARY KEY,
                parent_id integer REFERENCES public.ref_parent(id)
            );
        "#,
        test_db: &test_db,
        rule_filter: vec!["B001", "B003", "B005"],
    }
    .test()
    .await;
}
