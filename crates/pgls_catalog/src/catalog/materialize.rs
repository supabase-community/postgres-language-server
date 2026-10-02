//! The catalog as a [`Snapshot`], for features that list database objects, like completions
//! and hover.

use std::sync::Arc;

use rustc_hash::FxHashMap;

use super::{Catalog, Entry, Key};
use crate::lookup::{ColumnInfo, FunctionKind, Origin, RelationInfo, RelationKind};
use crate::snapshot::{
    Column, ColumnClassKind, Function, PostgresType, PostgresTypeAttribute, ProcKind, Schema,
    Sequence, Snapshot, Table, TableKind, TypeAttributes,
};

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
        };

        for schema in &self.dropped_schemas {
            builder.drop_schema(schema);
        }
        for (name, exists) in &self.schemas {
            if *exists {
                builder.create_schema(name);
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
                Entry::Defined(type_) if type_.origin == Origin::File => {
                    builder.define_type(key, type_.attributes.as_deref());
                }
                Entry::Dropped => builder
                    .snapshot
                    .types
                    .retain(|type_| (&type_.schema, &type_.name) != (&key.0, &key.1)),
                _ => {}
            }
        }
        for (key, entry) in &self.functions {
            match entry {
                Entry::Defined(overloads) => {
                    for function in overloads {
                        if function.origin == Origin::File {
                            builder.add_function(key, function.kind, function.returns_set);
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

struct Builder {
    snapshot: Snapshot,
    /// Ids of objects created by the file are negative, so they never collide with oids.
    next_id: i64,
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
                    type_id: self.type_id(column.type_name.as_deref()),
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

    fn define_type(&mut self, (schema, name): &Key, attributes: Option<&[ColumnInfo]>) {
        let exists = self
            .snapshot
            .types
            .iter()
            .any(|type_| &type_.schema == schema && &type_.name == name);
        if exists {
            return;
        }
        let attrs = attributes
            .unwrap_or_default()
            .iter()
            .map(|attribute| PostgresTypeAttribute {
                name: attribute.name.clone(),
                type_id: self.type_id(attribute.type_name.as_deref()),
            })
            .collect();
        let id = self.next_id();
        self.snapshot.types.push(PostgresType {
            id,
            name: name.clone(),
            schema: schema.clone(),
            attributes: TypeAttributes { attrs },
            ..Default::default()
        });
    }

    fn add_function(&mut self, (schema, name): &Key, kind: FunctionKind, returns_set: bool) {
        let id = self.next_id();
        self.snapshot.functions.push(Function {
            id,
            schema: schema.clone(),
            name: name.clone(),
            kind: match kind {
                FunctionKind::Function => ProcKind::Function,
                FunctionKind::Aggregate => ProcKind::Aggregate,
                FunctionKind::Window => ProcKind::Window,
                FunctionKind::Procedure => ProcKind::Procedure,
            },
            is_set_returning_function: returns_set,
            ..Default::default()
        });
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
