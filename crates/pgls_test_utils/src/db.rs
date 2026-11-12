//! The test databases of [`crate::db_test`].
//!
//! `#[db_test]` lets sqlx create, migrate and drop a database per test, as `#[sqlx::test]` does,
//! and changes two things:
//!
//! - It checks once per test binary that the test database answers. Without the check, every
//!   test retried for 30 seconds and then failed with `PoolTimedOut`.
//! - It makes the pool of each test smaller and lets it wait longer for a connection, see
//!   [`TEST_POOL_CONNECTIONS_MAX`] and [`TEST_POOL_ACQUIRE_TIMEOUT`].

use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Duration;

use sqlx::pool::PoolOptions;
use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{Connection, PgPool, Postgres};

/// The connections one test holds at most. sqlx allows 5, but the pools of all tests in a binary
/// take their connections from one parent pool of 20, and the setup of the next test's database
/// waits at most 30 seconds for one of them (not configurable). `Snapshot::load` runs 14 queries
/// at once, so a few tests that load a snapshot took all 20 and the setup of other tests timed out
/// on a loaded machine. With 2, a binary on 8 test threads needs at most 16.
pub const TEST_POOL_CONNECTIONS_MAX: u32 = 2;

/// How long a test waits for a connection. sqlx's default of 30 seconds is too short when several
/// worktrees run their tests against the one test database at the same time.
pub const TEST_POOL_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(120);

/// How long the check waits for the test database to answer. A database that is down refuses the
/// connection at once; this only bounds a host that drops the packets.
const REACHABLE_TIMEOUT: Duration = Duration::from_secs(30);

/// The URL of the test database, from `DATABASE_URL` or the `.env` file.
pub fn database_url() -> String {
    dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set, see .env")
}

/// Panics with a clear message if the test database doesn't answer. Checks once per test binary.
pub fn require_test_db() {
    static REACHABLE: OnceLock<Result<(), String>> = OnceLock::new();

    if let Err(message) = REACHABLE.get_or_init(|| check_reachable(&database_url())) {
        panic!("{message}");
    }
}

fn check_reachable(url: &str) -> Result<(), String> {
    let options = PgConnectOptions::from_str(url)
        .map_err(|error| format!("DATABASE_URL {url} is not a valid URL ({error})"))?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to start the runtime of the test database check");

    runtime.block_on(async {
        let not_reachable = |reason: String| {
            format!("test database not reachable at {url} ({reason}); run `just db`")
        };

        match tokio::time::timeout(REACHABLE_TIMEOUT, PgConnection::connect_with(&options)).await {
            Ok(Ok(connection)) => {
                let _ = connection.close().await;
                Ok(())
            }
            // The database answers but is busy or still starting up; sqlx retries these.
            Ok(Err(sqlx::Error::Database(error)))
                if matches!(error.code().as_deref(), Some("53300" | "57P03")) =>
            {
                Ok(())
            }
            Ok(Err(error)) => Err(not_reachable(error.to_string())),
            Err(_) => Err(not_reachable(format!(
                "no answer within {} seconds",
                REACHABLE_TIMEOUT.as_secs()
            ))),
        }
    })
}

/// Connects the pool of a test. `pool_options` are sqlx's, which share the connections of the
/// test binary.
pub async fn connect_test_pool(
    pool_options: PoolOptions<Postgres>,
    connect_options: PgConnectOptions,
) -> PgPool {
    pool_options
        .max_connections(TEST_POOL_CONNECTIONS_MAX)
        .acquire_timeout(TEST_POOL_ACQUIRE_TIMEOUT)
        .connect_with(connect_options)
        .await
        .expect("failed to connect test pool")
}

/// Closes the pool of a test, so that sqlx can drop its database.
pub async fn close_test_pool(pool: PgPool) {
    // As in `#[sqlx::test]`: a test that still holds a connection blocks `close` forever.
    if tokio::time::timeout(Duration::from_secs(10), pool.close())
        .await
        .is_err()
    {
        eprintln!("the test held onto its pool after exiting");
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::{TEST_POOL_CONNECTIONS_MAX, check_reachable};

    #[test]
    fn reports_a_database_that_is_not_running() {
        // Nothing listens on port 1.
        let error = check_reachable("postgresql://postgres:postgres@127.0.0.1:1/postgres")
            .expect_err("port 1 should refuse the connection");

        assert!(
            error.starts_with("test database not reachable at postgresql://postgres:postgres@127.0.0.1:1/postgres ("),
            "{error}"
        );
        assert!(error.ends_with("); run `just db`"), "{error}");
    }

    #[crate::db_test]
    async fn runs_against_a_fresh_database_with_a_small_pool(test_db: PgPool) {
        assert_eq!(
            test_db.options().get_max_connections(),
            TEST_POOL_CONNECTIONS_MAX
        );

        let database: String = sqlx::query_scalar("select current_database()")
            .fetch_one(&test_db)
            .await
            .unwrap();
        assert!(database.starts_with("_sqlx_test_"), "{database}");

        let migrations: i64 = sqlx::query_scalar("select count(*) from _sqlx_migrations")
            .fetch_one(&test_db)
            .await
            .unwrap();
        assert_eq!(migrations, 1);
    }
}
