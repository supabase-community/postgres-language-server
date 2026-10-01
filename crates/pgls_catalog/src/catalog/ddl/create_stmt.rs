use pgls_query::{NodeEnum, protobuf::CreateStmt};

use crate::catalog::{Catalog, key, names::type_name, overlay::column_info};
use crate::lookup::{CatalogView, ColumnInfo, Origin, RelationInfo, RelationKind};

pub(super) fn apply_create_stmt(c: &mut Catalog, n: &CreateStmt, search_path: &[String]) {
    create_table(c, n, RelationKind::Table, search_path);
}

/// Creates a table, or a foreign table with `kind: ForeignTable`.
pub(super) fn create_table(
    c: &mut Catalog,
    n: &CreateStmt,
    kind: RelationKind,
    search_path: &[String],
) {
    let Some(range_var) = &n.relation else {
        return;
    };
    let Some(new_key) = c.relation_creation_key(range_var, search_path) else {
        return;
    };
    if c.skip_existing_relation(&new_key, n.if_not_exists) {
        return;
    }

    let mut columns: Option<Vec<ColumnInfo>> = Some(Vec::new());
    let mut add_columns = |new: Option<Vec<ColumnInfo>>| match (&mut columns, new) {
        (Some(columns), Some(new)) => {
            for column in new {
                if !columns.iter().any(|c: &ColumnInfo| c.name == column.name) {
                    columns.push(column);
                }
            }
        }
        _ => columns = None,
    };

    // Typed tables (`OF type`) take the attributes of the type.
    if let Some(of_type) = &n.of_typename {
        let attributes = type_name(of_type).and_then(|name| {
            c.type_(name.schema(), &name.name, search_path)
                .found()
                .and_then(|type_info| type_info.attributes)
        });
        add_columns(attributes);
    }

    // Inherited columns (`INHERITS`, `PARTITION OF`) come first.
    let mut parents = Vec::new();
    for parent in &n.inh_relations {
        let parent = match parent.node.as_ref() {
            Some(NodeEnum::RangeVar(parent)) => c.relation_of(parent, search_path).found(),
            _ => None,
        };
        if let Some(parent) = &parent {
            parents.push(key(&parent.schema, &parent.name));
        }
        add_columns(parent.and_then(|parent| parent.columns));
    }

    for element in &n.table_elts {
        match element.node.as_ref() {
            Some(NodeEnum::ColumnDef(column)) => {
                add_columns(Some(vec![column_info(c, column, search_path)]))
            }
            Some(NodeEnum::TableLikeClause(like)) => {
                let source = like
                    .relation
                    .as_ref()
                    .and_then(|source| c.relation_of(source, search_path).found());
                add_columns(source.and_then(|source| source.columns));
            }
            _ => {}
        }
    }

    for parent in parents {
        c.children.entry(parent).or_default().push(new_key.clone());
    }

    let kind = if n.partspec.is_some() {
        RelationKind::PartitionedTable
    } else {
        kind
    };
    c.define_relation(RelationInfo {
        schema: new_key.0,
        name: new_key.1,
        kind,
        columns,
        origin: Origin::File,
    });
}
