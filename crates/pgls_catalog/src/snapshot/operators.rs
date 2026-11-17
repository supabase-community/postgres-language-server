#[cfg(feature = "db")]
use super::SnapshotItem;
use serde::{Deserialize, Serialize};
#[cfg(feature = "db")]
use sqlx::PgPool;

use super::code::Code;
use crate::OperatorKind;

/// An operator present in pg_operator.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PostgresOperator {
    pub oid: i64,
    pub schema: String,
    pub name: String,
    /// `prefix` or `infix`, from `pg_operator.oprkind`. Unknown for a postfix operator
    /// (Postgres 13 and older).
    pub kind: Code<OperatorKind>,
    pub left: i64,
    pub right: i64,
    pub result: i64,
}
#[cfg(feature = "db")]
impl SnapshotItem for PostgresOperator {
    type Item = PostgresOperator;
    async fn load(pool: &PgPool) -> Result<Vec<Self::Item>, sqlx::Error> {
        sqlx::query_file_as!(PostgresOperator, "src/snapshot/queries/operators.sql")
            .fetch_all(pool)
            .await
    }
}
