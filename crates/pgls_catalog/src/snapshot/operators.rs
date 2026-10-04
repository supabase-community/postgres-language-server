#[cfg(feature = "db")]
use super::SnapshotItem;
use serde::{Deserialize, Serialize};
#[cfg(feature = "db")]
use sqlx::PgPool;

/// An operator present in pg_operator.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PostgresOperator {
    pub oid: i64,
    pub schema: String,
    pub name: String,
    pub kind: String,
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
