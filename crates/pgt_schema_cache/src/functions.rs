use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::JsonValue;

use crate::schema_cache::SchemaCacheItem;

/// `Behavior` describes the characteristics of the function. Is it deterministic? Does it changed due to side effects, and if so, when?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Behavior {
    /// The function is a pure function (same input leads to same output.)
    Immutable,

    /// The results of the function do not change within a scan.
    Stable,

    /// The results of the function might change at any time.
    #[default]
    Volatile,
}

impl From<Option<String>> for Behavior {
    fn from(s: Option<String>) -> Self {
        match s {
            Some(s) => match s.as_str() {
                "IMMUTABLE" => Behavior::Immutable,
                "STABLE" => Behavior::Stable,
                "VOLATILE" => Behavior::Volatile,
                _ => panic!("Invalid behavior"),
            },
            None => Behavior::Volatile,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionArg {
    /// `in`, `out`, or `inout`.
    pub mode: String,

    pub name: String,

    /// Refers to the argument type's ID in the `pg_type` table.
    pub type_id: i64,

    pub has_default: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionArgs {
    pub args: Vec<FunctionArg>,
}

impl From<Option<JsonValue>> for FunctionArgs {
    fn from(s: Option<JsonValue>) -> Self {
        let args: Vec<FunctionArg> =
            serde_json::from_value(s.unwrap_or(JsonValue::Array(vec![]))).unwrap();
        FunctionArgs { args }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Function {
    /// The Id (`oid`).
    pub id: i64,

    /// The name of the schema the function belongs to.
    pub schema: String,

    /// The name of the function.
    pub name: String,

    /// e.g. `plpgsql/sql` or `internal`.
    pub language: String,

    /// The body of the function – the `declare [..] begin [..] end [..]` block.` Not set for internal functions.
    pub body: Option<String>,

    /// The full definition of the function. Includes the full `CREATE OR REPLACE...` shenanigans. Not set for internal functions.
    pub definition: Option<String>,

    /// The Rust representation of the function's arguments.
    pub args: FunctionArgs,

    /// Comma-separated list of argument types, in the form required for a CREATE FUNCTION statement. For example, `"text, smallint"`. `None` if the function doesn't take any arguments.
    pub argument_types: Option<String>,

    /// Comma-separated list of argument types, in the form required to identify a function in an ALTER FUNCTION statement. For example, `"text, smallint"`. `None` if the function doesn't take any arguments.
    pub identity_argument_types: Option<String>,

    /// An ID identifying the return type. For example, `2275` refers to `cstring`. 2278 refers to `void`.
    pub return_type_id: i64,

    /// The return type, for example "text", "trigger", or "void".
    pub return_type: String,

    /// If the return type is a composite type, this will point the matching entry's `oid` column in the `pg_class` table. `None` if the function does not return a composite type.
    pub return_type_relation_id: Option<i64>,

    /// Does the function returns multiple values of a data type?
    pub is_set_returning_function: bool,

    /// See `Behavior`.
    pub behavior: Behavior,

    /// Is the function's security set to `Definer` (true) or `Invoker` (false)?
    pub security_definer: bool,
}

impl SchemaCacheItem for Function {
    type Item = Function;

    async fn load(pool: &PgPool) -> Result<Vec<Function>, sqlx::Error> {
        sqlx::query_file_as!(Function, "src/queries/functions.sql")
            .fetch_all(pool)
            .await
    }
}
