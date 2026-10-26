use std::time::Duration;

use futures::{FutureExt, TryStreamExt, channel::oneshot, future};
use pgls_query::NodeEnum;
use pgls_text_size::TextSize;
use sqlx::{
    Column, Either, Executor, PgConnection, Row, Statement, ValueRef,
    postgres::{PgColumn, PgDatabaseError, PgErrorPosition, PgRow},
};
use tracing::instrument::WithSubscriber;

use super::{
    EXECUTE_STATEMENT_MAX_RESULT_SIZE, EXECUTE_STATEMENT_MAX_ROWS,
    EXECUTE_STATEMENT_MAX_VALUE_SIZE, ExecuteStatementError, ExecuteStatementResult, notices,
};
use crate::diagnostics::is_connection_failure;
use crate::workspace::server::DatabasePools;

/// The cursor that a query runs in, so that Postgres stops once it produced the rows the result
/// holds.
const CURSOR: &str = "pgls_statement";

/// How to execute a statement.
pub(crate) struct ExecuteOptions {
    /// Whether the statement is a query that a cursor can run: a `select` or `values` without
    /// `into` and without a data-modifying `with`.
    pub(crate) is_query: bool,
    /// How long the statement may run, or `None` to keep the session's `statement_timeout`.
    pub(crate) timeout: Option<Duration>,
}

/// Whether a cursor can run `ast`: `declare` takes a `select` or `values` without `into` and
/// without a data-modifying `with`.
pub(crate) fn is_query(ast: &NodeEnum) -> bool {
    let NodeEnum::SelectStmt(select) = ast else {
        return false;
    };
    select.into_clause.is_none()
        && select.with_clause.as_ref().is_none_or(|with| {
            with.ctes.iter().all(|cte| match &cte.node {
                Some(NodeEnum::CommonTableExpr(cte)) => matches!(
                    cte.ctequery
                        .as_deref()
                        .and_then(|query| query.node.as_ref()),
                    Some(NodeEnum::SelectStmt(_))
                ),
                _ => false,
            })
        })
}

/// Whether executing `ast` may change the schema, i.e. it does more than reading or writing rows.
pub(crate) fn may_change_schema(ast: &NodeEnum) -> bool {
    match ast {
        NodeEnum::SelectStmt(select) => select.into_clause.is_some(),
        NodeEnum::InsertStmt(_)
        | NodeEnum::UpdateStmt(_)
        | NodeEnum::DeleteStmt(_)
        | NodeEnum::MergeStmt(_)
        | NodeEnum::VariableShowStmt(_) => false,
        _ => true,
    }
}

/// Executes `sql` and collects the rows it returns, if any.
///
/// The statement runs over the simple query protocol, so Postgres sends every value in its text
/// representation and no type needs decoding. An error that Postgres raises for the statement is
/// part of the result, with its position relative to `sql`. Only a failed connection is an error.
///
/// Each statement runs on its own: afterwards, the session is reset, so a `begin` or `set` does
/// not carry over to the next statement on the connection.
///
/// When `cancel` fires, Postgres cancels the statement.
pub(crate) async fn execute_statement(
    pools: &DatabasePools,
    sql: &str,
    options: ExecuteOptions,
    cancel: oneshot::Receiver<()>,
) -> Result<ExecuteStatementResult, sqlx::Error> {
    let notices = notices::NoticeCollector::default();
    let result = execute_on_own_connection(pools, sql, options, cancel)
        .with_subscriber(notices.dispatch())
        .await;

    result.map(|mut result| {
        result.notices = notices.take();
        for notice in &result.notices {
            result.message.push('\n');
            result.message.push_str(notice);
        }
        result
    })
}

