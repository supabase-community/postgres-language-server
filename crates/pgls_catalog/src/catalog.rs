//! The database snapshot plus the in-memory changes of the current file.

use std::sync::Arc;

use pgls_schema_cache::SchemaCache;

use crate::view::{CatalogView, FunctionInfo, Lookup, RelationInfo, TypeInfo};

/// Indexed, immutable view of a [`SchemaCache`]. Build it once per schema cache and share it.
pub struct CatalogBase {
    #[allow(dead_code)]
    schema_cache: Arc<SchemaCache>,
}

impl CatalogBase {
    pub fn new(schema_cache: Arc<SchemaCache>) -> Self {
        Self { schema_cache }
    }

    pub fn schema_cache(&self) -> &SchemaCache {
        &self.schema_cache
    }
}

/// The catalog as seen by a statement: the database snapshot plus the changes made by the
/// statements before it.
#[derive(Clone)]
pub struct Catalog {
    #[allow(dead_code)]
    base: Option<Arc<CatalogBase>>,
}

impl Catalog {
    /// Without a base, every lookup that the file itself doesn't answer is [`Lookup::Unknown`].
    pub fn new(base: Option<Arc<CatalogBase>>) -> Self {
        Self { base }
    }

    /// Applies the effects of `stmt` on the catalog. `search_path` is the session's search path
    /// before the statement and decides where unqualified objects are created.
    pub fn apply(&mut self, stmt: &pgls_query::NodeEnum, search_path: &[String]) {
        let _ = (stmt, search_path);
    }
}

impl CatalogView for Catalog {
    fn relation(
        &self,
        _schema: Option<&str>,
        _name: &str,
        _search_path: &[String],
    ) -> Lookup<RelationInfo> {
        Lookup::Unknown
    }

    fn schema(&self, _name: &str) -> Lookup<()> {
        Lookup::Unknown
    }

    fn functions(
        &self,
        _schema: Option<&str>,
        _name: &str,
        _search_path: &[String],
    ) -> Lookup<Vec<FunctionInfo>> {
        Lookup::Unknown
    }

    fn type_(
        &self,
        _schema: Option<&str>,
        _name: &str,
        _search_path: &[String],
    ) -> Lookup<TypeInfo> {
        Lookup::Unknown
    }
}
