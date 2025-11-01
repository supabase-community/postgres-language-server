use pgls_console::fmt::{Formatter, HTML};
use pgls_diagnostics::{Diagnostic, LogCategory, Visit};
use pgls_splinter::{run_splinter, SplinterParams};
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
        // Create Supabase-specific roles that splinter expects
        for role in ["anon", "authenticated", "service_role"] {
            let result = sqlx::query(&format!("CREATE ROLE {role} NOLOGIN"))
                .execute(self.test_db)
                .await;

            // Ignore duplicate role errors
            if let Err(sqlx::Error::Database(db_err)) = &result {
                let code = db_err.code();
                if code.as_deref() != Some("23505") && code.as_deref() != Some("42710") {
                    result.expect("Failed to create Supabase roles");
                }
            }
        }

        // Run setup SQL
        sqlx::raw_sql(self.setup)
            .execute(self.test_db)
            .await
            .expect("Failed to setup test database");

        // Run splinter checks
        let diagnostics = run_splinter(SplinterParams { conn: self.test_db })
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
async fn missing_roles_returns_empty(test_db: PgPool) {
    let diagnostics = run_splinter(SplinterParams { conn: &test_db })
        .await
        .expect("Should not error when roles are missing");

    assert!(
        diagnostics.is_empty(),
        "Expected empty diagnostics when Supabase roles are missing, but got {} diagnostics",
        diagnostics.len()
    );
}
