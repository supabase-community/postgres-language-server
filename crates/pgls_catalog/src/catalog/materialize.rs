//! The catalog as a [`Snapshot`], for features that list database objects, like completions
//! and hover.

use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use super::{Catalog, Entry, Key};
use crate::lookup::{FunctionInfo, FunctionKind, Origin, RelationInfo, RelationKind, TypeInfo};
use crate::snapshot::{
    Column, ColumnClassKind, Function, PostgresType, PostgresTypeAttribute, ProcKind, Schema,
    Sequence, Snapshot, Table, TableKind, TypeAttributes,
};
use crate::typing::{Type, TypeId};
use crate::{FunctionArgumentMode, TypeKind};

impl Catalog {
    /// The database snapshot with the changes of the file applied.
    ///
    /// Objects created by the file only carry what the file says about them: names, kinds, and
    /// column types. Objects the catalog can't follow keep their database definition. `None`
    /// without a database snapshot.
    pub fn snapshot(&self) -> Option<Arc<Snapshot>> {
        let base = self.base.as_ref()?;
        if self.schemas.is_empty()
            && self.relations.is_empty()
            && self.types.is_empty()
            && self.functions.is_empty()
        {
            return Some(Arc::clone(base.snapshot()));
        }

        let mut builder = Builder {
            snapshot: Snapshot::clone(base.snapshot()),
            next_id: -1,
            type_ids: FxHashMap::default(),
            tables: FxHashMap::default(),
            types: FxHashMap::default(),
            type_names: FxHashMap::default(),
            sequences: FxHashSet::default(),
        };

        for schema in &self.dropped_schemas {
            builder.drop_schema(schema);
        }
        for (name, exists) in &self.schemas {
            if *exists {
                builder.create_schema(name);
            }
        }
        builder.index();
        for entry in self.types.values() {
            if let Entry::Defined(type_) = entry
                && type_.origin == Origin::File
            {
                builder.allocate_type(type_);
            }
        }
        // The relations have different names, so dropping them all after defining the others
        // removes the same objects as dropping them in between.
        let mut defined = Vec::new();
        let mut dropped = FxHashSet::default();
        for ((schema, name), entry) in &self.relations {
            match entry {
                Entry::Defined(relation) => {
                    if let Some((table_id, class_kind)) = builder.define_relation(relation) {
                        defined.push((table_id, class_kind, relation.as_ref()));
                    }
                }
                Entry::Dropped => {
                    dropped.insert((schema.as_str(), name.as_str()));
                }
                Entry::Unknown => {}
            }
        }
        builder.drop_relations(&dropped);
        builder.define_columns(&defined);

        let dropped: FxHashSet<(&str, &str)> = self
            .types
            .iter()
            .filter(|(_, entry)| matches!(entry, Entry::Dropped))
            .map(|((schema, name), _)| (schema.as_str(), name.as_str()))
            .collect();
        if !dropped.is_empty() {
            builder
                .snapshot
                .types
                .retain(|type_| !dropped.contains(&(type_.schema.as_str(), type_.name.as_str())));
        }
        builder.index();
        for (key, entry) in &self.types {
            if let Entry::Defined(type_) = entry
                && type_.origin == Origin::File
            {
                builder.define_type(key, type_);
            }
        }
        // The same holds for the functions.
        let mut dropped = FxHashSet::default();
        for (key, entry) in &self.functions {
            match entry {
                Entry::Defined(overloads) => {
                    for function in overloads.iter() {
                        if function.origin == Origin::File {
                            builder.add_function(key, function);
                        }
                    }
                }
                Entry::Dropped => {
                    dropped.insert((key.0.as_str(), key.1.as_str()));
                }
                Entry::Unknown => {}
            }
        }
        if !dropped.is_empty() {
            builder.snapshot.functions.retain(|function| {
                !dropped.contains(&(function.schema.as_str(), function.name.as_str()))
            });
        }

        Some(Arc::new(builder.snapshot))
    }
}

fn type_kind_code(kind: TypeKind) -> char {
    match kind {
        TypeKind::Base => 'b',
        TypeKind::Composite => 'c',
        TypeKind::Domain => 'd',
        TypeKind::Enum => 'e',
        TypeKind::Pseudo => 'p',
        TypeKind::Range => 'r',
        TypeKind::Multirange => 'm',
    }
}

