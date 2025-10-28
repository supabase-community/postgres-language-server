use sqlx::postgres::PgPool;

use crate::columns::Column;
use crate::functions::Function;
use crate::policies::Policy;
use crate::schemas::Schema;
use crate::tables::Table;
use crate::types::PostgresType;
use crate::versions::Version;
use crate::{Extension, Role, Trigger};

#[derive(Debug, Default)]
pub struct SchemaCache {
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
}

impl SchemaCache {
    pub async fn load(pool: &PgPool) -> Result<SchemaCache, sqlx::Error> {
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
        )?;

        let version = versions
            .into_iter()
            .next()
            .expect("Expected at least one version row");

        Ok(SchemaCache {
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

pub trait SchemaCacheItem {
    type Item;

    async fn load(pool: &PgPool) -> Result<Vec<Self::Item>, sqlx::Error>;
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use sqlx::{Executor, PgPool};

    use crate::SchemaCache;

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn it_loads(test_db: PgPool) {
        SchemaCache::load(&test_db)
            .await
            .expect("Couldnt' load Schema Cache");
    }

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn it_does_not_have_duplicate_entries(test_db: PgPool) {
        // we had some duplicate columns in the schema_cache because of indices including the same column multiple times.
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

        let cache = SchemaCache::load(&test_db)
            .await
            .expect("Couldn't load Schema Cache");

        let set: HashSet<String> = cache
            .columns
            .iter()
            .map(|c| format!("{}.{}.{}", c.schema_name, c.table_name, c.name))
            .collect();

        assert_eq!(set.len(), cache.columns.len());
    }
}
