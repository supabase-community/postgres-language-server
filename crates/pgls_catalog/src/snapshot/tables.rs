use serde::{Deserialize, Serialize};
#[cfg(feature = "db")]
use sqlx::PgPool;

#[cfg(feature = "db")]
use super::SnapshotItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ReplicaIdentity {
    #[default]
    Default,
    Index,
    Full,
    Nothing,
    /// A `pg_class.relreplident` we don't know.
    #[serde(other)]
    #[cfg_attr(feature = "schema", schemars(skip))]
    Unknown,
}

impl From<String> for ReplicaIdentity {
    fn from(s: String) -> Self {
        match s.as_str() {
            "DEFAULT" => ReplicaIdentity::Default,
            "INDEX" => ReplicaIdentity::Index,
            "FULL" => ReplicaIdentity::Full,
            "NOTHING" => ReplicaIdentity::Nothing,
            _ => ReplicaIdentity::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TableKind {
    #[default]
    Ordinary,
    View,
    MaterializedView,
    Partitioned,
    /// A `pg_class.relkind` we don't know.
    #[serde(other)]
    #[cfg_attr(feature = "schema", schemars(skip))]
    Unknown,
}

// `pg_class.relkind`, a `"char"`.
impl From<i8> for TableKind {
    fn from(code: i8) -> Self {
        match code.cast_unsigned() {
            b'r' => Self::Ordinary,
            b'p' => Self::Partitioned,
            b'v' => Self::View,
            b'm' => Self::MaterializedView,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Table {
    pub id: i64,
    pub schema: String,
    pub name: String,
    pub rls_enabled: bool,
    pub rls_forced: bool,
    pub replica_identity: ReplicaIdentity,
    pub table_kind: TableKind,
    pub bytes: i64,
    pub size: String,
    pub live_rows_estimate: i64,
    pub dead_rows_estimate: i64,
    pub comment: Option<String>,
    /// Whether the table is a partition or inherits from another table.
    #[serde(default)]
    pub is_inheritance_child: bool,
}

#[cfg(feature = "db")]
impl SnapshotItem for Table {
    type Item = Table;

    async fn load(pool: &PgPool) -> Result<Vec<Table>, sqlx::Error> {
        sqlx::query_file_as!(Table, "src/snapshot/queries/tables.sql")
            .fetch_all(pool)
            .await
    }
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use sqlx::{Executor, PgPool};

    use crate::snapshot::{Snapshot, tables::TableKind};

    #[pgls_test_utils::db_test]
    async fn includes_views_in_query(test_db: PgPool) {
        let setup = r#"
            create table public.base_table (
                id serial primary key,
                value text
            );

            create view public.my_view as
            select * from public.base_table;
        "#;

        test_db
            .execute(setup)
            .await
            .expect("Failed to setup test database");

        let cache = Snapshot::load(&test_db)
            .await
            .expect("Failed to load snapshot");

        let view = cache
            .tables
            .iter()
            .find(|t| t.name == "my_view")
            .expect("View not found");

        assert_eq!(view.table_kind, TableKind::View);
        assert_eq!(view.schema, "public");
    }

    #[pgls_test_utils::db_test]
    async fn includes_materialized_views_in_query(test_db: PgPool) {
        let setup = r#"
            create table public.base_table (
                id serial primary key,
                value text
            );

            create materialized view public.my_mat_view as
            select * from public.base_table;
        "#;

        test_db
            .execute(setup)
            .await
            .expect("Failed to setup test database");

        let cache = Snapshot::load(&test_db)
            .await
            .expect("Failed to load snapshot");

        let mat_view = cache
            .tables
            .iter()
            .find(|t| t.name == "my_mat_view")
            .expect("Materialized view not found");

        assert_eq!(mat_view.table_kind, TableKind::MaterializedView);
        assert_eq!(mat_view.schema, "public");
    }
}
