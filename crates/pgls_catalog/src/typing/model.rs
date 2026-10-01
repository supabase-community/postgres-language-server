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

/// Candidate objects visible to a lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidates<T> {
    pub items: Vec<T>,
    pub complete: bool,
}

/// pg_type.typtype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    Base,
    Composite,
    Domain,
    Enum,
    Pseudo,
    Range,
    Multirange,
}

/// Operator arity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorKind {
    Prefix,
    Infix,
}

/// The mode of a function argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionArgumentMode {
    In,
    InOut,
    Variadic,
}

/// An input argument of a function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionArgument {
    pub name: Option<String>,
    pub ty: Option<TypeId>,
    pub mode: FunctionArgumentMode,
}

/// The typing-relevant function definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub arguments: Vec<FunctionArgument>,
    pub input_defaults: usize,
    pub variadic_element: Option<TypeId>,
    pub return_type: Option<TypeId>,
    pub returns_set: bool,
}

/// A database operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorInfo {
    pub oid: i64,
    pub schema: String,
    pub name: String,
    pub kind: OperatorKind,
    pub left: Option<TypeId>,
    pub right: Option<TypeId>,
    pub result: Option<TypeId>,
}

/// A cast context from pg_cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastContext {
    Implicit,
    Assignment,
    Explicit,
}

/// A cast implementation from pg_cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastMethod {
    Function,
    InputOutput,
    Binary,
}

/// A catalog cast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CastInfo {
    pub source: TypeId,
    pub target: TypeId,
    pub context: CastContext,
    pub method: CastMethod,
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
