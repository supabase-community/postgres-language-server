//! The catalog as a [`Snapshot`], for features that list database objects, like completions
//! and hover.

use std::sync::Arc;

use rustc_hash::FxHashMap;

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
            type_ids: rustc_hash::FxHashMap::default(),
        };

        for schema in &self.dropped_schemas {
            builder.drop_schema(schema);
        }
        for (name, exists) in &self.schemas {
            if *exists {
                builder.create_schema(name);
            }
        }
        for entry in self.types.values() {
            if let Entry::Defined(type_) = entry
                && type_.origin == Origin::File
            {
                builder.allocate_type(type_);
            }
        }
        for ((schema, name), entry) in &self.relations {
            match entry {
                Entry::Defined(relation) => builder.define_relation(relation),
                Entry::Dropped => builder.drop_relation(schema, name),
                Entry::Unknown => {}
            }
        }
        for (key, entry) in &self.types {
            match entry {
                Entry::Defined(type_) if type_.origin == Origin::File => {}
                Entry::Dropped => builder
                    .snapshot
                    .types
                    .retain(|type_| (&type_.schema, &type_.name) != (&key.0, &key.1)),
                _ => {}
            }
        }
        for (key, entry) in &self.types {
            if let Entry::Defined(type_) = entry
                && type_.origin == Origin::File
            {
                builder.define_type(key, type_);
            }
        }
        for (key, entry) in &self.functions {
            match entry {
                Entry::Defined(overloads) => {
                    for function in overloads {
                        if function.origin == Origin::File {
                            builder.add_function(key, function);
                        }
                    }
                }
                Entry::Dropped => builder
                    .snapshot
                    .functions
                    .retain(|function| (&function.schema, &function.name) != (&key.0, &key.1)),
                Entry::Unknown => {}
            }
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
    type_ids: rustc_hash::FxHashMap<TypeId, i64>,
}

impl Builder {
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

    fn drop_relation(&mut self, schema: &str, name: &str) {
        let snapshot = &mut self.snapshot;
        let is_relation = |s: &str, n: &str| s == schema && n == name;
        snapshot
            .tables
            .retain(|table| !is_relation(&table.schema, &table.name));
        snapshot
            .columns
            .retain(|column| !is_relation(&column.schema_name, &column.table_name));
        snapshot
            .sequences
            .retain(|sequence| !is_relation(&sequence.schema, &sequence.name));
        snapshot
            .indexes
            .retain(|index| !is_relation(&index.schema, &index.table_name));
        snapshot
            .policies
            .retain(|policy| !is_relation(&policy.schema_name, &policy.table_name));
        snapshot
            .triggers
            .retain(|trigger| !is_relation(&trigger.table_schema, &trigger.table_name));
    }

    fn define_relation(&mut self, relation: &RelationInfo) {
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
                return;
            }
        };

        let existing = self
            .snapshot
            .tables
            .iter()
            .find(|table| table.schema == relation.schema && table.name == relation.name)
            .map(|table| table.id);
        let table_id = match existing {
            Some(id) => id,
            None => {
                let id = self.next_id();
                self.snapshot.tables.push(Table {
                    id,
                    schema: relation.schema.clone(),
                    name: relation.name.clone(),
                    table_kind,
                    ..Default::default()
                });
                id
            }
        };

        // Without known columns, a database table keeps its columns.
        let Some(columns) = &relation.columns else {
            return;
        };

        let mut previous: FxHashMap<String, Column> = FxHashMap::default();
        self.snapshot.columns.retain(|column| {
            if column.table_oid == table_id {
                previous.insert(column.name.clone(), column.clone());
                false
            } else {
                true
            }
        });

        for (number, column) in columns.iter().enumerate() {
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

    fn define_sequence(&mut self, relation: &RelationInfo) {
        let exists =
            self.snapshot.sequences.iter().any(|sequence| {
                sequence.schema == relation.schema && sequence.name == relation.name
            });
        if !exists {
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
            .snapshot
            .types
            .iter()
            .any(|type_| type_.schema == info.schema && type_.name == info.name)
        {
            return;
        }
        let id = self.next_id();
        self.type_ids.insert(TypeId::File(*file_id), id);
    }

    fn define_type(&mut self, (schema, name): &Key, info: &TypeInfo) {
        let exists = self
            .snapshot
            .types
            .iter()
            .any(|type_| &type_.schema == schema && &type_.name == name);
        if exists {
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
            self.snapshot
                .tables
                .iter()
                .find(|table| table.schema == *schema && table.name == *name)
                .map_or(0, |table| table.id)
        });
        type_.is_array = info.element.is_some();
        self.snapshot.types.push(type_);
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
        let matches = |type_: &&PostgresType| {
            type_.name == name && schema.is_none_or(|schema| type_.schema == schema)
        };
        let types = &self.snapshot.types;
        types
            .iter()
            .filter(matches)
            .find(|type_| schema.is_some() || type_.schema == "pg_catalog")
            .or_else(|| types.iter().find(matches))
            .map_or(0, |type_| type_.id)
    }
}
