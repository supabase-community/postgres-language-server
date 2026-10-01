use pgls_query::{
    NodeEnum,
    protobuf::{ObjectType, RangeVar, RenameStmt},
};

use crate::catalog::{
    Catalog, Entry, key,
    names::{qualified_name, range_var_name},
};
use crate::lookup::{CatalogView, ColumnInfo, Origin};

pub(super) fn apply_rename_stmt(c: &mut Catalog, n: &RenameStmt, search_path: &[String]) {
    let object = n.object.as_deref().and_then(|object| object.node.as_ref());
    match n.rename_type() {
        ObjectType::ObjectTable
        | ObjectType::ObjectView
        | ObjectType::ObjectMatview
        | ObjectType::ObjectForeignTable
        | ObjectType::ObjectSequence => {
            if let Some(range_var) = &n.relation {
                c.move_relation(range_var, None, Some(&n.newname), search_path);
            }
        }
        ObjectType::ObjectColumn => {
            if let Some(range_var) = &n.relation {
                rename_column(c, range_var, &n.subname, &n.newname, search_path);
            }
        }
        ObjectType::ObjectAttribute => {
            if let Some(range_var) = &n.relation {
                let name = range_var_name(range_var);
                if let Some(mut type_info) = c.type_(name.schema(), &name.name, search_path).found()
                {
                    rename_in(&mut type_info.attributes, &n.subname, &n.newname);
                    type_info.origin = Origin::File;
                    let key = key(&type_info.schema, &type_info.name);
                    c.types.insert(key, Entry::Defined(type_info));
                }
            }
        }
        ObjectType::ObjectType | ObjectType::ObjectDomain => {
            if let Some(NodeEnum::List(list)) = object {
                if let Some(name) = qualified_name(&list.items) {
                    c.move_type(&name, None, Some(&n.newname), search_path);
                }
            }
        }
        ObjectType::ObjectFunction
        | ObjectType::ObjectProcedure
        | ObjectType::ObjectRoutine
        | ObjectType::ObjectAggregate => {
            if let Some(NodeEnum::ObjectWithArgs(function)) = object {
                c.move_function(function, None, Some(&n.newname), search_path);
            }
        }
        // Moving every object of a schema to a new name is not modelled.
        ObjectType::ObjectSchema => c.tainted = true,
        _ => {}
    }
}

fn rename_column(
    c: &mut Catalog,
    range_var: &RangeVar,
    old_name: &str,
    new_name: &str,
    search_path: &[String],
) {
    let Some(relation) = c.relation_of(range_var, search_path).found() else {
        return;
    };
    c.change_columns(&relation, |columns| rename_in(columns, old_name, new_name));
}

fn rename_in(columns: &mut Option<Vec<ColumnInfo>>, old_name: &str, new_name: &str) {
    if let Some(column) = columns
        .iter_mut()
        .flatten()
        .find(|column| column.name == old_name)
    {
        column.name = new_name.to_owned();
    }
}
