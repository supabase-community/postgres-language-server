use sqlx::PgPool;

use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Clone, Default)]
pub struct Schema {
    id: i64,
    name: String,
    owner: String,
}

impl SchemaCacheItem for Schema {
    type Item = Schema;

    async fn load(pool: &PgPool) -> Result<Vec<Schema>, sqlx::Error> {
        sqlx::query_file_as!(Schema, "src/queries/schemas.sql")
            .fetch_all(pool)
            .await
    }
}
