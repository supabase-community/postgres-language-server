#[cfg(feature = "db")]
use super::SnapshotItem;
use serde::{Deserialize, Serialize};
#[cfg(feature = "db")]
use sqlx::PgPool;

/// A cast present in pg_cast.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PostgresCast {
    pub source: i64,
    pub target: i64,
    pub context: String,
    pub method: String,
}
#[cfg(feature = "db")]
impl SnapshotItem for PostgresCast {
    type Item = PostgresCast;
    async fn load(pool: &PgPool) -> Result<Vec<Self::Item>, sqlx::Error> {
        sqlx::query_file_as!(PostgresCast, "src/snapshot/queries/casts.sql")
            .fetch_all(pool)
            .await
    }
}
