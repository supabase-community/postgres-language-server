//! Integration tests for pglinter diagnostics
//!
//! These tests require the pglinter extension to be installed in the test database.

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

struct TestSetup<'a> {
    name: &'a str,
    setup: &'a str,
    test_db: &'a PgPool,
}

impl TestSetup<'_> {
    async fn test(self) {
        // Load schema cache
        let schema_cache = SchemaCache::load(self.test_db)
            .await
            .expect("Failed to load schema cache");

        // Assert pglinter extension is installed
        assert!(
            schema_cache.extensions.iter().any(|e| e.name == "pglinter"),
            "pglinter extension must be installed for tests to run"
        );

        // Run setup SQL
        sqlx::raw_sql(self.setup)
            .execute(self.test_db)
            .await
            .expect("Failed to setup test database");

        // Reload schema cache after setup
        let schema_cache = SchemaCache::load(self.test_db)
            .await
            .expect("Failed to reload schema cache");

        // Load pglinter cache
        let cache = PglinterCache::load(self.test_db, &schema_cache)
            .await
            .expect("Failed to load pglinter cache");

        // Run pglinter checks with all rules enabled
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

        let content = if diagnostics.is_empty() {
            String::from("No Diagnostics")
        } else {
            let mut result = String::new();

            for (idx, diagnostic) in diagnostics.iter().enumerate() {
                if idx > 0 {
                    writeln!(&mut result).unwrap();
                    writeln!(&mut result, "---").unwrap();
                    writeln!(&mut result).unwrap();
                }

                // Write category
                let category_name = diagnostic.category().map(|c| c.name()).unwrap_or("unknown");
                writeln!(&mut result, "Category: {category_name}").unwrap();

                // Write severity
                writeln!(&mut result, "Severity: {:?}", diagnostic.severity()).unwrap();

                // Write message
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

                // Write advices using custom visitor
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

/// Test that checks extension availability
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn extension_check(test_db: PgPool) {
    let schema_cache = SchemaCache::load(&test_db)
        .await
        .expect("Failed to load schema cache");

    assert!(
        schema_cache.extensions.iter().any(|e| e.name == "pglinter"),
        "pglinter extension must be installed for tests to run"
    );
}

/// Test B001: Table without primary key
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
    }
    .test()
    .await;
}

/// Test with a clean table (has primary key)
#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn table_with_primary_key(test_db: PgPool) {
    TestSetup {
        name: "table_with_primary_key",
        setup: r#"
            CREATE TABLE public.test_with_pk (
                id serial PRIMARY KEY,
                name text
            );
        "#,
        test_db: &test_db,
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
    }
    .test()
    .await;
}
