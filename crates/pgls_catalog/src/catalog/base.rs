use std::sync::Arc;

use crate::typing::{Type, TypeId};
use crate::{CastInfo, FunctionArgument, FunctionArgumentMode, FunctionSignature, OperatorInfo};
use crate::{FunctionArg, Generated, Identity, ProcKind, Snapshot, TableKind};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{Key, key};
use crate::lookup::{
    ColumnInfo, FunctionInfo, FunctionKind, IndexInfo, Lookup, Origin, RelationInfo, RelationKind,
    TypeInfo,
};

/// Indexed, immutable view of a database snapshot. Build it once per snapshot and share it
/// between files.
pub struct CatalogBase {
    snapshot: Arc<Snapshot>,
    schemas: FxHashSet<String>,
    roles: FxHashSet<String>,
    relations: FxHashMap<Key, RelationInfo>,
    /// Names that may be relations the snapshot doesn't list: foreign tables, relations the
    /// user has no privileges on, and composite types. Lookups of them are unknown, not missing.
    possible_relations: FxHashSet<Key>,
    /// Partitions and tables that inherit from another table.
    inheritance_children: FxHashSet<Key>,
    indexes: FxHashMap<Key, IndexInfo>,
    /// The indexes of each table, by the key of the table.
    table_indexes: FxHashMap<Key, Vec<Key>>,
    types: FxHashMap<Key, TypeInfo>,
    functions: FxHashMap<Key, Vec<FunctionInfo>>,
    types_by_id: FxHashMap<i64, TypeInfo>,
    /// `None` for a cast whose context or method we don't know.
    casts: FxHashMap<(i64, i64), Option<CastInfo>>,
    operators: FxHashMap<Key, Vec<OperatorInfo>>,
    /// Operator names that some operator of a kind we don't know has, like a postfix operator.
    operators_of_unknown_kind: FxHashSet<Key>,
}

