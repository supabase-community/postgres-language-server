use sqlx::PgPool;

use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Index {
    pub id: i64,
    pub schema: String,
    pub name: String,
    pub table_name: String,
}

impl SchemaCacheItem for Index {
    type Item = Index;

    async fn load(pool: &PgPool) -> Result<Vec<Index>, sqlx::Error> {
        sqlx::query_file_as!(Index, "src/queries/indexes.sql")
            .fetch_all(pool)
            .await
    }
}
