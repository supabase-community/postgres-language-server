use pgls_analyse::AnalysisFilter;
use pgls_configuration::rules::RuleConfiguration;
use pgls_configuration::splinter::{Performance, Rules, SplinterRuleOptions};
use pgls_console::fmt::{Formatter, HTML};
use pgls_diagnostics::{Diagnostic, LogCategory, Visit};
use pgls_splinter::{SplinterParams, run_splinter};
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
        // Run setup SQL
        sqlx::raw_sql(self.setup)
            .execute(self.test_db)
            .await
            .expect("Failed to setup test database");

        // Run splinter checks with all rules enabled
        let filter = AnalysisFilter::default();
        let diagnostics = run_splinter(
            SplinterParams {
                conn: self.test_db,
                schema_cache: None,
                rules_config: None,
            },
            &filter,
        )
        .await
        .expect("Failed to run splinter checks");

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

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn unindexed_foreign_key(test_db: PgPool) {
    TestSetup {
        name: "unindexed_foreign_key",
        setup: r#"
            create table public.users (
                id serial primary key,
                name text not null
            );

            create table public.posts (
                id serial primary key,
                user_id integer not null references public.users(id),
                title text not null
            );
        "#,
        test_db: &test_db,
    }
    .test()
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn no_primary_key(test_db: PgPool) {
    TestSetup {
        name: "no_primary_key",
        setup: r#"
            create table public.articles (
                title text not null,
                content text
            );
        "#,
        test_db: &test_db,
    }
    .test()
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn rls_disabled_in_public(test_db: PgPool) {
    TestSetup {
        name: "rls_disabled_in_public",
        setup: r#"
            create table public.sensitive_data (
                id serial primary key,
                secret text not null
            );
        "#,
        test_db: &test_db,
    }
    .test()
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn policy_exists_rls_disabled(test_db: PgPool) {
    TestSetup {
        name: "policy_exists_rls_disabled",
        setup: r#"
            create table public.documents (
                id serial primary key,
                content text not null
            );

            create policy "documents_policy"
                on public.documents
                for select
                to public
                using (true);
        "#,
        test_db: &test_db,
    }
    .test()
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn no_issues(test_db: PgPool) {
    TestSetup {
        name: "no_issues",
        setup: r#"
            create table public.clean_table (
                id serial primary key,
                name text not null
            );
        "#,
        test_db: &test_db,
    }
    .test()
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn multiple_issues(test_db: PgPool) {
    TestSetup {
        name: "multiple_issues",
        setup: r#"
            -- Table without primary key
            create table public.no_pk_table (
                name text
            );

            -- Table with unindexed foreign key
            create table public.parent_table (
                id serial primary key
            );

            create table public.child_table (
                id serial primary key,
                parent_id integer not null references public.parent_table(id)
            );
        "#,
        test_db: &test_db,
    }
    .test()
    .await;
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn missing_roles_runs_generic_checks_only(test_db: PgPool) {
    // Without Supabase roles, generic rules should still run
    // but Supabase-specific rules should be skipped
    let filter = AnalysisFilter::default();
    let diagnostics = run_splinter(
        SplinterParams {
            conn: &test_db,
            schema_cache: None,
            rules_config: None,
        },
        &filter,
    )
    .await
    .expect("Should not error when Supabase roles are missing");

    assert!(
        diagnostics.is_empty(),
        "Expected empty diagnostics for a clean database without Supabase roles, but got {} diagnostics",
        diagnostics.len()
    );

    // Now create a table with a generic issue (no primary key)
    sqlx::raw_sql("CREATE TABLE public.test_table (name text)")
        .execute(&test_db)
        .await
        .expect("Failed to create test table");

    let filter = AnalysisFilter::default();
    let diagnostics_with_issue = run_splinter(
        SplinterParams {
            conn: &test_db,
            schema_cache: None,
            rules_config: None,
        },
        &filter,
    )
    .await
    .expect("Should not error when checking for issues");

    assert!(
        !diagnostics_with_issue.is_empty(),
        "Expected to detect generic issues (no primary key) even without Supabase roles"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn ignore_filtering_filters_matching_objects(test_db: PgPool) {
    // Create multiple tables without primary keys
    sqlx::raw_sql(
        r#"
        CREATE TABLE public.ignored_table (name text);
        CREATE TABLE public.another_ignored (name text);
        CREATE TABLE public.not_ignored (name text);
        "#,
    )
    .execute(&test_db)
    .await
    .expect("Failed to create test tables");

    // First, run without any ignore config - should get diagnostics for all 3 tables
    let filter = AnalysisFilter::default();
    let diagnostics_without_ignore = run_splinter(
        SplinterParams {
            conn: &test_db,
            schema_cache: None,
            rules_config: None,
        },
        &filter,
    )
    .await
    .expect("Failed to run splinter checks");

    // Filter to only noPrimaryKey diagnostics
    let no_pk_diagnostics: Vec<_> = diagnostics_without_ignore
        .iter()
        .filter(|d| d.category.name().contains("noPrimaryKey"))
        .collect();

    assert_eq!(
        no_pk_diagnostics.len(),
        3,
        "Expected 3 noPrimaryKey diagnostics without ignore config, got {}",
        no_pk_diagnostics.len()
    );

    // Now run with ignore config that filters out some tables
    let rules_config = Rules {
        recommended: None,
        all: None,
        performance: Some(Performance {
            recommended: None,
            all: None,
            auth_rls_initplan: None,
            duplicate_index: None,
            multiple_permissive_policies: None,
            no_primary_key: Some(RuleConfiguration::WithOptions(
                pgls_configuration::rules::RuleWithOptions {
                    level: pgls_configuration::rules::RulePlainConfiguration::Warn,
                    options: SplinterRuleOptions {
                        ignore: vec![
                            "public.ignored_table".to_string(),
                            "public.another_*".to_string(), // glob pattern
                        ],
                    },
                },
            )),
            table_bloat: None,
            unindexed_foreign_keys: None,
            unused_index: None,
        }),
        security: None,
    };

    let diagnostics_with_ignore = run_splinter(
        SplinterParams {
            conn: &test_db,
            schema_cache: None,
            rules_config: Some(&rules_config),
        },
        &filter,
    )
    .await
    .expect("Failed to run splinter checks with ignore config");

    // Filter to only noPrimaryKey diagnostics
    let no_pk_diagnostics_filtered: Vec<_> = diagnostics_with_ignore
        .iter()
        .filter(|d| d.category.name().contains("noPrimaryKey"))
        .collect();

    assert_eq!(
        no_pk_diagnostics_filtered.len(),
        1,
        "Expected 1 noPrimaryKey diagnostic after ignore filtering (only public.not_ignored), got {}",
        no_pk_diagnostics_filtered.len()
    );

    // Verify the remaining diagnostic is for public.not_ignored
    let remaining = &no_pk_diagnostics_filtered[0];
    assert_eq!(
        remaining.advices.schema.as_deref(),
        Some("public"),
        "Expected schema 'public'"
    );
    assert_eq!(
        remaining.advices.object_name.as_deref(),
        Some("not_ignored"),
        "Expected object_name 'not_ignored'"
    );
}

#[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
async fn ignore_filtering_with_schema_wildcard(test_db: PgPool) {
    // Create a schema and tables
    sqlx::raw_sql(
        r#"
        CREATE SCHEMA audit;
        CREATE TABLE audit.log1 (data text);
        CREATE TABLE audit.log2 (data text);
        CREATE TABLE public.regular_table (data text);
        "#,
    )
    .execute(&test_db)
    .await
    .expect("Failed to create test tables");

    // Run with ignore config that filters out all audit schema tables
    let rules_config = Rules {
        recommended: None,
        all: None,
        performance: Some(Performance {
            recommended: None,
            all: None,
            auth_rls_initplan: None,
            duplicate_index: None,
            multiple_permissive_policies: None,
            no_primary_key: Some(RuleConfiguration::WithOptions(
                pgls_configuration::rules::RuleWithOptions {
                    level: pgls_configuration::rules::RulePlainConfiguration::Warn,
                    options: SplinterRuleOptions {
                        ignore: vec!["audit.*".to_string()],
                    },
                },
            )),
            table_bloat: None,
            unindexed_foreign_keys: None,
            unused_index: None,
        }),
        security: None,
    };

    let filter = AnalysisFilter::default();
    let diagnostics = run_splinter(
        SplinterParams {
            conn: &test_db,
            schema_cache: None,
            rules_config: Some(&rules_config),
        },
        &filter,
    )
    .await
    .expect("Failed to run splinter checks");

    // Filter to only noPrimaryKey diagnostics
    let no_pk_diagnostics: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.category.name().contains("noPrimaryKey"))
        .collect();

    // Should only have the public.regular_table diagnostic
    assert_eq!(
        no_pk_diagnostics.len(),
        1,
        "Expected 1 noPrimaryKey diagnostic (audit tables should be ignored), got {}",
        no_pk_diagnostics.len()
    );

    assert_eq!(
        no_pk_diagnostics[0].advices.schema.as_deref(),
        Some("public"),
        "Expected remaining diagnostic to be in public schema"
    );
    assert_eq!(
        no_pk_diagnostics[0].advices.object_name.as_deref(),
        Some("regular_table"),
        "Expected remaining diagnostic to be for regular_table"
    );
}