async fn execute_on_own_connection(
    pools: &DatabasePools,
    sql: &str,
    options: ExecuteOptions,
    cancel: oneshot::Receiver<()>,
) -> Result<ExecuteStatementResult, sqlx::Error> {
    let mut pool_conn = pools.statement_pool.acquire().await?;
    let conn: &mut PgConnection = &mut pool_conn;
    let backend_pid = prepare_session(conn, options.timeout).await?;

    // Boxing the future pins down the lifetime of the borrowed connection, which the `Send`
    // check of `run_async` can't infer otherwise.
    let result = match future::select(run(conn, sql, options.is_query).boxed(), cancel).await {
        future::Either::Left((result, _)) => result,
        future::Either::Right((cancelled, run)) => {
            if cancelled.is_ok() {
                // A connection can't cancel its own statement while it waits for the result.
                if let Err(err) = sqlx::query("select pg_cancel_backend($1)")
                    .bind(backend_pid)
                    .execute(&pools.pool)
                    .await
                {
                    tracing::warn!("Could not cancel the executed statement: {err}");
                }
            }
            run.await
        }
    };

    match reset_session(conn).await {
        // The cursor of a query has a transaction of its own, which a failed query leaves open.
        Ok(rolled_back) if rolled_back && !options.is_query => result.map(|mut result| {
            if !result.message.ends_with('.') {
                result.message.push('.');
            }
            result.message.push_str(
                " The open transaction was rolled back, as executed statements don't share a session.",
            );
            result
        }),
        Ok(_) => result,
        Err(err) => {
            tracing::warn!("Closing the connection, as its session could not be reset: {err}");
            let _ = pool_conn.close().await;
            result
        }
    }
}

/// Applies the statement timeout and returns the process ID of the connection's backend, which
/// `pg_cancel_backend` takes.
async fn prepare_session(
    conn: &mut PgConnection,
    timeout: Option<Duration>,
) -> Result<i32, sqlx::Error> {
    let query = match timeout {
        Some(timeout) => format!(
            "select pg_backend_pid(), set_config('statement_timeout', '{}', false)",
            timeout.as_millis()
        ),
        None => "select pg_backend_pid()".to_string(),
    };
    conn.fetch_one(sqlx::raw_sql(&query)).await?.try_get(0)
}

/// Rolls back the transaction that a statement left open and resets the session, like
/// `pgbouncer`'s `server_reset_query`. Returns whether a transaction was open.
async fn reset_session(conn: &mut PgConnection) -> Result<bool, sqlx::Error> {
    match conn.execute(sqlx::raw_sql("discard all")).await {
        Ok(_) => Ok(false),
        // `discard all` can't run inside a transaction block (25001), and a failed transaction
        // only accepts a rollback (25P02).
        Err(sqlx::Error::Database(err))
            if matches!(err.code().as_deref(), Some("25001" | "25P02")) =>
        {
            conn.execute(sqlx::raw_sql("rollback")).await?;
            conn.execute(sqlx::raw_sql("discard all")).await?;
            Ok(true)
        }
        Err(err) => Err(err),
    }
}

/// Runs `sql`, a query in a cursor that fetches one row more than the result holds.
async fn run(
    conn: &mut PgConnection,
    sql: &str,
    is_query: bool,
) -> Result<ExecuteStatementResult, sqlx::Error> {
    let prefix = format!("begin; declare {CURSOR} no scroll cursor for\n");
    let query = if is_query {
        let statement = sql.trim_end();
        let statement = statement.strip_suffix(';').unwrap_or(statement);
        // The newline ends a trailing line comment.
        format!(
            "{prefix}{statement}\n; fetch {} from {CURSOR}; commit",
            EXECUTE_STATEMENT_MAX_ROWS + 1
        )
    } else {
        sql.to_string()
    };

    match collect_result(conn, &query, sql).await {
        Err(err) if !is_connection_failure(&err) => match err
            .as_database_error()
            .and_then(|err| err.try_downcast_ref::<PgDatabaseError>())
        {
            Some(err) => {
                let offset = if is_query { prefix.len() } else { 0 };
                Ok(rejected(err, sql, offset))
            }
            None => Err(err),
        },
        result => result,
    }
}

