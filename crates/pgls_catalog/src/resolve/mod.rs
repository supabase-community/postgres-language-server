//! Conservative name resolution of a single statement against a [`CatalogView`].
//!
//! The resolver reports a [`Finding`] only when Postgres would certainly raise the error. Whenever
//! something is not known with certainty (an unknown object, a relation whose columns are not
//! known, a construct the resolver doesn't model), it stays silent.
//!
//! It also decides whether a statement is [`Resolution::database_only`]: whether everything it
//! references comes unchanged from the database, so that checking it against the database gives
//! the right answer.

mod column_name;
#[cfg(all(test, feature = "db"))]
mod differential_tests;
pub(crate) mod expr;
mod nodes;
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
    /// The statement text, used to compute the spans of findings.
    pub sql: Option<&'a str>,
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
}

/// Resolves the names of a statement.
pub fn resolve(params: ResolveParams<'_>) -> Resolution {
    let mut resolver = Resolver::new(
        params.catalog,
        params.search_path,
        params.function,
        params.sql,
    );
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

/// Port of `parse_target.c: resolveTargetListUnknowns`: output columns of unknown type become
/// text. Postgres does this for every query except the source of an INSERT and the branches of a
/// set operation, which coerce the unknowns to their target type instead.
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
