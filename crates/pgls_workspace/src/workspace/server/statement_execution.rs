use futures::TryStreamExt;
use sqlx::{
    Column, Either, Executor, PgPool, Row, Statement, ValueRef,
    postgres::{PgColumn, PgRow},
};

use crate::features::code_actions::{EXECUTE_STATEMENT_MAX_ROWS, ExecuteStatementResult};

/// Executes `sql` and collects the rows it returns, if any.
///
/// The statement runs over the simple query protocol, so Postgres sends every value in its text
/// representation and no type needs decoding.
pub(super) async fn execute_statement(
    pool: &PgPool,
    sql: &str,
) -> Result<ExecuteStatementResult, sqlx::Error> {
    let mut columns = None;
    let mut rows = Vec::new();
    let mut rows_affected = 0;
    let mut truncated = false;

    let mut results = sqlx::raw_sql(sql).fetch_many(pool);
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
        columns = pool
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
            rows_affected: None,
        },
        None => ExecuteStatementResult {
            message: format!("Successfully executed statement. Rows affected: {rows_affected}"),
            rows_affected: Some(rows_affected),
            ..Default::default()
        },
    })
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
