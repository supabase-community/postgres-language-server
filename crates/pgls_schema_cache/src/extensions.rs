use sqlx::PgPool;

use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Default)]
pub struct Extension {
    pub name: String,
    pub schema: Option<String>,
    pub default_version: String,
    pub installed_version: Option<String>,
    pub comment: Option<String>,
}

impl SchemaCacheItem for Extension {
    type Item = Extension;

    async fn load(pool: &PgPool) -> Result<Vec<Extension>, sqlx::Error> {
        sqlx::query_file_as!(Extension, "src/queries/extensions.sql")
            .fetch_all(pool)
            .await
    }
}