struct Builder {
    snapshot: Snapshot,
    /// Ids of objects created by the file are negative, so they never collide with oids.
    next_id: i64,
    type_ids: FxHashMap<TypeId, i64>,
    /// The ids of the tables and types of `snapshot` by schema and name, and of the types by
    /// name, see [`Self::index`]. Each keeps the first object of a name in `snapshot`.
    tables: FxHashMap<Key, i64>,
    types: FxHashMap<Key, i64>,
    type_names: FxHashMap<String, i64>,
    sequences: FxHashSet<Key>,
}

impl Builder {
    /// Indexes the tables, types and sequences of the snapshot, so that finding one by name
    /// doesn't search all of them.
    fn index(&mut self) {
        let snapshot = &self.snapshot;
        self.tables.clear();
        for table in &snapshot.tables {
            self.tables
                .entry((table.schema.clone(), table.name.clone()))
                .or_insert(table.id);
        }
        self.types.clear();
        self.type_names.clear();
        for type_ in &snapshot.types {
            index_type(&mut self.types, &mut self.type_names, type_);
        }
        self.sequences = snapshot
            .sequences
            .iter()
            .map(|sequence| (sequence.schema.clone(), sequence.name.clone()))
            .collect();
    }

    fn push_type(&mut self, type_: PostgresType) {
        index_type(&mut self.types, &mut self.type_names, &type_);
        self.snapshot.types.push(type_);
    }

    fn next_id(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id -= 1;
        id
    }

    fn drop_schema(&mut self, name: &str) {
        let snapshot = &mut self.snapshot;
        snapshot.schemas.retain(|schema| schema.name != name);
        snapshot.tables.retain(|table| table.schema != name);
        snapshot.columns.retain(|column| column.schema_name != name);
        snapshot
            .functions
            .retain(|function| function.schema != name);
        snapshot.types.retain(|type_| type_.schema != name);
        snapshot
            .sequences
            .retain(|sequence| sequence.schema != name);
        snapshot.indexes.retain(|index| index.schema != name);
        snapshot
            .policies
            .retain(|policy| policy.schema_name != name);
        snapshot
            .triggers
            .retain(|trigger| trigger.table_schema != name);
    }

    fn create_schema(&mut self, name: &str) {
        if self
            .snapshot
            .schemas
            .iter()
            .any(|schema| schema.name == name)
        {
            return;
        }
        let id = self.next_id();
        self.snapshot.schemas.push(Schema {
            id,
            name: name.to_owned(),
            ..Default::default()
        });
    }

    fn drop_relations(&mut self, dropped: &FxHashSet<(&str, &str)>) {
        if dropped.is_empty() {
            return;
        }
        let snapshot = &mut self.snapshot;
        let is_dropped = |schema: &str, name: &str| dropped.contains(&(schema, name));
        snapshot
            .tables
            .retain(|table| !is_dropped(&table.schema, &table.name));
        snapshot
            .columns
            .retain(|column| !is_dropped(&column.schema_name, &column.table_name));
        snapshot
            .sequences
            .retain(|sequence| !is_dropped(&sequence.schema, &sequence.name));
        snapshot
            .indexes
            .retain(|index| !is_dropped(&index.schema, &index.table_name));
        snapshot
            .policies
            .retain(|policy| !is_dropped(&policy.schema_name, &policy.table_name));
        snapshot
            .triggers
            .retain(|trigger| !is_dropped(&trigger.table_schema, &trigger.table_name));
    }

