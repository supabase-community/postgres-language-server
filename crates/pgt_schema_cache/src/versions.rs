use sqlx::PgPool;

use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Default)]
pub struct Version {
    pub version: Option<String>,
    pub version_num: Option<i64>,
    pub major_version: Option<i64>,
    pub active_connections: Option<i64>,
    pub max_connections: Option<i64>,
}

impl SchemaCacheItem for Version {
    type Item = Version;

    async fn load(pool: &PgPool) -> Result<Vec<Version>, sqlx::Error> {
        sqlx::query_file_as!(Version, "src/queries/versions.sql")
            .fetch_all(pool)
            .await
    }

    /*
    Sample Output:
    -[ RECORD 1 ]------+--------------------------------------------------------------------------------------------------------------------------
    version            | PostgreSQL 15.7 (Debian 15.7-1.pgdg120+1) on aarch64-unknown-linux-gnu, compiled by gcc (Debian 12.2.0-14) 12.2.0, 64-bit
    version_num        | 150007
    active_connections | 8
    max_connections    | 100
    */
}
