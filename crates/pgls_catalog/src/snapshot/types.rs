use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
#[cfg(feature = "db")]
use sqlx::PgPool;

#[cfg(feature = "db")]
use super::SnapshotItem;
use super::code::Code;
use crate::TypeKind;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TypeAttributes {
    pub attrs: Vec<PostgresTypeAttribute>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PostgresTypeAttribute {
    pub name: String,
    pub type_id: i64,
}

impl From<Option<JsonValue>> for TypeAttributes {
    // `types.sql` builds the array, so it has this shape. Should it ever not, the type has no
    // attributes rather than the server panicking.
    fn from(s: Option<JsonValue>) -> Self {
        let attrs = s
            .map(serde_json::from_value::<Vec<PostgresTypeAttribute>>)
            .transpose()
            .unwrap_or_else(|error| {
                tracing::error!("Unexpected type attributes in the snapshot: {error}");
                None
            })
            .unwrap_or_default();
        TypeAttributes { attrs }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Enums {
    pub values: Vec<String>,
}

impl From<Option<JsonValue>> for Enums {
    // `types.sql` builds the array, so it has this shape. Should it ever not, the enum has no
    // labels rather than the server panicking.
    fn from(s: Option<JsonValue>) -> Self {
        let values = s
            .map(serde_json::from_value::<Vec<String>>)
            .transpose()
            .unwrap_or_else(|error| {
                tracing::error!("Unexpected enum labels in the snapshot: {error}");
                None
            })
            .unwrap_or_default();
        Enums { values }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PostgresType {
    pub id: i64,
    /// `pg_type.typtype`.
    #[serde(default)]
    pub typtype: Code<TypeKind>,
    #[serde(default)]
    pub typcategory: String,
    #[serde(default)]
    pub typispreferred: bool,
    #[serde(default)]
    pub typelem: i64,
    #[serde(default)]
    pub typarray: i64,
    #[serde(default)]
    pub typbasetype: i64,
    #[serde(default)]
    pub typrelid: i64,
    #[serde(default)]
    pub is_array: bool,
    pub name: String,
    pub schema: String,
    pub format: String,
    pub enums: Enums,
    pub attributes: TypeAttributes,
    pub comment: Option<String>,
}

#[cfg(feature = "db")]
impl SnapshotItem for PostgresType {
    type Item = PostgresType;

    async fn load(pool: &PgPool) -> Result<Vec<PostgresType>, sqlx::Error> {
        sqlx::query_file_as!(PostgresType, "src/snapshot/queries/types.sql")
            .fetch_all(pool)
            .await
    }
}
