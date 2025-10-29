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

#[derive(Debug, Default)]
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
