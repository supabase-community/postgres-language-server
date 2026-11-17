use futures::FutureExt;
use futures::future::BoxFuture;
use pgls_analyse::AnalysisFilter;
use pgls_catalog::Snapshot;
use pgls_plpgsql_check::{PlPgSqlCheckDiagnostic, PlPgSqlCheckParams};
use pgls_splinter::{SplinterDiagnostic, SplinterParams};
use pgls_typecheck::{TypecheckDiagnostic, TypecheckParams};
use sqlx::PgPool;

/// The queries the workspace sends a database, besides the statements a user executes.
///
/// [`Postgres`] sends them to the database. A test can answer them with a fake instead, to inject
/// failed connections, failing queries and diagnostics with spans no database would report, see
/// [`super::WorkspaceServer::with_runtime_and_database`].
pub trait DatabaseQueries: Send + Sync + 'static {
    /// The snapshot of the database's schema, see [`Snapshot::load`].
    fn snapshot<'a>(&'a self, pool: &'a PgPool) -> BoxFuture<'a, Result<Snapshot, sqlx::Error>>;

    /// The type check of a statement, see [`pgls_typecheck::check_sql`].
    fn typecheck<'a>(
        &'a self,
        params: TypecheckParams<'a>,
    ) -> BoxFuture<'a, Result<Option<TypecheckDiagnostic>, sqlx::Error>>;

    /// plpgsql_check's diagnostics of a function, see [`pgls_plpgsql_check::check_plpgsql`].
    fn plpgsql_check<'a>(
        &'a self,
        params: PlPgSqlCheckParams<'a>,
    ) -> BoxFuture<'a, Result<Option<Vec<PlPgSqlCheckDiagnostic>>, sqlx::Error>>;

    /// splinter's diagnostics of the database, see [`pgls_splinter::run_splinter`].
    fn splinter<'a>(
        &'a self,
        params: SplinterParams<'a>,
        filter: &'a AnalysisFilter<'a>,
    ) -> BoxFuture<'a, Result<Vec<SplinterDiagnostic>, sqlx::Error>>;
}

/// Sends the queries to the database.
pub struct Postgres;

impl DatabaseQueries for Postgres {
    fn snapshot<'a>(&'a self, pool: &'a PgPool) -> BoxFuture<'a, Result<Snapshot, sqlx::Error>> {
        Snapshot::load(pool).boxed()
    }

    fn typecheck<'a>(
        &'a self,
        params: TypecheckParams<'a>,
    ) -> BoxFuture<'a, Result<Option<TypecheckDiagnostic>, sqlx::Error>> {
        pgls_typecheck::check_sql(params).boxed()
    }

    fn plpgsql_check<'a>(
        &'a self,
        params: PlPgSqlCheckParams<'a>,
    ) -> BoxFuture<'a, Result<Option<Vec<PlPgSqlCheckDiagnostic>>, sqlx::Error>> {
        pgls_plpgsql_check::check_plpgsql(params).boxed()
    }

    fn splinter<'a>(
        &'a self,
        params: SplinterParams<'a>,
        filter: &'a AnalysisFilter<'a>,
    ) -> BoxFuture<'a, Result<Vec<SplinterDiagnostic>, sqlx::Error>> {
        pgls_splinter::run_splinter(params, filter).boxed()
    }
}
