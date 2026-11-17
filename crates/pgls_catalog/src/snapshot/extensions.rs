use serde::{Deserialize, Serialize};
#[cfg(feature = "db")]
use sqlx::PgPool;

#[cfg(feature = "db")]
use super::SnapshotItem;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Extension {
    pub name: String,
    pub schema: Option<String>,
    pub default_version: String,
    pub installed_version: Option<String>,
    pub comment: Option<String>,
}

impl Extension {
    /// Whether the extension is installed in the database. The snapshot also lists the
    /// extensions that are only available on the server, without an installed version.
    pub fn is_installed(&self) -> bool {
        self.installed_version.is_some()
    }
}

#[cfg(feature = "db")]
impl SnapshotItem for Extension {
    type Item = Extension;

    async fn load(pool: &PgPool) -> Result<Vec<Extension>, sqlx::Error> {
        sqlx::query_file_as!(Extension, "src/snapshot/queries/extensions.sql")
            .fetch_all(pool)
            .await
    }
}
