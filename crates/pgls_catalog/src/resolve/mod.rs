//! Conservative name resolution of a single statement against a [`CatalogView`].
//!
//! The resolver reports a [`Finding`] only when Postgres would certainly raise the error. Whenever
//! something is not known with certainty (an unknown object, a relation whose columns are not
//! known, a construct the resolver doesn't model), it stays silent.
//!
//! It also decides whether a statement is [`Resolution::database_only`]: whether everything it
//! references comes unchanged from the database, so that checking it against the database gives
//! the right answer.

mod dml;
mod expr;
mod from;
mod query;
mod scope;
mod span;
#[cfg(test)]
mod tests;

use pgls_query::{NodeEnum, protobuf};
use pgls_text_size::{TextRange, TextSize};

use crate::view::{CatalogView, Lookup, Origin};
use scope::Cte;

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
    let mut resolver = Resolver {
        catalog: params.catalog,
        search_path: params.search_path,
        function: params.function,
        sql: params.sql,
        findings: Vec::new(),
        database_only: true,
    };

    let supports_database_check = resolver.statement(params.stmt);
    if let Some(function) = params.function {
        resolver.check_function_params(function);
    }

    let mut findings: Vec<Finding> = Vec::new();
    for finding in resolver.findings {
        if !findings.contains(&finding) {
            findings.push(finding);
        }
    }

    Resolution {
        findings,
        database_only: supports_database_check && resolver.database_only,
    }
}

/// The output column names of a query, or `None` if they are not known with certainty.
pub fn query_output_columns(
    query: &protobuf::Node,
    catalog: &dyn CatalogView,
    search_path: &[String],
) -> Option<Vec<String>> {
    let mut resolver = Resolver {
        catalog,
        search_path,
        function: None,
        sql: None,
        findings: Vec::new(),
        database_only: true,
    };
    resolver.query(query, &[], &[])
}

struct Resolver<'a> {
    catalog: &'a dyn CatalogView,
    search_path: &'a [String],
    function: Option<&'a FunctionContext>,
    sql: Option<&'a str>,
    findings: Vec<Finding>,
    /// Cleared as soon as the statement references something that doesn't come unchanged from
    /// the database, or something the resolver doesn't understand.
    database_only: bool,
}

impl Resolver<'_> {
    /// Resolves a statement. Returns whether the statement kind can be checked against the
    /// database.
    fn statement(&mut self, stmt: &NodeEnum) -> bool {
        match stmt {
            NodeEnum::SelectStmt(select) => {
                self.select(select, &[], &[]);
                select.into_clause.is_none()
            }
            NodeEnum::InsertStmt(insert) => {
                self.insert(insert, &[]);
                true
            }
            NodeEnum::UpdateStmt(update) => {
                self.update(update, &[]);
                true
            }
            NodeEnum::DeleteStmt(delete) => {
                self.delete(delete, &[]);
                true
            }
            // The query of `CREATE TABLE AS` and views is checked on its own.
            NodeEnum::CreateTableAsStmt(stmt) => {
                if let Some(query) = stmt.query.as_deref() {
                    self.query(query, &[], &[]);
                }
                matches!(
                    stmt.query.as_deref().and_then(|query| query.node.as_ref()),
                    Some(NodeEnum::SelectStmt(_))
                )
            }
            NodeEnum::ViewStmt(stmt) => {
                if let Some(query) = stmt.query.as_deref() {
                    self.query(query, &[], &[]);
                }
                true
            }
            NodeEnum::ExplainStmt(stmt) => {
                if let Some(inner) = stmt.query.as_deref().and_then(|query| query.node.as_ref()) {
                    self.statement(inner);
                }
                false
            }
            NodeEnum::DeclareCursorStmt(stmt) => {
                if let Some(query) = stmt.query.as_deref() {
                    self.query(query, &[], &[]);
                }
                false
            }
            NodeEnum::CopyStmt(stmt) => {
                if let Some(query) = stmt.query.as_deref().and_then(|query| query.node.as_ref()) {
                    self.statement(query);
                } else if let Some(relation) = &stmt.relation {
                    self.relation(relation);
                }
                false
            }
            _ => false,
        }
    }

    /// SQL function bodies are only checked against the database if their parameters have
    /// scalar types from the database: composite parameters can't be replaced by literals.
    fn check_function_params(&mut self, function: &FunctionContext) {
        for param in &function.params {
            let lookup = self.catalog.type_(
                param.type_schema.as_deref(),
                &param.type_name,
                self.search_path,
            );
            match lookup {
                Lookup::Found(type_info)
                    if type_info.origin == Origin::Database && type_info.attributes.is_none() => {}
                _ => self.database_only = false,
            }
        }
    }

    /// Marks the statement as depending on something that is not an unchanged database object.
    fn depends_on_file(&mut self) {
        self.database_only = false;
    }

    /// Records the origin of a found object.
    fn uses(&mut self, origin: Origin) {
        if origin != Origin::Database {
            self.database_only = false;
        }
    }

    fn report(&mut self, kind: FindingKind, location: i32) {
        let span = self.span(location);
        self.database_only = false;
        self.findings.push(Finding { kind, span });
    }

    /// The span of the (possibly qualified) name starting at `location`.
    fn span(&self, location: i32) -> Option<TextRange> {
        let start = usize::try_from(location).ok()?;
        let sql = self.sql?;
        let end = span::reference_end(sql, start)?;
        Some(TextRange::new(
            TextSize::try_from(start).ok()?,
            TextSize::try_from(end).ok()?,
        ))
    }

    /// Reports an unknown schema. Returns `true` if the schema is known to be missing.
    fn check_schema(&mut self, schema: &str, location: i32) -> bool {
        match self.catalog.schema(schema) {
            Lookup::Missing => {
                self.report(
                    FindingKind::UnknownSchema {
                        name: schema.to_owned(),
                    },
                    location,
                );
                true
            }
            Lookup::Unknown => {
                self.depends_on_file();
                false
            }
            Lookup::Found(()) => false,
        }
    }
}

/// The value of a `String` node.
fn string_value(node: &protobuf::Node) -> Option<&str> {
    match node.node.as_ref()? {
        NodeEnum::String(value) => Some(&value.sval),
        _ => None,
    }
}

/// All values of a list of `String` nodes. `None` if a node is not a string.
fn string_values(nodes: &[protobuf::Node]) -> Option<Vec<String>> {
    nodes
        .iter()
        .map(|node| string_value(node).map(str::to_owned))
        .collect()
}

/// Applies the column names of an alias (`AS t(a, b)`) to the first columns.
fn apply_column_aliases(
    columns: Option<Vec<String>>,
    alias: Option<&protobuf::Alias>,
) -> Option<Vec<String>> {
    let Some(alias) = alias.filter(|alias| !alias.colnames.is_empty()) else {
        return columns;
    };
    let mut columns = columns?;
    let names = string_values(&alias.colnames)?;
    if names.len() > columns.len() {
        return None;
    }
    for (column, name) in columns.iter_mut().zip(names) {
        *column = name;
    }
    Some(columns)
}

/// Common table expressions visible to a query, innermost last.
type Ctes<'a> = &'a [Cte];
