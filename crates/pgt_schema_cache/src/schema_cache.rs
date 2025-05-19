use sqlx::postgres::PgPool;

use crate::Trigger;
use crate::columns::Column;
use crate::functions::Function;
use crate::policies::Policy;
use crate::schemas::Schema;
use crate::tables::Table;
use crate::types::PostgresType;
use crate::versions::Version;

#[derive(Debug, Default)]
pub struct SchemaCache {
    pub schemas: Vec<Schema>,
    pub tables: Vec<Table>,
    pub functions: Vec<Function>,
    pub types: Vec<PostgresType>,
    pub versions: Vec<Version>,
    pub columns: Vec<Column>,
    pub policies: Vec<Policy>,
    pub triggers: Vec<Trigger>,
}

impl SchemaCache {
    pub async fn load(pool: &PgPool) -> Result<SchemaCache, sqlx::Error> {
        let (schemas, tables, functions, types, versions, columns, policies, triggers) = futures_util::try_join!(
            Schema::load(pool),
            Table::load(pool),
            Function::load(pool),
            PostgresType::load(pool),
            Version::load(pool),
            Column::load(pool),
            Policy::load(pool),
            Trigger::load(pool),
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
        })
    }

    /// Applies an AST node to the repository
    ///
    /// For example, alter table add column will add the column to the table if it does not exist
    /// yet
    pub fn mutate(&mut self) {
        unimplemented!();
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
    use pgt_test_utils::test_database::get_new_test_db;

    use crate::SchemaCache;

    #[tokio::test]
    async fn it_loads() {
        let test_db = get_new_test_db().await;

        SchemaCache::load(&test_db)
            .await
            .expect("Couldnt' load Schema Cache");
    }
}
