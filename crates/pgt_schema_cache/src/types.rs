use serde::Deserialize;
use sqlx::PgPool;
use sqlx::types::JsonValue;

use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Clone, Default)]
pub struct TypeAttributes {
    pub attrs: Vec<PostgresTypeAttribute>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PostgresTypeAttribute {
    pub name: String,
    pub type_id: i64,
}

impl From<Option<JsonValue>> for TypeAttributes {
    fn from(s: Option<JsonValue>) -> Self {
        let values: Vec<PostgresTypeAttribute> =
            serde_json::from_value(s.unwrap_or(JsonValue::Array(vec![]))).unwrap();
        TypeAttributes { attrs: values }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Enums {
    pub values: Vec<String>,
}

impl From<Option<JsonValue>> for Enums {
    fn from(s: Option<JsonValue>) -> Self {
        let values: Vec<String> =
            serde_json::from_value(s.unwrap_or(JsonValue::Array(vec![]))).unwrap();
        Enums { values }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PostgresType {
    pub id: i64,
    pub name: String,
    pub schema: String,
    pub format: String,
    pub enums: Enums,
    pub attributes: TypeAttributes,
    pub comment: Option<String>,
}

impl SchemaCacheItem for PostgresType {
    type Item = PostgresType;

    async fn load(pool: &PgPool) -> Result<Vec<PostgresType>, sqlx::Error> {
        sqlx::query_file_as!(PostgresType, "src/queries/types.sql")
            .fetch_all(pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use pgt_test_utils::test_database::get_new_test_db;
    use sqlx::Executor;

    use crate::{schema_cache::SchemaCacheItem, types::PostgresType};

    #[tokio::test]
    async fn test_types() {
        let setup = r#"
            CREATE TYPE "public"."priority" AS ENUM (
                'critical',
                'high',
                'default',
                'low',
                'very_low'
            );

            CREATE TYPE complex AS (
                r       double precision,
                i       double precision
            );
        "#;

        let test_db = get_new_test_db().await;

        test_db
            .execute(setup)
            .await
            .expect("Failed to setup test database");

        let types = PostgresType::load(&test_db).await.unwrap();

        let enum_type = types.iter().find(|t| t.name == "priority");
        let comp_type = types.iter().find(|t| t.name == "complex");

        println!("{:?}", enum_type);
        // search for type id
        println!("{:?}", comp_type);

        comp_type.and_then(|t| {
            t.attributes.attrs.iter().for_each(|a| {
                let typ = types.iter().find(|t| t.id == a.type_id);
                println!(
                    "{}: {} - {:?}",
                    a.name,
                    a.type_id,
                    typ.as_ref().map(|t| t.name.clone())
                );
            });
            Some(())
        });
    }
}
