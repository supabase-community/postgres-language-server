use serde::{Deserialize, Serialize};
#[cfg(feature = "db")]
use sqlx::PgPool;

#[cfg(feature = "db")]
use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TableKind {
    #[default]
    Ordinary,
    View,
    MaterializedView,
    Partitioned,
}

impl From<char> for TableKind {
    fn from(s: char) -> Self {
        match s {
            'r' => Self::Ordinary,
            'p' => Self::Partitioned,
            'v' => Self::View,
            'm' => Self::MaterializedView,
            _ => panic!("Invalid table kind"),
        }
    }
}

impl From<i8> for TableKind {
    fn from(s: i8) -> Self {
        let c = char::from(u8::try_from(s).unwrap());
        c.into()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
}

#[cfg(feature = "db")]
impl SchemaCacheItem for Table {
    type Item = Table;

    async fn load(pool: &PgPool) -> Result<Vec<Table>, sqlx::Error> {
        sqlx::query_file_as!(Table, "src/queries/tables.sql")
            .fetch_all(pool)
            .await
    }
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use sqlx::{Executor, PgPool};

    use crate::{SchemaCache, tables::TableKind};

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

        let cache = SchemaCache::load(&test_db)
            .await
            .expect("Failed to load Schema Cache");

        let view = cache
            .tables
            .iter()
            .find(|t| t.name == "my_view")
            .expect("View not found");

        assert_eq!(view.table_kind, TableKind::View);
        assert_eq!(view.schema, "public");
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
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

        let cache = SchemaCache::load(&test_db)
            .await
            .expect("Failed to load Schema Cache");

        let mat_view = cache
            .tables
            .iter()
            .find(|t| t.name == "my_mat_view")
            .expect("Materialized view not found");

        assert_eq!(mat_view.table_kind, TableKind::MaterializedView);
        assert_eq!(mat_view.schema, "public");
    }
}
