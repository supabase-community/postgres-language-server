use sqlx::PgPool;

use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Default)]
pub struct Schema {
    pub id: i64,
    pub name: String,
    pub owner: String,
    pub allowed_users: Vec<String>,
    pub allowed_creators: Vec<String>,
    pub table_count: i64,
    pub view_count: i64,
    pub function_count: i64,
    pub total_size: String,
    pub comment: Option<String>,
}

impl SchemaCacheItem for Schema {
    type Item = Schema;

    async fn load(pool: &PgPool) -> Result<Vec<Schema>, sqlx::Error> {
        sqlx::query_file_as!(Schema, "src/queries/schemas.sql")
            .fetch_all(pool)
            .await
    }
}
