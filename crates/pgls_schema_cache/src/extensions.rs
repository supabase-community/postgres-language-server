use serde::{Deserialize, Serialize};
#[cfg(feature = "db")]
use sqlx::PgPool;

#[cfg(feature = "db")]
use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Extension {
    pub name: String,
    pub schema: Option<String>,
    pub default_version: String,
    pub installed_version: Option<String>,
    pub comment: Option<String>,
}

#[cfg(feature = "db")]
impl SchemaCacheItem for Extension {
    type Item = Extension;

    async fn load(pool: &PgPool) -> Result<Vec<Extension>, sqlx::Error> {
        sqlx::query_file_as!(Extension, "src/queries/extensions.sql")
            .fetch_all(pool)
            .await
    }
}