/// Runs `query` and collects its rows. `sql` is the statement as written, to describe it.
async fn collect_result(
    conn: &mut PgConnection,
    query: &str,
    sql: &str,
) -> Result<ExecuteStatementResult, sqlx::Error> {
    let mut columns = None;
    let mut rows = Vec::new();
    let mut size = 0;
    let mut rows_affected = 0;
    let mut truncated = false;
    let mut truncated_values = false;

    // Read the whole result even when it is truncated, so that the statement runs to completion
    // before the session is reset. A query in a cursor stops early instead.
    let mut results = sqlx::raw_sql(query).fetch_many(&mut *conn);
    while let Some(result) = results.try_next().await? {
        match result {
            Either::Left(result) => rows_affected += result.rows_affected(),
            Either::Right(row) => {
                columns.get_or_insert_with(|| column_names(row.columns()));
                if truncated {
                    continue;
                }
                let values = row_values(&row, &mut truncated_values);
                let row_size = values.iter().flatten().map(String::len).sum::<usize>();
                if rows.len() == EXECUTE_STATEMENT_MAX_ROWS
                    || size + row_size > EXECUTE_STATEMENT_MAX_RESULT_SIZE
                {
                    truncated = true;
                    continue;
                }
                size += row_size;
                rows.push(values);
            }
        }
    }
    drop(results);

    // Postgres sends no columns along with zero rows, so a query without rows looks like a command.
    // Describing the statement tells them apart, unless it affected rows and is thus a command.
    if columns.is_none() && rows_affected == 0 {
        columns = conn
            .prepare(sql)
            .await
            .ok()
            .map(|statement| column_names(statement.columns()))
            .filter(|columns| !columns.is_empty());
    }

    Ok(match columns {
        Some(columns) => ExecuteStatementResult {
            message: if truncated {
                format!("Showing the first {} rows (truncated).", rows.len())
            } else if rows.len() == 1 {
                "1 row".to_string()
            } else {
                format!("{} rows", rows.len())
            },
            columns,
            rows,
            truncated,
            truncated_values,
            ..Default::default()
        },
        None => ExecuteStatementResult {
            message: format!("Successfully executed statement. Rows affected: {rows_affected}"),
            rows_affected: Some(rows_affected),
            ..Default::default()
        },
    })
}

/// The result for an error that Postgres raised. Its position counts the characters of the query,
/// which has `offset` bytes of ASCII before `sql`.
fn rejected(err: &PgDatabaseError, sql: &str, offset: usize) -> ExecuteStatementResult {
    // Postgres counts the characters from 1, and points one past the end for an error at the end
    // of the input.
    let position = match err.position() {
        Some(PgErrorPosition::Original(position)) => position
            .checked_sub(1 + offset)
            .and_then(|index| {
                sql.char_indices()
                    .map(|(offset, _)| offset)
                    .chain([sql.len()])
                    .nth(index)
            })
            .and_then(|offset| TextSize::try_from(offset).ok()),
        _ => None,
    };

    ExecuteStatementResult {
        message: format!("Statement failed: {}", err.message()),
        error: Some(ExecuteStatementError {
            message: err.message().to_string(),
            code: err.code().to_string(),
            detail: err.detail().map(str::to_string),
            hint: err.hint().map(str::to_string),
            position,
        }),
        ..Default::default()
    }
}

fn column_names(columns: &[PgColumn]) -> Vec<String> {
    columns
        .iter()
        .map(|column| column.name().to_string())
        .collect()
}

/// The values of `row`, each cut to [`EXECUTE_STATEMENT_MAX_VALUE_SIZE`] bytes.
fn row_values(row: &PgRow, truncated_values: &mut bool) -> Vec<Option<String>> {
    (0..row.len())
        .map(|index| {
            let value = row.try_get_raw(index).ok()?;
            if value.is_null() {
                return None;
            }
            let bytes = value.as_bytes().unwrap_or_default();
            if bytes.len() <= EXECUTE_STATEMENT_MAX_VALUE_SIZE {
                return Some(String::from_utf8_lossy(bytes).into_owned());
            }

            *truncated_values = true;
            // Cut before a UTF-8 continuation byte, so that no character is split.
            let mut end = EXECUTE_STATEMENT_MAX_VALUE_SIZE;
            while end > 0 && bytes[end] & 0b1100_0000 == 0b1000_0000 {
                end -= 1;
            }
            let mut value = String::from_utf8_lossy(&bytes[..end]).into_owned();
            value.push('…');
            Some(value)
        })
        .collect()
}