    /// Adds the table of a relation, or finds the one of the database. Returns its id and the
    /// class of its columns when the columns are known, for [`Self::define_columns`].
    fn define_relation(&mut self, relation: &RelationInfo) -> Option<(i64, ColumnClassKind)> {
        let (table_kind, class_kind) = match relation.kind {
            RelationKind::Table => (TableKind::Ordinary, ColumnClassKind::OrdinaryTable),
            RelationKind::PartitionedTable => {
                (TableKind::Partitioned, ColumnClassKind::PartitionedTable)
            }
            RelationKind::View => (TableKind::View, ColumnClassKind::View),
            RelationKind::MaterializedView => (
                TableKind::MaterializedView,
                ColumnClassKind::MaterializedView,
            ),
            RelationKind::ForeignTable => (TableKind::Ordinary, ColumnClassKind::ForeignTable),
            RelationKind::Other => {
                self.define_sequence(relation);
                return None;
            }
        };

        let key = (relation.schema.clone(), relation.name.clone());
        let table_id = match self.tables.get(&key) {
            Some(id) => *id,
            None => {
                let id = self.next_id();
                self.snapshot.tables.push(Table {
                    id,
                    schema: relation.schema.clone(),
                    name: relation.name.clone(),
                    table_kind,
                    ..Default::default()
                });
                self.tables.insert(key, id);
                id
            }
        };

        // Without known columns, a database table keeps its columns.
        relation.columns.is_some().then_some((table_id, class_kind))
    }

    /// Replaces the columns of the tables of `defined`, in its order, keeping the database
    /// definition of the columns that keep their type.
    fn define_columns(&mut self, defined: &[(i64, ColumnClassKind, &RelationInfo)]) {
        let table_ids: FxHashSet<i64> = defined.iter().map(|(id, _, _)| *id).collect();
        let mut previous: FxHashMap<i64, FxHashMap<String, Column>> = FxHashMap::default();
        self.snapshot.columns.retain(|column| {
            if table_ids.contains(&column.table_oid) {
                previous
                    .entry(column.table_oid)
                    .or_default()
                    .insert(column.name.clone(), column.clone());
                false
            } else {
                true
            }
        });

        for (table_id, class_kind, relation) in defined {
            let table_id = *table_id;
            let mut previous = previous.remove(&table_id).unwrap_or_default();
            for (number, column) in relation.columns.iter().flatten().enumerate() {
                let column = match previous.remove(&column.name) {
                    Some(mut previous) if previous.type_name == column.type_name => {
                        previous.number = number as i64 + 1;
                        previous
                    }
                    _ => Column {
                        name: column.name.clone(),
                        table_name: relation.name.clone(),
                        table_oid: table_id,
                        class_kind: class_kind.clone(),
                        number: number as i64 + 1,
                        schema_name: relation.schema.clone(),
                        type_id: column
                            .ty
                            .as_ref()
                            .and_then(|ty| match ty {
                                Type::Named(id) => self.id_for(id),
                                _ => None,
                            })
                            .unwrap_or_else(|| self.type_id(column.type_name.as_deref())),

                        type_name: column.type_name.clone(),
                        is_nullable: true,
                        is_primary_key: false,
                        is_unique: false,
                        default_expr: None,
                        varchar_length: None,
                        comment: None,
                    },
                };
                self.snapshot.columns.push(column);
            }
        }
    }

    fn define_sequence(&mut self, relation: &RelationInfo) {
        let key = (relation.schema.clone(), relation.name.clone());
        if !self.sequences.contains(&key) {
            self.sequences.insert(key);
            let id = self.next_id();
            self.snapshot.sequences.push(Sequence {
                id,
                schema: relation.schema.clone(),
                name: relation.name.clone(),
            });
        }
    }

    fn allocate_type(&mut self, info: &TypeInfo) {
        let Some(TypeId::File(file_id)) = &info.id else {
            return;
        };
        if self
            .types
            .contains_key(&(info.schema.clone(), info.name.clone()))
        {
            return;
        }
        let id = self.next_id();
        self.type_ids.insert(TypeId::File(*file_id), id);
    }

