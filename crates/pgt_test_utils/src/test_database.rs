use std::ops::Deref;

use sqlx::{
    Executor, PgPool,
    postgres::{PgConnectOptions, PgQueryResult},
};
use uuid::Uuid;

#[derive(Debug)]
pub struct TestDb {
    pool: PgPool,
    roles: Vec<String>,
}

#[derive(Debug)]
pub struct RoleWithArgs {
    pub role: String,
    pub args: Vec<String>,
}

impl TestDb {
    pub async fn execute(&self, sql: &str) -> Result<PgQueryResult, sqlx::Error> {
        if sql.to_ascii_lowercase().contains("create role") {
            panic!("Please setup roles via the `setup_roles` method.")
        }
        self.pool.execute(sql).await
    }

    pub async fn setup_roles(
        &mut self,
        roles: Vec<RoleWithArgs>,
    ) -> Result<PgQueryResult, sqlx::Error> {
        self.roles = roles.iter().map(|r| &r.role).cloned().collect();

        let role_statements: Vec<String> = roles
            .into_iter()
            .map(|r| {
                format!(
                    r#"
                if not exists (
                    select from pg_catalog.pg_roles
                    where rolname = '{0}'
                ) then 
                    create role {0} {1};
                end if;
            "#,
                    r.role,
                    r.args.join(" ")
                )
            })
            .collect();

        let query = format!(
            r#"
            do $$
            begin
                {}
            end $$;
        "#,
            role_statements.join("\n")
        );

        self.pool.execute(query.as_str()).await
    }

    pub fn get_roles(&self) -> &[String] {
        &self.roles
    }
}

impl Deref for TestDb {
    type Target = PgPool;
    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

// TODO: Work with proper config objects instead of a connection_string.
// With the current implementation, we can't parse the password from the connection string.
pub async fn get_new_test_db() -> TestDb {
    dotenv::dotenv().expect("Unable to load .env file for tests");

    let connection_string = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let password = std::env::var("DB_PASSWORD").unwrap_or("postgres".into());

    let options_from_conn_str: PgConnectOptions = connection_string
        .parse()
        .expect("Invalid Connection String");

    let host = options_from_conn_str.get_host();
    assert!(
        host == "localhost" || host == "127.0.0.1",
        "Running tests against non-local database!"
    );

    let options_without_db_name = PgConnectOptions::new()
        .host(host)
        .port(options_from_conn_str.get_port())
        .username(options_from_conn_str.get_username())
        .password(&password);

    let postgres = sqlx::PgPool::connect_with(options_without_db_name.clone())
        .await
        .expect("Unable to connect to test postgres instance");

    let database_name = Uuid::new_v4().to_string();

    postgres
        .execute(format!(r#"create database "{}";"#, database_name).as_str())
        .await
        .expect("Failed to create test database.");

    let pool = sqlx::PgPool::connect_with(options_without_db_name.database(&database_name))
        .await
        .expect("Could not connect to test database");

    TestDb {
        pool,
        roles: vec![],
    }
}
