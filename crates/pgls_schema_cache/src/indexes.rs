#[cfg(feature = "db")]
use sqlx::PgPool;

#[cfg(feature = "db")]
use crate::schema_cache::SchemaCacheItem;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Index {
    pub id: i64,
    pub schema: String,
    pub name: String,
    pub table_name: String,
}

#[cfg(feature = "db")]
impl SchemaCacheItem for Index {
    type Item = Index;

    async fn load(pool: &PgPool) -> Result<Vec<Index>, sqlx::Error> {
        sqlx::query_file_as!(Index, "src/queries/indexes.sql")
            .fetch_all(pool)
            .await
    }
}
