#[cfg(feature = "db")]
use sqlx::PgPool;

#[cfg(feature = "db")]
use super::SnapshotItem;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Index {
    pub id: i64,
    pub schema: String,
    pub name: String,
    pub table_name: String,
}

#[cfg(feature = "db")]
impl SnapshotItem for Index {
    type Item = Index;

    async fn load(pool: &PgPool) -> Result<Vec<Index>, sqlx::Error> {
        sqlx::query_file_as!(Index, "src/snapshot/queries/indexes.sql")
            .fetch_all(pool)
            .await
    }
}
