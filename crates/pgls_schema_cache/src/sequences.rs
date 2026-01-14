use sqlx::PgPool;

use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Sequence {
    pub id: i64,
    pub schema: String,
    pub name: String,
}

impl SchemaCacheItem for Sequence {
    type Item = Sequence;

    async fn load(pool: &PgPool) -> Result<Vec<Sequence>, sqlx::Error> {
        sqlx::query_file_as!(Sequence, "src/queries/sequences.sql")
            .fetch_all(pool)
            .await
    }
}