    fn define_type(&mut self, (schema, name): &Key, info: &TypeInfo) {
        if self.types.contains_key(&(schema.clone(), name.clone())) {
            return;
        }
        let Some(id) = info.id.as_ref().and_then(|id| self.id_for(id)) else {
            return;
        };
        let attrs = info
            .attributes
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|attribute| PostgresTypeAttribute {
                name: attribute.name.clone(),
                type_id: attribute
                    .ty
                    .as_ref()
                    .and_then(|ty| match ty {
                        Type::Named(id) => self.id_for(id),
                        _ => None,
                    })
                    .unwrap_or_else(|| self.type_id(attribute.type_name.as_deref())),
            })
            .collect();
        let mut type_ = PostgresType {
            id,
            name: name.clone(),
            schema: schema.clone(),
            attributes: TypeAttributes { attrs },
            ..Default::default()
        };
        type_.typtype = info
            .kind
            .map(type_kind_code)
            .unwrap_or_default()
            .to_string();
        type_.typcategory = info.category.map(|c| c.to_string()).unwrap_or_default();
        type_.typispreferred = info.preferred.unwrap_or(false);
        type_.typelem = info
            .element
            .as_ref()
            .and_then(|id| self.id_for(id))
            .unwrap_or(0);
        type_.typarray = info
            .array
            .as_ref()
            .and_then(|id| self.id_for(id))
            .unwrap_or(0);
        type_.typbasetype = info
            .base
            .as_ref()
            .and_then(|id| self.id_for(id))
            .unwrap_or(0);
        type_.typrelid = info.relation.unwrap_or_else(|| {
            self.tables
                .get(&(schema.clone(), name.clone()))
                .copied()
                .unwrap_or(0)
        });
        type_.is_array = info.element.is_some();
        self.push_type(type_);
    }

    fn add_function(&mut self, (schema, name): &Key, info: &FunctionInfo) {
        let signature = info.signature.as_ref();
        let args = signature
            .map(|signature| {
                signature
                    .arguments
                    .iter()
                    .map(|argument| crate::snapshot::FunctionArg {
                        mode: match argument.mode {
                            FunctionArgumentMode::In => "in",
                            FunctionArgumentMode::InOut => "inout",
                            FunctionArgumentMode::Variadic => "variadic",
                            FunctionArgumentMode::Out => "out",
                        }
                        .into(),
                        name: argument.name.clone().unwrap_or_default(),
                        type_id: argument
                            .ty
                            .as_ref()
                            .and_then(|id| self.id_for(id))
                            .unwrap_or(0),
                        has_default: Some(false),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let id = self.next_id();
        self.snapshot.functions.push(Function {
            id,
            schema: schema.clone(),
            name: name.clone(),
            kind: match info.kind {
                FunctionKind::Function => ProcKind::Function,
                FunctionKind::Aggregate => ProcKind::Aggregate,
                FunctionKind::Window => ProcKind::Window,
                FunctionKind::Procedure => ProcKind::Procedure,
            },
            is_set_returning_function: signature.map_or(info.returns_set, |s| s.returns_set),
            input_defaults: signature.map_or(0, |s| s.input_defaults as i16),
            variadic_type_id: signature
                .and_then(|s| s.variadic_element.as_ref())
                .and_then(|id| self.id_for(id))
                .unwrap_or(0),
            return_type_id: signature
                .and_then(|s| s.return_type.as_ref())
                .and_then(|id| self.id_for(id)),
            args: crate::snapshot::FunctionArgs { args },
            ..Default::default()
        });
    }

    fn id_for(&self, id: &TypeId) -> Option<i64> {
        match id {
            TypeId::Snapshot(id) => Some(*id),
            TypeId::File(_) => self.type_ids.get(id).copied(),
        }
    }

    /// The oid of a type, given its name as the catalog writes it (`int4`, `public.my_enum`).
    /// `0` if unknown.
    fn type_id(&self, type_name: Option<&str>) -> i64 {
        let Some(type_name) = type_name else {
            return 0;
        };
        let (schema, name) = match type_name.rsplit_once('.') {
            Some((schema, name)) => (Some(schema), name),
            None => (None, type_name),
        };
        // An unqualified name is the type in `pg_catalog`, or else the first of that name.
        let found = match schema {
            Some(schema) => self.types.get(&(schema.to_owned(), name.to_owned())),
            None => self
                .types
                .get(&("pg_catalog".to_owned(), name.to_owned()))
                .or_else(|| self.type_names.get(name)),
        };
        found.copied().unwrap_or(0)
    }
}

/// Adds a type to the indexes of [`Builder`], unless they have one of its name already.
fn index_type(
    types: &mut FxHashMap<Key, i64>,
    type_names: &mut FxHashMap<String, i64>,
    type_: &PostgresType,
) {
    types
        .entry((type_.schema.clone(), type_.name.clone()))
        .or_insert(type_.id);
    type_names.entry(type_.name.clone()).or_insert(type_.id);
}
