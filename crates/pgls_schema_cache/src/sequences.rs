#[cfg(feature = "db")]
use sqlx::PgPool;

#[cfg(feature = "db")]
use crate::schema_cache::SchemaCacheItem;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sequence {
    pub id: i64,
    pub schema: String,
    pub name: String,
}

#[cfg(feature = "db")]
impl SchemaCacheItem for Sequence {
    type Item = Sequence;

    async fn load(pool: &PgPool) -> Result<Vec<Sequence>, sqlx::Error> {
        sqlx::query_file_as!(Sequence, "src/queries/sequences.sql")
            .fetch_all(pool)
            .await
    }
}
