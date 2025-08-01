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
    pub versions: Vec<Version>,
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

        Ok(SchemaCache {
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
        })
    }

    pub fn find_table(&self, name: &str, schema: Option<&str>) -> Option<&Table> {
        self.tables
            .iter()
            .find(|t| t.name == name && schema.is_none() || Some(t.schema.as_str()) == schema)
    }

    pub fn find_type(&self, name: &str, schema: Option<&str>) -> Option<&PostgresType> {
        self.types
            .iter()
            .find(|t| t.name == name && schema.is_none() || Some(t.schema.as_str()) == schema)
    }

    pub fn find_col(&self, name: &str, table: &str, schema: Option<&str>) -> Option<&Column> {
        self.columns.iter().find(|c| {
            c.name.as_str() == name
                && c.table_name.as_str() == table
                && schema.is_none_or(|s| s == c.schema_name.as_str())
        })
    }

    pub fn find_types(&self, name: &str, schema: Option<&str>) -> Vec<&PostgresType> {
        self.types
            .iter()
            .filter(|t| t.name == name && schema.is_none() || Some(t.schema.as_str()) == schema)
            .collect()
    }
}

pub trait SchemaCacheItem {
    type Item;

    async fn load(pool: &PgPool) -> Result<Vec<Self::Item>, sqlx::Error>;
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::SchemaCache;

    #[sqlx::test(migrator = "pgt_test_utils::MIGRATIONS")]
    async fn it_loads(test_db: PgPool) {
        SchemaCache::load(&test_db)
            .await
            .expect("Couldnt' load Schema Cache");
    }
}
