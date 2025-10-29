use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::JsonValue;

use crate::schema_cache::SchemaCacheItem;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ProcKind {
    #[default]
    Function,
    Aggregate,
    Window,
    Procedure,
}

impl From<char> for ProcKind {
    fn from(value: char) -> Self {
        match value {
            'f' => Self::Function,
            'p' => Self::Procedure,
            'w' => Self::Window,
            'a' => Self::Aggregate,
            _ => unreachable!(),
        }
    }
}

impl From<i8> for ProcKind {
    fn from(value: i8) -> Self {
        char::from(u8::try_from(value).unwrap()).into()
    }
}

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

    pub kind: ProcKind,

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
    pub return_type_id: Option<i64>,

    /// The return type, for example "text", "trigger", or "void".
    pub return_type: Option<String>,

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

#[cfg(test)]
mod tests {
    use sqlx::{Executor, PgPool};

    use crate::{Behavior, SchemaCache, functions::ProcKind};

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn loads(pool: PgPool) {
        let setup = r#"
          create table coos (
            id serial primary key,
            name text
          );

          create or replace function my_cool_foo()
          returns trigger
          language plpgsql
          security invoker
          as $$
          begin
            raise exception 'dont matter';
          end;
          $$;

          create or replace procedure my_cool_proc()
          language plpgsql
          security invoker
          as $$
          begin
            raise exception 'dont matter';
          end;
          $$;

          create or replace function string_concat_state(
            state text, 
            value text, 
            separator text)
          returns text
          language plpgsql
          as $$
          begin
              if state is null then
                  return value;
              else
                  return state || separator || value;
              end if;
          end;
          $$;

          create aggregate string_concat(text, text) (
            sfunc = string_concat_state,
            stype = text,
            initcond = ''
          );
        "#;

        pool.execute(setup).await.unwrap();

        let cache = SchemaCache::load(&pool).await.unwrap();

        // Find and check the function
        let foo_fn = cache
            .functions
            .iter()
            .find(|f| f.name == "my_cool_foo")
            .unwrap();
        assert_eq!(foo_fn.schema, "public");
        assert_eq!(foo_fn.kind, ProcKind::Function);
        assert_eq!(foo_fn.language, "plpgsql");
        assert_eq!(foo_fn.return_type.as_deref(), Some("trigger"));
        assert!(!foo_fn.security_definer);
        assert_eq!(foo_fn.behavior, Behavior::Volatile);

        // Find and check the procedure
        let proc_fn = cache
            .functions
            .iter()
            .find(|f| f.name == "my_cool_proc")
            .unwrap();

        assert_eq!(proc_fn.kind, ProcKind::Procedure);
        assert_eq!(proc_fn.language, "plpgsql");
        assert!(!proc_fn.security_definer);

        // Find and check the aggregate
        let agg_fn = cache
            .functions
            .iter()
            .find(|f| f.name == "string_concat")
            .unwrap();
        assert_eq!(agg_fn.kind, ProcKind::Aggregate);
        assert_eq!(agg_fn.language, "internal"); // Aggregates are often "internal"
        // The return type should be text
        assert_eq!(agg_fn.return_type.as_deref(), Some("text"));

        // Find and check the state function for the aggregate
        let state_fn = cache
            .functions
            .iter()
            .find(|f| f.name == "string_concat_state")
            .unwrap();

        assert_eq!(state_fn.kind, ProcKind::Function);
        assert_eq!(state_fn.language, "plpgsql");
        assert_eq!(state_fn.return_type.as_deref(), Some("text"));
        assert_eq!(state_fn.args.args.len(), 3);
        let arg_names: Vec<_> = state_fn.args.args.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(arg_names, &["state", "value", "separator"]);
    }
}
