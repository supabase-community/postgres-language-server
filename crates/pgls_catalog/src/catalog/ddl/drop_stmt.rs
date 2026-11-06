use pgls_query::{
    NodeEnum,
    protobuf::{DropStmt, ObjectType},
};

use crate::catalog::{
    Catalog, Entry, key,
    names::{qualified_name, type_name},
};
use crate::lookup::{Lookup, RelationKind};

/// `DROP TABLE`, `DROP TYPE`, `DROP FUNCTION`, and the other `DROP` statements. Models
/// [`RemoveRelations`] and [`RemoveObjects`].
///
/// [`RemoveRelations`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L1547
/// [`RemoveObjects`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/dropcmds.c#L53
pub(super) fn apply_drop_stmt(c: &mut Catalog, n: &DropStmt, search_path: &[String]) {
    match n.remove_type() {
        ObjectType::ObjectCast => c.casts_incomplete = true,
        ObjectType::ObjectOperator => c.operators_incomplete = true,
        _ => {}
    }
    for object in &n.objects {
        let Some(object) = &object.node else {
            continue;
        };
        match n.remove_type() {
            ObjectType::ObjectTable
            | ObjectType::ObjectView
            | ObjectType::ObjectMatview
            | ObjectType::ObjectForeignTable
            | ObjectType::ObjectSequence => {
                let NodeEnum::List(list) = object else {
                    continue;
                };
                let Some(name) = qualified_name(&list.items) else {
                    continue;
                };
                if let Some(key) = c
                    .relation_key(name.schema(), &name.name, search_path)
                    .filter(|key| key.0 != "pg_catalog")
                {
                    c.drop_table_indexes(&key.0, &key.1);
                    c.types.insert(key.clone(), Entry::Dropped);
                    c.relations.insert(key, Entry::Dropped);
                }
            }
            ObjectType::ObjectIndex => {
                let NodeEnum::List(list) = object else {
                    continue;
                };
                match c.index_of(&list.items, search_path) {
                    Lookup::Found(index) => {
                        let is_partitioned = !matches!(
                            c.index_table(&index),
                            Lookup::Found(table) if table.kind != RelationKind::PartitionedTable
                        );
                        c.indexes
                            .insert(key(&index.schema, &index.name), Entry::Dropped);
                        if is_partitioned {
                            c.drop_partition_indexes();
                        }
                    }
                    // It may be the index of a partitioned table.
                    Lookup::Unknown => c.drop_partition_indexes(),
                    Lookup::Missing => {}
                }
            }
            ObjectType::ObjectType | ObjectType::ObjectDomain => {
                let NodeEnum::TypeName(type_) = object else {
                    continue;
                };
                let Some(name) = type_name(type_) else {
                    continue;
                };
                if let Some(key) = c.type_key(name.schema(), &name.name, search_path) {
                    c.types.insert(key.clone(), Entry::Dropped);
                    let array_key = (key.0, format!("_{}", key.1));
                    if c.types.contains_key(&array_key) {
                        c.types.insert(array_key, Entry::Dropped);
                    }
                    c.casts_incomplete = true;
                }
            }
            ObjectType::ObjectFunction
            | ObjectType::ObjectProcedure
            | ObjectType::ObjectRoutine
            | ObjectType::ObjectAggregate => {
                if let NodeEnum::ObjectWithArgs(function) = object {
                    c.take_function(function, search_path);
                }
            }
            ObjectType::ObjectSchema => {
                if let NodeEnum::String(name) = object {
                    c.drop_schema(&name.sval);
                }
            }
            _ => {}
        }
    }
}
