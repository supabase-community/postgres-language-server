use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
#[cfg(feature = "db")]
use sqlx::PgPool;

#[cfg(feature = "db")]
use super::SnapshotItem;
use super::code::Code;
use crate::FunctionArgumentMode;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ProcKind {
    #[default]
    Function,
    Aggregate,
    Window,
    Procedure,
    /// A `pg_proc.prokind` we don't know.
    #[serde(other)]
    #[cfg_attr(feature = "schema", schemars(skip))]
    Unknown,
}

// `pg_proc.prokind`, a `"char"`.
impl From<i8> for ProcKind {
    fn from(code: i8) -> Self {
        match code.cast_unsigned() {
            b'f' => Self::Function,
            b'p' => Self::Procedure,
            b'w' => Self::Window,
            b'a' => Self::Aggregate,
            _ => Self::Unknown,
        }
    }
}

/// `Behavior` describes the characteristics of the function. Is it deterministic? Does it changed due to side effects, and if so, when?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    // `functions.sql` gives no behavior for a `provolatile` it doesn't know. Like then, a
    // function is volatile unless it is known not to be.
    fn from(s: Option<String>) -> Self {
        match s.as_deref() {
            Some("IMMUTABLE") => Behavior::Immutable,
            Some("STABLE") => Behavior::Stable,
            _ => Behavior::Volatile,
        }
    }
}

/// The mode of a function argument, as `functions.sql` gives it for `pg_proc.proargmodes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentMode {
    In,
    Out,
    InOut,
    Variadic,
    /// A column of `RETURNS TABLE`.
    Table,
}

impl ArgumentMode {
    /// The mode in the catalog: the columns of `RETURNS TABLE` are output arguments.
    pub fn function_argument_mode(self) -> FunctionArgumentMode {
        match self {
            Self::In => FunctionArgumentMode::In,
            Self::Out | Self::Table => FunctionArgumentMode::Out,
            Self::InOut => FunctionArgumentMode::InOut,
            Self::Variadic => FunctionArgumentMode::Variadic,
        }
    }
}

impl From<FunctionArgumentMode> for ArgumentMode {
    fn from(mode: FunctionArgumentMode) -> Self {
        match mode {
            FunctionArgumentMode::In => Self::In,
            FunctionArgumentMode::Out => Self::Out,
            FunctionArgumentMode::InOut => Self::InOut,
            FunctionArgumentMode::Variadic => Self::Variadic,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FunctionArg {
    /// `in`, `out`, `inout`, `variadic` or `table`.
    pub mode: Code<ArgumentMode>,

    pub name: String,

    /// Refers to the argument type's ID in the `pg_type` table.
    pub type_id: i64,

    pub has_default: Option<bool>,
}

impl FunctionArg {
    /// The argument's mode in the catalog, `None` when the snapshot's mode is unknown.
    pub fn mode(&self) -> Option<FunctionArgumentMode> {
        self.mode.get().map(ArgumentMode::function_argument_mode)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FunctionArgs {
    pub args: Vec<FunctionArg>,
}

impl From<Option<JsonValue>> for FunctionArgs {
    // `functions.sql` builds the array, so it has this shape. Should it ever not, the function
    // has no arguments rather than the server panicking.
    fn from(s: Option<JsonValue>) -> Self {
        let args = s
            .map(serde_json::from_value::<Vec<FunctionArg>>)
            .transpose()
            .unwrap_or_else(|error| {
                tracing::error!("Unexpected function arguments in the snapshot: {error}");
                None
            })
            .unwrap_or_default();
        FunctionArgs { args }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Function {
    /// The Id (`oid`).
    pub id: i64,

    /// Number of trailing input arguments with defaults.
    #[serde(default)]
    pub input_defaults: i16,

    /// Variadic argument element type oid, or zero when not variadic.
    #[serde(default)]
    pub variadic_type_id: i64,

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

impl Function {
    /// The arguments a call passes: the input arguments, and for a procedure its output
    /// arguments too, which `CALL` passes (`FuncnameGetCandidates` with `include_out_arguments`):
    /// https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L1272
    pub fn call_arguments(&self) -> impl Iterator<Item = &FunctionArg> {
        let is_procedure = self.kind == ProcKind::Procedure;
        self.args.args.iter().filter(move |arg| {
            arg.mode()
                .is_some_and(|mode| mode.is_call_argument(is_procedure))
        })
    }
}

#[cfg(feature = "db")]
impl SnapshotItem for Function {
    type Item = Function;

    async fn load(pool: &PgPool) -> Result<Vec<Function>, sqlx::Error> {
        sqlx::query_file_as!(Function, "src/snapshot/queries/functions.sql")
            .fetch_all(pool)
            .await
    }
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use sqlx::{Executor, PgPool};

    use crate::snapshot::{Behavior, Snapshot, functions::ProcKind};

    #[pgls_test_utils::db_test]
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

        let cache = Snapshot::load(&pool).await.unwrap();

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