impl CatalogBase {
    pub fn new(snapshot: Arc<Snapshot>) -> Self {
        let schemas = snapshot
            .schemas
            .iter()
            .map(|schema| schema.name.clone())
            .collect();
        let roles = snapshot
            .roles
            .iter()
            .map(|role| role.name.clone())
            .collect();

        let types_by_id: FxHashMap<i64, &crate::PostgresType> =
            snapshot.types.iter().map(|t| (t.id, t)).collect();

        // Attributes of composite types and row types. Unlike the columns of the snapshot,
        // they are not filtered by column privileges.
        let attributes_of = |type_: &crate::PostgresType| -> Option<Vec<ColumnInfo>> {
            (!type_.attributes.attrs.is_empty()).then(|| {
                type_
                    .attributes
                    .attrs
                    .iter()
                    .map(|attribute| {
                        ColumnInfo::new(
                            attribute.name.clone(),
                            types_by_id.get(&attribute.type_id).map(|t| t.name.clone()),
                            types_by_id
                                .contains_key(&attribute.type_id)
                                .then_some(Type::Named(TypeId::Snapshot(attribute.type_id))),
                        )
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

                    id: Some(TypeId::Snapshot(type_.id)),
                    kind: type_.typtype.get(),
                    category: type_.typcategory.chars().next(),
                    preferred: Some(type_.typispreferred),
                    element: (type_.is_array && type_.typelem != 0)
                        .then_some(TypeId::Snapshot(type_.typelem)),
                    array: (type_.typarray != 0).then_some(TypeId::Snapshot(type_.typarray)),
                    base: (type_.typbasetype != 0).then_some(TypeId::Snapshot(type_.typbasetype)),
                    relation: (type_.typrelid != 0).then_some(type_.typrelid),
                },
            );
        }

        let mut columns_by_table: FxHashMap<i64, Vec<ColumnInfo>> = FxHashMap::default();
        for column in &snapshot.columns {
            columns_by_table
                .entry(column.table_oid)
                .or_default()
                .push(ColumnInfo {
                    generated: column.generated,
                    identity: column.identity,
                    ..ColumnInfo::new(
                        column.name.clone(),
                        column.type_name.clone(),
                        types_by_id
                            .contains_key(&column.type_id)
                            .then_some(Type::Named(TypeId::Snapshot(column.type_id))),
                    )
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
            let table_columns = columns_by_table.get(&table.id);
            let columns = types
                .get(&key)
                .and_then(|row_type: &TypeInfo| row_type.attributes.clone())
                .map(|mut columns| {
                    // The row type doesn't know which columns are generated or identity columns.
                    for column in &mut columns {
                        let table_column = table_columns.and_then(|table_columns| {
                            table_columns.iter().find(|c| c.name == column.name)
                        });
                        column.generated = table_column.map_or(Generated::No, |c| c.generated);
                        column.identity = table_column.map_or(Identity::No, |c| c.identity);
                    }
                    columns
                })
                .or_else(|| table_columns.filter(|columns| !columns.is_empty()).cloned());
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
                        TableKind::Unknown => RelationKind::Other,
                    },
                    columns,
                    origin: Origin::Database,
                },
            );
        }
        // Keep a name-only row type when a table's pg_type row is unavailable (for example,
        // because the snapshot was filtered), but never invent a type identity.
        for relation in relations.values() {
            types
                .entry(key(&relation.schema, &relation.name))
                .or_insert_with(|| TypeInfo {
                    schema: relation.schema.clone(),
                    name: relation.name.clone(),
                    attributes: relation.columns.clone(),
                    origin: Origin::Database,
                    id: None,
                    kind: None,
                    category: None,
                    preferred: None,
                    element: None,
                    array: None,
                    base: None,
                    relation: None,
                });
        }
        let sequence_columns = sequence_columns(|name| {
            types
                .get(&key("pg_catalog", name))
                .and_then(|type_: &TypeInfo| type_.id.clone())
        });
        for sequence in &snapshot.sequences {
            relations
                .entry(key(&sequence.schema, &sequence.name))
                .or_insert_with(|| RelationInfo {
                    schema: sequence.schema.clone(),
                    name: sequence.name.clone(),
                    kind: RelationKind::Other,
                    columns: Some(sequence_columns.clone()),
                    origin: Origin::Database,
                });
        }

        let mut indexes = FxHashMap::default();
        let mut table_indexes: FxHashMap<Key, Vec<Key>> = FxHashMap::default();
        for index in &snapshot.indexes {
            let index_key = key(&index.schema, &index.name);
            table_indexes
                .entry(key(&index.schema, &index.table_name))
                .or_default()
                .push(index_key.clone());
            indexes.insert(
                index_key,
                IndexInfo {
                    schema: index.schema.clone(),
                    name: index.name.clone(),
                    table: index.table_name.clone(),
                    columns: index.columns.clone(),
                    origin: Origin::Database,
                },
            );
        }

        let mut functions: FxHashMap<Key, Vec<FunctionInfo>> = FxHashMap::default();
        for function in &snapshot.functions {
            let args = &function.args.args;
            let kind = match function.kind {
                ProcKind::Function => Some(FunctionKind::Function),
                ProcKind::Aggregate => Some(FunctionKind::Aggregate),
                ProcKind::Window => Some(FunctionKind::Window),
                ProcKind::Procedure => Some(FunctionKind::Procedure),
                ProcKind::Unknown => None,
            };
            let is_procedure = kind == Some(FunctionKind::Procedure);
            // With a kind or an argument mode we don't know, the arguments that a call passes
            // are unknown.
            let known = kind.is_some() && args.iter().all(|arg| arg.mode().is_some());
            let inputs = args
                .iter()
                .filter(|arg| {
                    arg.mode()
                        .is_some_and(|mode| mode.is_call_argument(is_procedure))
                })
                .count();
            // Defaults belong to the last input arguments. The snapshot doesn't attach them to
            // the right arguments when there are output arguments, but their number is correct.
            let defaults = usize::try_from(function.input_defaults)
                .unwrap_or_default()
                .max(
                    args.iter()
                        .filter(|arg| arg.has_default == Some(true))
                        .count(),
                )
                .min(inputs);
            let variadic = args
                .iter()
                .any(|arg| arg.mode() == Some(FunctionArgumentMode::Variadic));
            let outputs: Vec<ColumnInfo> = args
                .iter()
                .filter(|arg| arg.mode().is_some_and(FunctionArgumentMode::is_output))
                .map(|arg| {
                    ColumnInfo::new(
                        arg.name.clone(),
                        types_by_id.get(&arg.type_id).map(|t| t.name.clone()),
                        types_by_id
                            .contains_key(&arg.type_id)
                            .then_some(Type::Named(TypeId::Snapshot(arg.type_id))),
                    )
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
            let argument = |arg: &FunctionArg, mode| FunctionArgument {
                name: (!arg.name.is_empty()).then(|| arg.name.clone()),
                ty: types_by_id
                    .contains_key(&arg.type_id)
                    .then_some(TypeId::Snapshot(arg.type_id)),
                mode,
            };

            functions
                .entry(key(&function.schema, &function.name))
                .or_default()
                .push(FunctionInfo {
                    schema: function.schema.clone(),
                    name: function.name.clone(),
                    // Only a procedure is ever treated differently, and the signature of a
                    // function of unknown kind is unknown.
                    kind: kind.unwrap_or(FunctionKind::Function),
                    min_args: if known {
                        inputs.saturating_sub(defaults)
                    } else {
                        0
                    },
                    max_args: (known && !variadic).then_some(inputs),
                    returns_set: function.is_set_returning_function,
                    return_columns,
                    volatility: Some(function.behavior),
                    origin: Origin::Database,
                    id: Some(function.id),
                    signature: known.then(|| FunctionSignature {
                        arguments: args
                            .iter()
                            .filter_map(|arg| {
                                let mode = arg.mode()?;
                                mode.is_call_argument(is_procedure)
                                    .then(|| argument(arg, mode))
                            })
                            .collect(),
                        output_arguments: if is_procedure {
                            Vec::new()
                        } else {
                            args.iter()
                                .enumerate()
                                .filter(|(_, arg)| arg.mode() == Some(FunctionArgumentMode::Out))
                                .map(|(position, arg)| {
                                    (position, argument(arg, FunctionArgumentMode::Out))
                                })
                                .collect()
                        },
                        input_defaults: defaults,
                        variadic_element: (function.variadic_type_id != 0
                            && types_by_id.contains_key(&function.variadic_type_id))
                        .then_some(TypeId::Snapshot(function.variadic_type_id)),
                        return_type: function
                            .return_type_id
                            .filter(|id| types_by_id.contains_key(id))
                            .map(TypeId::Snapshot),
                        returns_set: function.is_set_returning_function,
                    }),
                });
        }

        let types_by_id = types
            .values()
            .filter_map(|info| match info.id.as_ref() {
                Some(TypeId::Snapshot(id)) => Some((*id, info.clone())),
                _ => None,
            })
            .collect();
        // A cast with a context or method we don't know is in the map without its facts, so
        // looking it up is unknown rather than missing.
        let casts = snapshot
            .casts
            .iter()
            .map(|cast| {
                let info = cast
                    .context
                    .get()
                    .zip(cast.method.get())
                    .map(|(context, method)| CastInfo {
                        source: TypeId::Snapshot(cast.source),
                        target: TypeId::Snapshot(cast.target),
                        context,
                        method,
                    });
                ((cast.source, cast.target), info)
            })
            .collect();
        let mut operators = FxHashMap::<Key, Vec<OperatorInfo>>::default();
        let mut operators_of_unknown_kind = FxHashSet::default();
        for op in &snapshot.operators {
            let key = key(&op.schema, &op.name);
            let Some(kind) = op.kind.get() else {
                operators_of_unknown_kind.insert(key);
                continue;
            };
            operators.entry(key).or_default().push(OperatorInfo {
                oid: op.oid,
                schema: op.schema.clone(),
                name: op.name.clone(),
                kind,
                left: (op.left != 0).then_some(TypeId::Snapshot(op.left)),
                right: (op.right != 0).then_some(TypeId::Snapshot(op.right)),
                result: (op.result != 0).then_some(TypeId::Snapshot(op.result)),
            });
        }
        Self {
            snapshot,
            schemas,
            roles,
            relations,
            possible_relations,
            inheritance_children,
            indexes,
            table_indexes,
            types,
            functions,
            types_by_id,
            casts,
            operators,
            operators_of_unknown_kind,
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

    /// Whether this was built from exactly this snapshot.
    pub fn is_built_from(&self, snapshot: &Arc<Snapshot>) -> bool {
        Arc::ptr_eq(&self.snapshot, snapshot)
    }

    pub fn snapshot(&self) -> &Arc<Snapshot> {
        &self.snapshot
    }

    pub(super) fn has_role(&self, name: &str) -> bool {
        self.roles.contains(name)
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

    /// The partitions and inheritance children of the snapshot.
    pub(super) fn inheritance_children(&self) -> impl Iterator<Item = &Key> {
        self.inheritance_children.iter()
    }

    pub(super) fn may_have_relation(&self, schema: &str, name: &str) -> bool {
        self.possible_relations.contains(&key(schema, name))
    }

    pub(super) fn index(&self, schema: &str, name: &str) -> Option<&IndexInfo> {
        self.indexes.get(&key(schema, name))
    }

    /// The indexes on a table.
    pub(super) fn table_indexes(
        &self,
        schema: &str,
        table: &str,
    ) -> impl Iterator<Item = &IndexInfo> {
        self.table_indexes
            .get(&key(schema, table))
            .into_iter()
            .flatten()
            .filter_map(|index| self.indexes.get(index))
    }

    /// Whether the snapshot may lack this index. The snapshot leaves out the indexes of the
    /// system schemas, whose names in `pg_catalog` and `pg_toast` all start with `pg_`.
    pub(super) fn may_have_index(&self, schema: &str, name: &str) -> bool {
        match schema {
            "pg_catalog" | "pg_toast" => name.starts_with("pg_"),
            "information_schema" => true,
            _ => false,
        }
    }

    pub(super) fn type_(&self, schema: &str, name: &str) -> Option<&TypeInfo> {
        self.types.get(&key(schema, name))
    }

    pub(super) fn functions(&self, schema: &str, name: &str) -> Option<&[FunctionInfo]> {
        self.functions.get(&key(schema, name)).map(Vec::as_slice)
    }

    pub(super) fn type_by_id(&self, id: &TypeId) -> Option<&TypeInfo> {
        match id {
            TypeId::Snapshot(id) => self.types_by_id.get(id),
            TypeId::File(_) => None,
        }
    }

    /// The direct cast between two database types. Missing only when the snapshot has the
    /// casts and this one isn't among them.
    pub(super) fn cast(&self, source: &TypeId, target: &TypeId) -> Lookup<CastInfo> {
        let (TypeId::Snapshot(source), TypeId::Snapshot(target)) = (source, target) else {
            return self.not_in_snapshot();
        };
        match self.casts.get(&(*source, *target)) {
            Some(Some(cast)) => Lookup::Found(cast.clone()),
            Some(None) => Lookup::Unknown,
            None => self.not_in_snapshot(),
        }
    }

    /// A typing fact that the snapshot doesn't have: missing if it has the typing metadata.
    fn not_in_snapshot<T>(&self) -> Lookup<T> {
        if self.snapshot.typing_metadata {
            Lookup::Missing
        } else {
            Lookup::Unknown
        }
    }

    pub(super) fn operators(&self, schema: &str, name: &str) -> Option<&[OperatorInfo]> {
        self.operators.get(&key(schema, name)).map(Vec::as_slice)
    }

    /// Whether an operator with this name has a kind we don't know, like a postfix operator.
    pub(super) fn has_operator_of_unknown_kind(&self, schema: &str, name: &str) -> bool {
        self.operators_of_unknown_kind.contains(&key(schema, name))
    }

    pub(super) fn has_installed_extension(&self, name: &str) -> bool {
        self.snapshot
            .extensions
            .iter()
            .any(|extension| extension.name == name && extension.installed_version.is_some())
    }
}

/// The columns of a sequence relation. `builtin_type` looks up the identity of a `pg_catalog`
/// type by name.
pub(super) fn sequence_columns(builtin_type: impl Fn(&str) -> Option<TypeId>) -> Vec<ColumnInfo> {
    [
        ("last_value", "int8"),
        ("log_cnt", "int8"),
        ("is_called", "bool"),
    ]
    .into_iter()
    .map(|(name, type_name)| {
        ColumnInfo::new(
            name,
            Some(type_name.into()),
            builtin_type(type_name).map(Type::Named),
        )
    })
    .collect()
}
