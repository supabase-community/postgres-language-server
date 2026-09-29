use std::sync::Arc;

use crate::{ProcKind, Snapshot, TableKind};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{Key, key};
use crate::view::{
    ColumnInfo, FunctionInfo, FunctionKind, Origin, RelationInfo, RelationKind, TypeInfo,
};

/// Indexed, immutable view of a schema cache. Build it once per schema cache and share it
/// between files.
pub struct CatalogBase {
    snapshot: Arc<Snapshot>,
    schemas: FxHashSet<String>,
    relations: FxHashMap<Key, RelationInfo>,
    /// Names that may be relations the schema cache doesn't list: foreign tables, relations the
    /// user has no privileges on, and composite types. Lookups of them are unknown, not missing.
    possible_relations: FxHashSet<Key>,
    /// Partitions and tables that inherit from another table.
    inheritance_children: FxHashSet<Key>,
    types: FxHashMap<Key, TypeInfo>,
    functions: FxHashMap<Key, Vec<FunctionInfo>>,
}

impl CatalogBase {
    pub fn new(snapshot: Arc<Snapshot>) -> Self {
        let schemas = snapshot
            .schemas
            .iter()
            .map(|schema| schema.name.clone())
            .collect();

        let types_by_id: FxHashMap<i64, &crate::PostgresType> =
            snapshot.types.iter().map(|t| (t.id, t)).collect();

        // Attributes of composite types and row types. Unlike the columns of the schema cache,
        // they are not filtered by column privileges.
        let attributes_of = |type_: &crate::PostgresType| -> Option<Vec<ColumnInfo>> {
            (!type_.attributes.attrs.is_empty()).then(|| {
                type_
                    .attributes
                    .attrs
                    .iter()
                    .map(|attribute| ColumnInfo {
                        name: attribute.name.clone(),
                        type_name: types_by_id.get(&attribute.type_id).map(|t| t.name.clone()),
                    })
                    .collect()
            })
        };

        let mut types = FxHashMap::default();
        let mut possible_relations = FxHashSet::default();
        for type_ in &snapshot.types {
            let attributes = attributes_of(type_);
            if attributes.is_some() {
                possible_relations.insert(key(&type_.schema, &type_.name));
            }
            types.insert(
                key(&type_.schema, &type_.name),
                TypeInfo {
                    schema: type_.schema.clone(),
                    name: type_.name.clone(),
                    attributes,
                    origin: Origin::Database,
                },
            );
        }

        let mut columns_by_table: FxHashMap<i64, Vec<ColumnInfo>> = FxHashMap::default();
        for column in &snapshot.columns {
            columns_by_table
                .entry(column.table_oid)
                .or_default()
                .push(ColumnInfo {
                    name: column.name.clone(),
                    type_name: column.type_name.clone(),
                });
        }

        let inheritance_children = snapshot
            .tables
            .iter()
            .filter(|table| table.is_inheritance_child)
            .map(|table| key(&table.schema, &table.name))
            .collect();

        let mut relations = FxHashMap::default();
        for table in &snapshot.tables {
            let key = key(&table.schema, &table.name);
            let columns = types
                .get(&key)
                .and_then(|row_type: &TypeInfo| row_type.attributes.clone())
                .or_else(|| {
                    columns_by_table
                        .get(&table.id)
                        .filter(|columns| !columns.is_empty())
                        .cloned()
                });
            relations.insert(
                key,
                RelationInfo {
                    schema: table.schema.clone(),
                    name: table.name.clone(),
                    kind: match table.table_kind {
                        TableKind::Ordinary => RelationKind::Table,
                        TableKind::Partitioned => RelationKind::PartitionedTable,
                        TableKind::View => RelationKind::View,
                        TableKind::MaterializedView => RelationKind::MaterializedView,
                    },
                    columns,
                    origin: Origin::Database,
                },
            );
        }
        // Every table and view has a row type.
        for relation in relations.values() {
            types
                .entry(key(&relation.schema, &relation.name))
                .or_insert_with(|| TypeInfo {
                    schema: relation.schema.clone(),
                    name: relation.name.clone(),
                    attributes: relation.columns.clone(),
                    origin: Origin::Database,
                });
        }
        for sequence in &snapshot.sequences {
            relations
                .entry(key(&sequence.schema, &sequence.name))
                .or_insert_with(|| RelationInfo {
                    schema: sequence.schema.clone(),
                    name: sequence.name.clone(),
                    kind: RelationKind::Other,
                    columns: Some(sequence_columns()),
                    origin: Origin::Database,
                });
        }

        let mut functions: FxHashMap<Key, Vec<FunctionInfo>> = FxHashMap::default();
        for function in &snapshot.functions {
            let args = &function.args.args;
            let inputs = args
                .iter()
                .filter(|arg| matches!(arg.mode.as_str(), "in" | "inout" | "variadic"))
                .count();
            // Defaults belong to the last input arguments. The schema cache doesn't attach them to
            // the right arguments when there are output arguments, but their number is correct.
            let defaults = args
                .iter()
                .filter(|arg| arg.has_default == Some(true))
                .count();
            let variadic = args.iter().any(|arg| arg.mode == "variadic");
            let outputs: Vec<ColumnInfo> = args
                .iter()
                .filter(|arg| matches!(arg.mode.as_str(), "out" | "inout" | "table"))
                .map(|arg| ColumnInfo {
                    name: arg.name.clone(),
                    type_name: types_by_id.get(&arg.type_id).map(|t| t.name.clone()),
                })
                .collect();
            let return_columns = if outputs.is_empty() {
                function
                    .return_type_id
                    .and_then(|id| types_by_id.get(&id))
                    .and_then(|type_| attributes_of(type_))
            } else {
                Some(outputs)
            };

            functions
                .entry(key(&function.schema, &function.name))
                .or_default()
                .push(FunctionInfo {
                    schema: function.schema.clone(),
                    name: function.name.clone(),
                    kind: match function.kind {
                        ProcKind::Function => FunctionKind::Function,
                        ProcKind::Aggregate => FunctionKind::Aggregate,
                        ProcKind::Window => FunctionKind::Window,
                        ProcKind::Procedure => FunctionKind::Procedure,
                    },
                    min_args: inputs.saturating_sub(defaults),
                    max_args: (!variadic).then_some(inputs),
                    returns_set: function.is_set_returning_function,
                    return_columns,
                    origin: Origin::Database,
                });
        }

        Self {
            snapshot,
            schemas,
            relations,
            possible_relations,
            inheritance_children,
            types,
            functions,
        }
    }

