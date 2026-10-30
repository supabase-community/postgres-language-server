//! Conservative name resolution of a single statement against a [`CatalogView`].
//!
//! The resolver reports a [`Finding`] only when Postgres would certainly raise the error. Whenever
//! something is not known with certainty (an unknown object, a relation whose columns are not
//! known, a construct the resolver doesn't model), it stays silent.
//!
//! Not modelled yet, so never reported: the columns of scalar functions in `FROM`, `NATURAL`
//! joins, `XMLTABLE`/`JSON_TABLE` and `TABLESAMPLE`, the `ORDER BY` of set operations, recursive
//! CTEs and `SEARCH`/`CYCLE`, and statements with psql identifier parameters (`:schema.table`).
//!
//! It also decides whether a statement is [`Resolution::database_only`]: whether everything it
//! references comes unchanged from the database, so that checking it against the database gives
//! the right answer.

mod column_name;
pub use column_name::figure_column_name;
#[cfg(all(test, feature = "db"))]
mod differential_tests;
pub(crate) mod expr;
mod nodes;
#[cfg(all(test, feature = "db"))]
mod reference_tests;
mod resolver;
mod scope;
mod span;
#[cfg(test)]
mod tests;
#[cfg(all(test, feature = "db"))]
mod type_tests;

use pgls_query::{NodeEnum, protobuf};
use pgls_text_size::TextRange;

use crate::lookup::{CatalogView, Lookup};
use crate::typing::{QueryColumns, Type};
use resolver::Resolver;

/// A parameter of the SQL function whose body is being resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParam {
    pub name: Option<String>,
    pub type_schema: Option<String>,
    pub type_name: String,
    pub is_array: bool,
}

/// The SQL function whose body is being resolved. Its parameters are in scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionContext {
    pub function_name: String,
    pub params: Vec<FunctionParam>,
}

