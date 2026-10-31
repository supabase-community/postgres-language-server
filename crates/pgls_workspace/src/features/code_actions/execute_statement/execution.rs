use futures::TryStreamExt;
use pgls_text_size::TextSize;
use sqlx::{
    Column, Either, Executor, PgConnection, PgPool, Row, Statement, ValueRef,
    postgres::{PgColumn, PgDatabaseError, PgErrorPosition, PgRow},
};

use super::{EXECUTE_STATEMENT_MAX_ROWS, ExecuteStatementError, ExecuteStatementResult};
use crate::diagnostics::is_connection_failure;

/// Executes `sql` and collects the rows it returns, if any.
///
/// The statement runs over the simple query protocol, so Postgres sends every value in its text
/// representation and no type needs decoding. An error that Postgres raises for the statement is
/// part of the result, with its position relative to `sql`. Only a failed connection is an error.
///
/// Each statement runs on its own: afterwards, the session is reset, so a `begin` or `set` does
/// not carry over to the next statement on the connection.
pub(crate) async fn execute_statement(
    pool: &PgPool,
    sql: &str,
) -> Result<ExecuteStatementResult, sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let result = match collect_result(&mut conn, sql).await {
        Err(err) if !is_connection_failure(&err) => match err
            .as_database_error()
            .and_then(|err| err.try_downcast_ref::<PgDatabaseError>())
        {
            Some(err) => Ok(rejected(err, sql)),
            None => Err(err),
        },
        result => result,
    };

    match reset_session(&mut conn).await {
        Ok(rolled_back) => result.map(|mut result| {
            if rolled_back {
                if !result.message.ends_with('.') {
                    result.message.push('.');
                }
                result.message.push_str(
                    " The open transaction was rolled back, as executed statements don't share a session.",
                );
            }
            result
        }),
        Err(err) => {
            tracing::warn!("Closing the connection, as its session could not be reset: {err}");
            let _ = conn.close().await;
            result
        }
    }
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

async fn collect_result(
    conn: &mut PgConnection,
    sql: &str,
) -> Result<ExecuteStatementResult, sqlx::Error> {
    let mut columns = None;
    let mut rows = Vec::new();
    let mut rows_affected = 0;
    let mut truncated = false;

    let mut results = sqlx::raw_sql(sql).fetch_many(&mut *conn);
    while let Some(result) = results.try_next().await? {
        match result {
            Either::Left(result) => rows_affected += result.rows_affected(),
            Either::Right(row) => {
                if rows.len() == EXECUTE_STATEMENT_MAX_ROWS {
                    truncated = true;
                    break;
                }
                columns.get_or_insert_with(|| column_names(row.columns()));
                rows.push(row_values(&row));
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
                format!("Showing the first {EXECUTE_STATEMENT_MAX_ROWS} rows (truncated).")
            } else if rows.len() == 1 {
                "1 row".to_string()
            } else {
                format!("{} rows", rows.len())
            },
            columns,
            rows,
            truncated,
            ..Default::default()
        },
        None => ExecuteStatementResult {
            message: format!("Successfully executed statement. Rows affected: {rows_affected}"),
            rows_affected: Some(rows_affected),
            ..Default::default()
        },
    })
}

fn rejected(err: &PgDatabaseError, sql: &str) -> ExecuteStatementResult {
    // Postgres counts the characters from 1, and points one past the end for an error at the end
    // of the input.
    let position = match err.position() {
        Some(PgErrorPosition::Original(position)) => position.checked_sub(1).and_then(|index| {
            sql.char_indices()
                .map(|(offset, _)| offset)
                .chain([sql.len()])
                .nth(index)
                .and_then(|offset| TextSize::try_from(offset).ok())
        }),
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

fn row_values(row: &PgRow) -> Vec<Option<String>> {
    (0..row.len())
        .map(|index| {
            let value = row.try_get_raw(index).ok()?;
            if value.is_null() {
                return None;
            }
            let bytes = value.as_bytes().unwrap_or_default();
            Some(String::from_utf8_lossy(bytes).into_owned())
        })
        .collect()
}
