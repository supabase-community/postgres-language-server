use pgls_query::{
    NodeEnum,
    protobuf::{DropStmt, ObjectType},
};

use crate::catalog::{
    Catalog, Entry,
    names::{qualified_name, type_name},
};

pub(super) fn apply_drop_stmt(c: &mut Catalog, n: &DropStmt, search_path: &[String]) {
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
                if let Some(key) = c.relation_key(name.schema(), &name.name, search_path) {
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
                    c.types.insert(key, Entry::Dropped);
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
