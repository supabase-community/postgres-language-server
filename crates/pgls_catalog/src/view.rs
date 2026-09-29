//! The read-only view of the catalog that name resolution works against.
//!
//! [`crate::Catalog`] implements [`CatalogView`] for the real database snapshot plus the
//! changes of the current file. The resolver only depends on this trait, so it can be tested
//! against small in-memory catalogs.

/// The result of a catalog lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lookup<T> {
    /// The object exists.
    Found(T),
    /// The object certainly does not exist.
    Missing,
    /// We can't tell, e.g. because the file ran statements the catalog can't model (`DO` blocks,
    /// `CREATE EXTENSION`, ...), or because there is no database snapshot. Never report these.
    Unknown,
}

impl<T> Lookup<T> {
    pub fn found(self) -> Option<T> {
        match self {
            Lookup::Found(value) => Some(value),
            _ => None,
        }
    }

    pub fn is_missing(&self) -> bool {
        matches!(self, Lookup::Missing)
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Lookup<U> {
        match self {
            Lookup::Found(value) => Lookup::Found(f(value)),
            Lookup::Missing => Lookup::Missing,
            Lookup::Unknown => Lookup::Unknown,
        }
    }
}

/// Where the definition of an object comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// The object comes from the database snapshot and was not changed by the current file.
    Database,
    /// The object was created or changed by a statement earlier in the current file.
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationKind {
    Table,
    PartitionedTable,
    View,
    MaterializedView,
    ForeignTable,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnInfo {
    pub name: String,
    /// The type name, if known (e.g. `int4`, `text`, `public.my_enum`).
    pub type_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationInfo {
    pub schema: String,
    pub name: String,
    pub kind: RelationKind,
    /// `None` when the column set is not known, e.g. a view whose select list could not be derived.
    pub columns: Option<Vec<ColumnInfo>>,
    pub origin: Origin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionKind {
    Function,
    Aggregate,
    Window,
    Procedure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionInfo {
    pub schema: String,
    pub name: String,
    pub kind: FunctionKind,
    /// Number of input arguments without a default.
    pub min_args: usize,
    /// Maximum number of input arguments. `None` for variadic functions.
    pub max_args: Option<usize>,
    pub returns_set: bool,
    /// Output columns when the function returns a relation or composite type
    /// (what `select * from fn()` yields). `None` when unknown or scalar.
    pub return_columns: Option<Vec<ColumnInfo>>,
    pub origin: Origin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInfo {
    pub schema: String,
    pub name: String,
    /// Attributes for composite types (including table row types), `None` otherwise or when unknown.
    pub attributes: Option<Vec<ColumnInfo>>,
    pub origin: Origin,
}

/// Read-only access to the catalog as it is before the statement being analysed.
///
/// All `search_path` arguments are the explicit search path of the session. Implementations
/// apply Postgres' implicit rules on top of it: `pg_catalog` is searched first unless it is
/// listed explicitly, and `pg_temp` is searched first for relations and types (never for
/// functions) unless it is listed explicitly.
pub trait CatalogView {
    /// Looks up a relation. With `schema: None`, the search path is used.
    fn relation(
        &self,
        schema: Option<&str>,
        name: &str,
        search_path: &[String],
    ) -> Lookup<RelationInfo>;

    /// Looks up a schema. `pg_temp` and `pg_catalog` always exist.
    fn schema(&self, name: &str) -> Lookup<()>;

    /// Returns all overloads with this name that are visible. With `schema: None`, all schemas
    /// on the search path are considered.
    fn functions(
        &self,
        schema: Option<&str>,
        name: &str,
        search_path: &[String],
    ) -> Lookup<Vec<FunctionInfo>>;

    /// Looks up a type by its internal name (e.g. `int4`, not `integer`). With `schema: None`,
    /// the search path is used.
    fn type_(&self, schema: Option<&str>, name: &str, search_path: &[String]) -> Lookup<TypeInfo>;
}