pub struct ResolveParams<'a> {
    pub stmt: &'a NodeEnum,
    pub catalog: &'a dyn CatalogView,
    /// The explicit search path of the session.
    pub search_path: &'a [String],
    /// Set when the statement is the body of a SQL function.
    pub function: Option<&'a FunctionContext>,
    /// The statement text, used to compute the spans of findings and references.
    pub sql: Option<&'a str>,
    /// Whether to collect the [`Resolution::references`].
    pub collect_references: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingKind {
    UnknownRelation {
        schema: Option<String>,
        name: String,
    },
    UnknownColumn {
        relation: Option<String>,
        column: String,
    },
    UnknownSchema {
        name: String,
    },
    AmbiguousColumn {
        column: String,
        candidates: Vec<String>,
    },
    OperatorMismatch {
        operator: String,
        left: Option<crate::typing::Type>,
        right: crate::typing::Type,
        failure: MatchFailure,
    },
    FunctionArgumentMismatch {
        schema: Option<String>,
        name: String,
        args: Vec<crate::typing::Type>,
        failure: MatchFailure,
    },
    InvalidCast {
        from: crate::typing::Type,
        to: crate::typing::Type,
    },
    AssignmentMismatch {
        column: String,
        column_type: crate::typing::Type,
        expr_type: crate::typing::Type,
    },
    UnknownFunction {
        schema: Option<String>,
        name: String,
        arg_count: usize,
        /// Whether functions with this name exist, but none accepts `arg_count` arguments.
        name_exists: bool,
    },
    InsertColumnMismatch {
        expected: usize,
        found: usize,
    },
    UnknownType {
        schema: Option<String>,
        name: String,
    },
    MissingFromClauseEntry {
        name: String,
    },
    /// The final statement of a SQL function doesn't return what the function is declared to
    /// return.
    FunctionReturnMismatch {
        /// The declared result type, e.g. `users`, `int4`, or `record`.
        declared: String,
        mismatch: ReturnMismatch,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchFailure {
    NoMatch,
    Ambiguous,
}

/// How the final statement of a SQL function differs from the declared result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnMismatch {
    /// It returns `found` columns instead of `expected`.
    ColumnCount { expected: usize, found: usize },
    /// It is not a query, so it returns no rows.
    NoRows,
    /// A column can't be assigned to its declared type. `position` is the 1-based column
    /// of a composite result, `None` for a scalar result.
    ColumnType {
        position: Option<usize>,
        expected: crate::typing::Type,
        found: crate::typing::Type,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub kind: FindingKind,
    /// The span relative to the start of the statement, if known.
    pub span: Option<TextRange>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resolution {
    pub findings: Vec<Finding>,
    /// `true` if every relation, function, and type the statement references exists unchanged
    /// in the database, and the statement contains nothing the resolver doesn't understand.
    pub database_only: bool,
    /// The names that resolve to exactly one object, in the order the resolver visits them.
    /// Empty unless [`ResolveParams::collect_references`] is set.
    pub references: Vec<Reference>,
}

/// A name in the statement that resolves to exactly one object. Names that match no object or
/// several, or whose object can't be determined with certainty, have no reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    /// A function call.
    Function {
        /// The location of the function name in the statement.
        location: i32,
        /// The span of the (possibly qualified) function name.
        span: Option<TextRange>,
        function: crate::FunctionInfo,
        /// The parameter that each argument of the call is passed to, in the order of the call's
        /// arguments. Expanded variadic arguments all go to the variadic parameter.
        arguments: Vec<ResolvedCallArgument>,
    },
    /// A column reference, like `name`, `u.name`, or `public.users.name`.
    Column {
        /// The location of the column reference in the statement.
        location: i32,
        /// The span of the whole column reference, including its qualifiers.
        span: Option<TextRange>,
        source: ColumnSource,
        /// The column as its source defines it: under its own name, even if an alias renames it
        /// (`users u(user_id)`). Columns of CTEs and subqueries have their inferred type.
        column: crate::ColumnInfo,
    },
    /// A relation of the catalog, in FROM, or the target of INSERT, UPDATE, DELETE, or COPY.
    /// CTEs referenced by name are not relations of the catalog, and have no reference.
    Relation {
        /// The location of the relation name in the statement.
        location: i32,
        /// The span of the (possibly qualified) relation name, without its alias.
        span: Option<TextRange>,
        relation: crate::RelationInfo,
    },
    /// The type of a cast, like `int` in `x::int` or `date` in `date 'today'`.
    Type {
        /// The location of the type name in the statement.
        location: i32,
        /// The span of the (possibly qualified) type name, with every word of multi-word types
        /// like `double precision`, and without modifiers like `(10)` or `[]`. The precision of
        /// `timestamp(3) with time zone` is inside the span.
        span: Option<TextRange>,
        type_info: crate::TypeInfo,
    },
    /// A parameter of the SQL function whose body is resolved, by name (`id`, `fn.id`) or by
    /// number (`$1`).
    Parameter {
        /// The location of the parameter reference in the statement.
        location: i32,
        /// The span of the whole parameter reference, including the function name qualifier.
        span: Option<TextRange>,
        /// The index of the parameter in [`FunctionContext::params`].
        index: usize,
    },
    /// A string literal that is coerced to an enum type, like `'happy'` in `mood = 'happy'`.
    /// Only recorded where the enum type is certain: casts, operators whose other operand is the
    /// enum, function parameters of the enum type, and values assigned to enum columns in INSERT
    /// and UPDATE.
    EnumLiteral {
        /// The location of the literal in the statement.
        location: i32,
        /// The span of the literal, including its quotes.
        span: Option<TextRange>,
        type_info: crate::TypeInfo,
    },
    /// The name of a common table expression, where the WITH clause defines it and where FROM
    /// references it.
    Cte {
        /// The location of the name in the statement.
        location: i32,
        /// The span of the name.
        span: Option<TextRange>,
        name: String,
    },
    /// The alias of a table or CTE in FROM, or of the target of INSERT, UPDATE, or DELETE, where
    /// it is defined, like `u` in `from users as u`. References to it, like `u` in `u.name`,
    /// are part of their column reference.
    Alias {
        /// The location of the alias in the statement.
        location: i32,
        /// The span of the alias.
        span: Option<TextRange>,
        name: String,
    },
    /// A `*` or `t.*` in a select list or RETURNING list whose columns are all known, and each
    /// can be referenced by a name that resolves to it.
    Star {
        /// The location of the `*`, or of the qualifier of `t.*`.
        location: i32,
        /// The span of the `*`, including its qualifier.
        span: Option<TextRange>,
        /// The columns that the `*` expands to, in order.
        columns: Vec<StarColumn>,
    },
}

/// A column that a `*` expands to, and the name that references it from where the `*` is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarColumn {
    /// The qualifier of the column, e.g. `["u"]` for `u.name`. Empty if the column name alone
    /// references it.
    pub qualifier: Vec<String>,
    pub name: String,
}

impl Reference {
    pub fn location(&self) -> i32 {
        match self {
            Self::Function { location, .. }
            | Self::Column { location, .. }
            | Self::Relation { location, .. }
            | Self::Type { location, .. }
            | Self::Parameter { location, .. }
            | Self::EnumLiteral { location, .. }
            | Self::Cte { location, .. }
            | Self::Alias { location, .. }
            | Self::Star { location, .. } => *location,
        }
    }

    pub fn span(&self) -> Option<TextRange> {
        match self {
            Self::Function { span, .. }
            | Self::Column { span, .. }
            | Self::Relation { span, .. }
            | Self::Type { span, .. }
            | Self::Parameter { span, .. }
            | Self::EnumLiteral { span, .. }
            | Self::Cte { span, .. }
            | Self::Alias { span, .. }
            | Self::Star { span, .. } => *span,
        }
    }
}

/// Where a referenced column comes from. Only the FROM item that a column reference resolves to
/// is followed, not where the columns of a CTE or subquery come from in turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnSource {
    /// A table, view, or other relation of the catalog.
    Relation { schema: String, name: String },
    /// A common table expression, by its name rather than the alias that references it.
    Cte { name: String },
    /// A subquery in FROM, by its alias. `None` if it has none.
    Subquery { alias: Option<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedCallArgument {
    /// The index of the parameter in the arguments of the function's signature.
    pub parameter: usize,
    /// Whether the argument is passed by name, as in `f(a => 1)`.
    pub named: bool,
}

/// Resolves the names of a statement.
pub fn resolve(params: ResolveParams<'_>) -> Resolution {
    let mut resolver = Resolver::new(
        params.catalog,
        params.search_path,
        params.function,
        params.sql,
    );
    if params.collect_references {
        resolver.references = Some(Vec::new());
    }
    nodes::resolve_node_enum(&mut resolver, params.stmt);
    resolver.check_function_params();

    let mut findings: Vec<Finding> = Vec::new();
    for finding in resolver.findings {
        if !findings.contains(&finding) {
            findings.push(finding);
        }
    }

    Resolution {
        findings,
        database_only: supports_database_check(params.stmt) && resolver.database_only,
        references: resolver.references.unwrap_or_default(),
    }
}

/// The output column names of a query, or `None` if they are not known with certainty.
pub fn query_output_columns(
    query: &protobuf::Node,
    catalog: &dyn CatalogView,
    search_path: &[String],
) -> Option<Vec<String>> {
    nodes::resolve_node(&mut Resolver::new(catalog, search_path, None, None), query)
}

/// The inferred output names and types of a query, or `None` when its shape is unknown.
pub fn query_output_types(
    query: &protobuf::Node,
    catalog: &dyn CatalogView,
    search_path: &[String],
) -> QueryColumns {
    let mut resolver = Resolver::new(catalog, search_path, None, None);
    nodes::resolve_node(&mut resolver, query);
    resolve_unknown_outputs(catalog, resolver.output)
}

/// Port of [`resolveTargetListUnknowns`]: output columns of unknown type become
/// text. Postgres does this for every query except the source of an INSERT and the branches of a
/// set operation, which coerce the unknowns to their target type instead.
///
/// [`resolveTargetListUnknowns`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_target.c#L288
pub(crate) fn resolve_unknown_outputs(
    catalog: &dyn CatalogView,
    columns: QueryColumns,
) -> QueryColumns {
    let mut columns = columns?;
    if columns
        .iter()
        .any(|column| column.ty == Some(Type::UnknownLiteral))
    {
        let text = match catalog.type_(Some("pg_catalog"), "text", &[]) {
            Lookup::Found(info) => info.id.map(Type::Named),
            _ => None,
        };
        for column in &mut columns {
            if column.ty == Some(Type::UnknownLiteral) {
                column.ty = text.clone();
            }
        }
    }
    Some(columns)
}

/// Whether the statement kind can be checked against the database with `EXPLAIN`.
fn supports_database_check(stmt: &NodeEnum) -> bool {
    match stmt {
        NodeEnum::SelectStmt(select) => select.into_clause.is_none(),
        NodeEnum::InsertStmt(_) | NodeEnum::UpdateStmt(_) | NodeEnum::DeleteStmt(_) => true,
        NodeEnum::CreateTableAsStmt(stmt) => matches!(
            stmt.query.as_deref().and_then(|query| query.node.as_ref()),
            Some(NodeEnum::SelectStmt(_))
        ),
        NodeEnum::ViewStmt(_) => true,
        _ => false,
    }
}
