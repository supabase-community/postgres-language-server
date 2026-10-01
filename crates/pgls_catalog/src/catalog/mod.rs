//! The database snapshot plus the changes made by the statements of the current file.
//!
//! [`CatalogBase`] indexes the database snapshot once. [`Catalog`] layers the effects of the
//! file's DDL on top of it without touching the database, and [`Catalog::snapshot`] turns the
//! result back into a snapshot. Everything the catalog can't be sure about
//! is reported as [`Lookup::Unknown`], so rules built on it never report false positives.

mod base;
mod ddl;
mod derive;
mod materialize;
mod names;
mod overlay;
#[cfg(test)]
mod tests;

use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::lookup::{CatalogView, FunctionInfo, Lookup, RelationInfo, TypeInfo};

pub use base::CatalogBase;

/// `(schema, name)` of a catalog object.
type Key = (String, String);

/// The state of an object that the file changed.
#[derive(Debug, Clone)]
enum Entry<T> {
    /// Created or altered by the file.
    Defined(T),
    /// Dropped (or renamed away) by the file.
    Dropped,
    /// Changed by the file in a way the catalog can't follow.
    Unknown,
}

/// The catalog as seen by a statement: the database snapshot plus the changes made by the
/// statements before it.
#[derive(Clone)]
pub struct Catalog {
    base: Option<Arc<CatalogBase>>,
    /// Schemas created (`true`) or dropped (`false`) by the file.
    schemas: FxHashMap<String, bool>,
    /// Schemas dropped at some point of the file. The database objects in them are gone, even if
    /// the schema was created again afterwards.
    dropped_schemas: FxHashSet<String>,
    relations: FxHashMap<Key, Entry<RelationInfo>>,
    types: FxHashMap<Key, Entry<TypeInfo>>,
    /// All overloads with a given name.
    functions: FxHashMap<Key, Entry<Vec<FunctionInfo>>>,
    /// Inheritance children and partitions created by the file, by parent.
    children: FxHashMap<Key, Vec<Key>>,
    /// Set once the file changed the columns of a database table. Its partitions and
    /// inheritance children in the database changed too, but we don't know which they are.
    database_columns_changed: bool,
    /// Set once the file ran a statement with effects the catalog can't model (`DO`, `CALL`,
    /// `CREATE EXTENSION` of a new extension, ...). From then on, nothing is known to be missing.
    tainted: bool,
    /// The catalog at the start of each open transaction and savepoint, restored on
    /// `ROLLBACK`.
    savepoints: Vec<Savepoint>,
}

/// The catalog at `BEGIN` (`name: None`) or `SAVEPOINT`.
#[derive(Clone)]
struct Savepoint {
    name: Option<String>,
    catalog: Box<Catalog>,
}

impl Catalog {
    pub fn new(base: Option<Arc<CatalogBase>>) -> Self {
        Self {
            base,
            schemas: FxHashMap::default(),
            dropped_schemas: FxHashSet::default(),
            relations: FxHashMap::default(),
            types: FxHashMap::default(),
            functions: FxHashMap::default(),
            children: FxHashMap::default(),
            database_columns_changed: false,
            tainted: false,
            savepoints: Vec::new(),
        }
    }

    /// Whether the catalog starts from a database snapshot. Without one, every lookup is
    /// [`Lookup::Unknown`].
    pub fn has_base(&self) -> bool {
        self.base.is_some()
    }

    /// Whether the file ran a statement the catalog can't model.
    pub fn is_tainted(&self) -> bool {
        self.tainted
    }

    /// The result of a lookup that found nothing.
    fn not_found<T>(&self) -> Lookup<T> {
        if self.base.is_none() || self.tainted {
            Lookup::Unknown
        } else {
            Lookup::Missing
        }
    }

    /// The database snapshot, unless the file dropped the schema.
    fn base_for(&self, schema: &str) -> Option<&CatalogBase> {
        self.base
            .as_deref()
            .filter(|_| !self.dropped_schemas.contains(schema))
    }

    fn entry_lookup<T: Clone>(&self, entry: &Entry<T>) -> Lookup<T> {
        match entry {
            Entry::Defined(value) => Lookup::Found(value.clone()),
            Entry::Dropped => self.not_found(),
            Entry::Unknown => Lookup::Unknown,
        }
    }

    fn relation_in(&self, schema: &str, name: &str) -> Lookup<RelationInfo> {
        if let Some(entry) = self.relations.get(&key(schema, name)) {
            return self.entry_lookup(entry);
        }
        match self.base_for(schema) {
            Some(base) => {
                if let Some(relation) = base.relation(schema, name) {
                    let mut relation = relation.clone();
                    if self.database_columns_changed && base.is_inheritance_child(schema, name) {
                        relation.columns = None;
                    }
                    Lookup::Found(relation)
                } else if base.may_have_relation(schema, name) {
                    Lookup::Unknown
                } else {
                    self.not_found()
                }
            }
            None => self.not_found(),
        }
    }

    fn type_in(&self, schema: &str, name: &str) -> Lookup<TypeInfo> {
        if let Some(entry) = self.types.get(&key(schema, name)) {
            return self.entry_lookup(entry);
        }
        match self
            .base_for(schema)
            .and_then(|base| base.type_(schema, name))
        {
            Some(type_info) => Lookup::Found(type_info.clone()),
            None => self.not_found(),
        }
    }

