use pgls_query::{
    NodeEnum,
    protobuf::{DropStmt, ObjectType},
};

use crate::catalog::{
    Catalog, Entry,
    names::{qualified_name, type_name},
};

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
                    c.types.insert(key.clone(), Entry::Dropped);
                    c.relations.insert(key, Entry::Dropped);
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