    /// A database with only the empty schemas `public` and `pg_catalog`. Used to check
    /// examples that create everything they use.
    pub fn empty() -> Self {
        let schemas = ["public", "pg_catalog"]
            .into_iter()
            .map(|name| crate::Schema {
                name: name.into(),
                ..Default::default()
            })
            .collect();
        Self::new(Arc::new(Snapshot {
            schemas,
            ..Default::default()
        }))
    }

    /// Whether this was built from exactly this schema cache.
    pub fn is_built_from(&self, snapshot: &Arc<Snapshot>) -> bool {
        Arc::ptr_eq(&self.snapshot, snapshot)
    }

    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }

    pub(super) fn has_schema(&self, name: &str) -> bool {
        self.schemas.contains(name)
    }

    pub(super) fn relation(&self, schema: &str, name: &str) -> Option<&RelationInfo> {
        self.relations.get(&key(schema, name))
    }

    pub(super) fn is_inheritance_child(&self, schema: &str, name: &str) -> bool {
        self.inheritance_children.contains(&key(schema, name))
    }

    pub(super) fn may_have_relation(&self, schema: &str, name: &str) -> bool {
        self.possible_relations.contains(&key(schema, name))
    }

    pub(super) fn type_(&self, schema: &str, name: &str) -> Option<&TypeInfo> {
        self.types.get(&key(schema, name))
    }

    pub(super) fn functions(&self, schema: &str, name: &str) -> Option<&[FunctionInfo]> {
        self.functions.get(&key(schema, name)).map(Vec::as_slice)
    }

    pub(super) fn has_installed_extension(&self, name: &str) -> bool {
        self.snapshot
            .extensions
            .iter()
            .any(|extension| extension.name == name && extension.installed_version.is_some())
    }
}

/// The columns of a sequence relation.
pub(super) fn sequence_columns() -> Vec<ColumnInfo> {
    [
        ("last_value", "int8"),
        ("log_cnt", "int8"),
        ("is_called", "bool"),
    ]
    .into_iter()
    .map(|(name, type_name)| ColumnInfo {
        name: name.into(),
        type_name: Some(type_name.into()),
    })
    .collect()
}
