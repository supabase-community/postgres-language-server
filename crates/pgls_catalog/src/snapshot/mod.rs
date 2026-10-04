//! The database snapshot: the objects of the connected database, as loaded by the queries in
//! `queries/`.

#![allow(dead_code)]

mod casts;
mod columns;
mod extensions;
mod functions;
mod indexes;
mod operators;
mod policies;
mod roles;
mod schemas;
mod search_path;
mod sequences;
mod tables;
mod triggers;
mod types;
mod versions;

pub use casts::PostgresCast;
pub use columns::*;
pub use extensions::Extension;
pub use functions::{Behavior, Function, FunctionArg, FunctionArgs, ProcKind};
pub use indexes::Index;
pub use operators::PostgresOperator;
pub use policies::{Policy, PolicyCommand};
pub use roles::*;
pub use schemas::Schema;
pub use sequences::Sequence;
pub use tables::{ReplicaIdentity, Table, TableKind};
pub use triggers::{Trigger, TriggerAffected, TriggerEvent};
pub use types::{PostgresType, PostgresTypeAttribute, TypeAttributes};
pub use versions::Version;

use serde::{Deserialize, Serialize};
#[cfg(feature = "db")]
use sqlx::postgres::PgPool;

// The JSON schema keeps the name `SchemaCache`, which the WASM package exports.
/// The objects of the connected database, loaded from the database or from JSON.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "SchemaCache"))]
#[serde(default)]
pub struct Snapshot {
    pub schemas: Vec<Schema>,
    pub tables: Vec<Table>,
    pub functions: Vec<Function>,
    pub types: Vec<PostgresType>,
    pub version: Version,
    pub columns: Vec<Column>,
    pub policies: Vec<Policy>,
    pub extensions: Vec<Extension>,
    pub triggers: Vec<Trigger>,
    pub roles: Vec<Role>,
    pub indexes: Vec<Index>,
    pub sequences: Vec<Sequence>,
    #[serde(default)]
    pub casts: Vec<PostgresCast>,
    #[serde(default)]
    pub operators: Vec<PostgresOperator>,
    /// Whether typing metadata was collected (false for legacy JSON snapshots).
    #[serde(default)]
    pub typing_metadata: bool,
}

impl Snapshot {
    #[cfg(feature = "db")]
    pub async fn load(pool: &PgPool) -> Result<Snapshot, sqlx::Error> {
        let (
            schemas,
            tables,
            functions,
            types,
            versions,
            columns,
            policies,
            triggers,
            roles,
            extensions,
            indexes,
            sequences,
            casts,
            operators,
        ) = futures_util::try_join!(
            Schema::load(pool),
            Table::load(pool),
            Function::load(pool),
            PostgresType::load(pool),
            Version::load(pool),
            Column::load(pool),
            Policy::load(pool),
            Trigger::load(pool),
            Role::load(pool),
            Extension::load(pool),
            Index::load(pool),
            Sequence::load(pool),
            PostgresCast::load(pool),
            PostgresOperator::load(pool),
        )?;

        let version = versions
            .into_iter()
            .next()
            .expect("Expected at least one version row");

        Ok(Snapshot {
            schemas,
            tables,
            functions,
            types,
            version,
            columns,
            policies,
            triggers,
            roles,
            extensions,
            indexes,
            sequences,
            casts,
            operators,
            typing_metadata: true,
        })
    }

    pub fn find_schema(&self, name: &str) -> Option<&Schema> {
        let sanitized_name = Self::sanitize_identifier(name);
        self.schemas.iter().find(|s| s.name == sanitized_name)
    }

    pub fn find_tables(&self, name: &str, schema: Option<&str>) -> Vec<&Table> {
        let sanitized_name = Self::sanitize_identifier(name);
        self.tables
            .iter()
            .filter(|t| {
                t.name == sanitized_name
                    && schema
                        .map(Self::sanitize_identifier)
                        .as_deref()
                        .is_none_or(|s| s == t.schema.as_str())
            })
            .collect()
    }

    pub fn find_type(&self, name: &str, schema: Option<&str>) -> Option<&PostgresType> {
        let sanitized_name = Self::sanitize_identifier(name);
        self.types.iter().find(|t| {
            t.name == sanitized_name
                && schema
                    .map(Self::sanitize_identifier)
                    .as_deref()
                    .is_none_or(|s| s == t.schema.as_str())
        })
    }

    pub fn find_type_by_id(&self, id: i64) -> Option<&PostgresType> {
        self.types.iter().find(|t| t.id == id)
    }

    pub fn find_table_by_id(&self, id: i64) -> Option<&Table> {
        self.tables.iter().find(|t| t.id == id)
    }

    pub fn find_function_by_id(&self, id: i64) -> Option<&Function> {
        self.functions.iter().find(|f| f.id == id)
    }

    pub fn find_schema_by_id(&self, id: i64) -> Option<&Schema> {
        self.schemas.iter().find(|s| s.id == id)
    }

    pub fn find_index_by_id(&self, id: i64) -> Option<&Index> {
        self.indexes.iter().find(|i| i.id == id)
    }

    pub fn find_sequence_by_id(&self, id: i64) -> Option<&Sequence> {
        self.sequences.iter().find(|s| s.id == id)
    }

