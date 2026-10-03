use crate::OperatorInfo;

/// A stable identity for a database or file-created type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeId {
    /// A type from the database snapshot, by oid.
    Snapshot(i64),
    /// A synthetic type created by the current file.
    File(u64),
}

/// A known SQL type value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// A named database type.
    Named(TypeId),
    /// Postgres' unresolved literal type.
    UnknownLiteral,
    /// An anonymous row.
    Record(Vec<TypedColumn>),
}

/// A named or unnamed typed output column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedColumn {
    pub name: String,
    pub ty: Option<Type>,
}

/// Output columns of a query; inner unknown types are preserved.
pub type QueryColumns = Option<Vec<TypedColumn>>;

/// A result that may not be known statically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision<T> {
    Known(T),
    Unknown,
}

/// A candidate selection result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection<T> {
    Match(T),
    NoMatch,
    Ambiguous,
    Unknown,
}

/// The coercion context requested by Postgres.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoercionContext {
    Implicit,
    Assignment,
    Explicit,
}

/// A selected function and its resolved result type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFunction {
    pub function: crate::FunctionInfo,
    pub result: Option<Type>,
}

/// A selected operator and its resolved result type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedOperator {
    pub operator: OperatorInfo,
    pub result: Option<Type>,
}

/// An argument supplied to a function call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallArg {
    pub ty: Option<Type>,
    pub name: Option<String>,
}