    fn functions_in(&self, schema: &str, name: &str) -> Lookup<Vec<FunctionInfo>> {
        if let Some(entry) = self.functions.get(&key(schema, name)) {
            return match self.entry_lookup(entry) {
                Lookup::Found(overloads) if overloads.is_empty() => self.not_found(),
                lookup => lookup,
            };
        }
        match self
            .base_for(schema)
            .and_then(|base| base.functions(schema, name))
        {
            Some(overloads) => Lookup::Found(overloads.to_vec()),
            None => self.not_found(),
        }
    }

    /// Looks `name` up in the schemas of the search path, in order.
    fn search<T>(
        &self,
        schemas: impl IntoIterator<Item = String>,
        lookup: impl Fn(&str) -> Lookup<T>,
    ) -> Lookup<T> {
        let mut unknown = false;
        for schema in schemas {
            match lookup(&schema) {
                Lookup::Found(value) => return Lookup::Found(value),
                Lookup::Unknown => unknown = true,
                Lookup::Missing => {}
            }
        }
        if unknown {
            Lookup::Unknown
        } else {
            self.not_found()
        }
    }

    /// Finds the schema of a relation, if the relation is known to exist.
    fn relation_key(
        &self,
        schema: Option<&str>,
        name: &str,
        search_path: &[String],
    ) -> Option<Key> {
        self.relation(schema, name, search_path)
            .found()
            .map(|relation| (relation.schema, relation.name))
    }

    fn type_key(&self, schema: Option<&str>, name: &str, search_path: &[String]) -> Option<Key> {
        self.type_(schema, name, search_path)
            .found()
            .map(|type_info| (type_info.schema, type_info.name))
    }

    /// Finds the schema that holds the functions with this name, if any.
    fn function_key(
        &self,
        schema: Option<&str>,
        name: &str,
        search_path: &[String],
    ) -> Option<Key> {
        match schema {
            Some(schema) => Some(key(schema, name)),
            None => function_search_path(search_path)
                .find(|schema| matches!(self.functions_in(schema, name), Lookup::Found(_)))
                .map(|schema| (schema, name.to_owned())),
        }
    }
}

fn key(schema: &str, name: &str) -> Key {
    (schema.to_owned(), name.to_owned())
}

/// Schemas searched for relations and types: `pg_temp` and `pg_catalog` come first unless the
/// search path lists them explicitly.
fn relation_search_path(search_path: &[String]) -> impl Iterator<Item = String> + '_ {
    let implicit = ["pg_temp", "pg_catalog"]
        .into_iter()
        .filter(|schema| !search_path.iter().any(|s| s == schema))
        .map(str::to_owned);
    implicit.chain(explicit_schemas(search_path))
}

/// Schemas searched for functions: `pg_catalog` comes first unless listed explicitly, and
/// `pg_temp` is never searched.
fn function_search_path(search_path: &[String]) -> impl Iterator<Item = String> + '_ {
    let implicit = (!search_path.iter().any(|s| s == "pg_catalog")).then(|| "pg_catalog".into());
    implicit
        .into_iter()
        .chain(explicit_schemas(search_path).filter(|schema| schema != "pg_temp"))
}

/// The explicit search path entries. `$user` is skipped: we don't know the role the file runs
/// as, and a schema named after it rarely exists.
fn explicit_schemas(search_path: &[String]) -> impl Iterator<Item = String> + '_ {
    search_path
        .iter()
        .filter(|schema| schema.as_str() != "$user")
        .cloned()
}

impl CatalogView for Catalog {
    fn relation(
        &self,
        schema: Option<&str>,
        name: &str,
        search_path: &[String],
    ) -> Lookup<RelationInfo> {
        match schema {
            Some(schema) => self.relation_in(schema, name),
            None => self.search(relation_search_path(search_path), |schema| {
                self.relation_in(schema, name)
            }),
        }
    }

    fn schema(&self, name: &str) -> Lookup<()> {
        if name == "pg_catalog" || name == "pg_temp" {
            return Lookup::Found(());
        }
        match self.schemas.get(name) {
            Some(true) => Lookup::Found(()),
            Some(false) => self.not_found(),
            None if self
                .base_for(name)
                .is_some_and(|base| base.has_schema(name)) =>
            {
                Lookup::Found(())
            }
            None => self.not_found(),
        }
    }

    fn functions(
        &self,
        schema: Option<&str>,
        name: &str,
        search_path: &[String],
    ) -> Lookup<Vec<FunctionInfo>> {
        if let Some(schema) = schema {
            return self.functions_in(schema, name);
        }

        // Overload resolution considers every schema of the search path, so collect them all.
        let mut overloads = Vec::new();
        let mut unknown = false;
        for schema in function_search_path(search_path) {
            match self.functions_in(&schema, name) {
                Lookup::Found(found) => overloads.extend(found),
                Lookup::Unknown => unknown = true,
                Lookup::Missing => {}
            }
        }

        if unknown {
            Lookup::Unknown
        } else if overloads.is_empty() {
            self.not_found()
        } else {
            Lookup::Found(overloads)
        }
    }

    fn type_(&self, schema: Option<&str>, name: &str, search_path: &[String]) -> Lookup<TypeInfo> {
        match schema {
            Some(schema) => self.type_in(schema, name),
            None => self.search(relation_search_path(search_path), |schema| {
                self.type_in(schema, name)
            }),
        }
    }
}