    pub fn find_cols(&self, name: &str, table: Option<&str>, schema: Option<&str>) -> Vec<&Column> {
        let sanitized_name = Self::sanitize_identifier(name);
        self.columns
            .iter()
            .filter(|c| {
                c.name.as_str() == sanitized_name
                    && table
                        .map(Self::sanitize_identifier)
                        .as_deref()
                        .is_none_or(|t| t == c.table_name.as_str())
                    && schema
                        .map(Self::sanitize_identifier)
                        .as_deref()
                        .is_none_or(|s| s == c.schema_name.as_str())
            })
            .collect()
    }

    pub fn find_types(&self, name: &str, schema: Option<&str>) -> Vec<&PostgresType> {
        let sanitized_name = Self::sanitize_identifier(name);
        self.types
            .iter()
            .filter(|t| {
                t.name == sanitized_name
                    && schema
                        .map(Self::sanitize_identifier)
                        .as_deref()
                        .is_none_or(|s| s == t.schema.as_str())
            })
            .collect()
    }

    pub fn find_functions(&self, name: &str, schema: Option<&str>) -> Vec<&Function> {
        let sanitized_name = Self::sanitize_identifier(name);
        self.functions
            .iter()
            .filter(|f| {
                f.name == sanitized_name
                    && schema
                        .map(Self::sanitize_identifier)
                        .as_deref()
                        .is_none_or(|s| s == f.schema.as_str())
            })
            .collect()
    }

    pub fn find_roles(&self, name: &str) -> Vec<&Role> {
        let sanitized_name = Self::sanitize_identifier(name);
        self.roles
            .iter()
            .filter(|r| r.name == sanitized_name)
            .collect()
    }

    fn sanitize_identifier(identifier: &str) -> String {
        identifier.replace('"', "")
    }
}

#[cfg(feature = "db")]
pub(crate) trait SnapshotItem {
    type Item;

    async fn load(pool: &PgPool) -> Result<Vec<Self::Item>, sqlx::Error>;
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use std::collections::HashSet;

    use sqlx::{Executor, PgPool};

    use super::Snapshot;
    use crate::OperatorKind;
    use crate::catalog::{Catalog, CatalogBase};
    use crate::lookup::{CatalogView, Lookup};
    use crate::typing::TypeId;
    use std::sync::Arc;

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn it_loads(test_db: PgPool) {
        Snapshot::load(&test_db)
            .await
            .expect("Failed to load snapshot");
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn typing_snapshot_has_catalog_metadata(test_db: PgPool) {
        test_db.execute("CREATE FUNCTION public.typing_defaults(a int, b int DEFAULT 3) RETURNS int LANGUAGE sql AS 'SELECT a + b'").await.unwrap();
        let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
        assert!(snapshot.typing_metadata);
        let array = snapshot.find_type("_int4", Some("pg_catalog")).unwrap();
        let scalar = snapshot.find_type("int4", Some("pg_catalog")).unwrap();
        assert!(array.is_array);
        assert_eq!(array.typelem, scalar.id);
        assert_eq!(scalar.typarray, array.id);

        let catalog = Catalog::new(Some(Arc::new(CatalogBase::new(snapshot))));
        assert!(matches!(
            catalog.cast(&TypeId::Snapshot(23), &TypeId::Snapshot(20)),
            Lookup::Found(_)
        ));
        assert_eq!(
            catalog.cast(&TypeId::Snapshot(23), &TypeId::Snapshot(25)),
            Lookup::Missing
        );
        assert!(
            catalog
                .operator_candidates(None, "+", OperatorKind::Infix, &[])
                .items
                .iter()
                .any(|op| {
                    op.left == Some(TypeId::Snapshot(23))
                        && op.right == Some(TypeId::Snapshot(23))
                        && op.result == Some(TypeId::Snapshot(23))
                })
        );
        let functions = catalog.function_candidates(None, "now", &[]);
        assert!(
            functions.items.iter().any(|f| f
                .signature
                .as_ref()
                .is_some_and(
                    |s| s.arguments.is_empty() && s.return_type == Some(TypeId::Snapshot(1184))
                ))
        );
        let series = catalog.function_candidates(None, "generate_series", &[]);
        assert!(!series.items.is_empty());
        let defaults = catalog.function_candidates(Some("public"), "typing_defaults", &[]);
        assert!(
            defaults
                .items
                .iter()
                .any(|f| f.signature.as_ref().is_some_and(|s| s.input_defaults == 1))
        );
        let format = catalog.function_candidates(None, "format", &[]);
        assert!(format.items.iter().any(|f| {
            f.signature
                .as_ref()
                .is_some_and(|s| s.variadic_element.is_some())
        }));
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn it_does_not_have_duplicate_entries(test_db: PgPool) {
        // we had some duplicate columns in the snapshot because of indices including the same column multiple times.
        // the columns were unnested as duplicates in the query
        let setup = r#"
        CREATE TABLE public.mfa_factors (
            id uuid PRIMARY KEY,
            factor_name text NOT NULL
        );

        -- a second index on id!
        CREATE INDEX idx_mfa_user_factor ON public.mfa_factors(id, factor_name);
        "#;

        test_db.execute(setup).await.unwrap();

        let cache = Snapshot::load(&test_db)
            .await
            .expect("Failed to load snapshot");

        let set: HashSet<String> = cache
            .columns
            .iter()
            .map(|c| format!("{}.{}.{}", c.schema_name, c.table_name, c.name))
            .collect();

        assert_eq!(set.len(), cache.columns.len());
    }
}
