//! Name resolution for a single statement against a [`CatalogView`].
//!
//! The resolver only reports what it is certain about. Anything that depends on an unknown
//! object, or on a construct the resolver doesn't model, stays silent.

use pgls_text_size::TextRange;

use crate::view::CatalogView;

/// A parameter of the SQL function whose body is being resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParam {
    /// `None` for unnamed parameters, which can only be referenced as `$n`.
    pub name: Option<String>,
    /// Schema of the parameter type, if qualified.
    pub type_schema: Option<String>,
    /// Internal type name, e.g. `int4` or `my_table`.
    pub type_name: String,
    pub is_array: bool,
}

/// The SQL function a statement is the body of. Parameters can be referenced as `name`,
/// `function_name.name`, or `$n`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionContext {
    pub function_name: String,
    pub params: Vec<FunctionParam>,
}

pub struct ResolveParams<'a> {
    pub stmt: &'a pgls_query::NodeEnum,
    pub catalog: &'a dyn CatalogView,
    /// Explicit search path of the session at this statement.
    pub search_path: &'a [String],
    /// Set when `stmt` is the body of a SQL function.
    pub function: Option<&'a FunctionContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingKind {
    UnknownRelation {
        schema: Option<String>,
        name: String,
    },
    UnknownColumn {
        /// The relation or alias the column was looked up in, if qualified.
        relation: Option<String>,
        column: String,
    },
    UnknownSchema {
        name: String,
    },
    AmbiguousColumn {
        column: String,
        /// Aliases or relation names that all have this column.
        candidates: Vec<String>,
    },
    UnknownFunction {
        schema: Option<String>,
        name: String,
        arg_count: usize,
        /// `true` when functions with this name exist, but none accepts `arg_count` arguments.
        name_exists: bool,
    },
    InsertColumnMismatch {
        /// Number of target columns.
        expected: usize,
        /// Number of values per row, or columns of the SELECT.
        found: usize,
    },
    UnknownType {
        schema: Option<String>,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub kind: FindingKind,
    /// Range relative to the start of the statement text, when libpg_query provides a location.
    pub span: Option<TextRange>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resolution {
    pub findings: Vec<Finding>,
    /// `true` when every lookup of this statement found an object with
    /// [`crate::view::Origin::Database`], and no lookup was missing or unknown. Only then does the
    /// database still describe everything the statement references, so the `EXPLAIN` fallback
    /// may check it.
    pub database_only: bool,
}

/// Resolves all names in `params.stmt`.
pub fn resolve(params: ResolveParams<'_>) -> Resolution {
    let _ = params;
    Resolution::default()
}
