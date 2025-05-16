use sqlx::PgPool;

use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ReplicaIdentity {
    #[default]
    Default,
    Index,
    Full,
    Nothing,
}

impl From<String> for ReplicaIdentity {
    fn from(s: String) -> Self {
        match s.as_str() {
            "DEFAULT" => ReplicaIdentity::Default,
            "INDEX" => ReplicaIdentity::Index,
            "FULL" => ReplicaIdentity::Full,
            "NOTHING" => ReplicaIdentity::Nothing,
            _ => panic!("Invalid replica identity"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Table {
    pub id: i64,
    pub schema: String,
    pub name: String,
    pub rls_enabled: bool,
    pub rls_forced: bool,
    pub replica_identity: ReplicaIdentity,
    pub bytes: i64,
    pub size: String,
    pub live_rows_estimate: i64,
    pub dead_rows_estimate: i64,
    pub comment: Option<String>,
}

impl SchemaCacheItem for Table {
    type Item = Table;

    async fn load(pool: &PgPool) -> Result<Vec<Table>, sqlx::Error> {
        sqlx::query_file_as!(Table, "src/queries/tables.sql")
            .fetch_all(pool)
            .await
    }
}
