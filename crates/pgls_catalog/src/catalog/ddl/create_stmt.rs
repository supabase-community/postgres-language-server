use pgls_query::{NodeEnum, protobuf::CreateStmt};

use crate::catalog::{Catalog, key, names::type_name, overlay::column_info};
use crate::lookup::{CatalogView, ColumnInfo, Origin, RelationInfo, RelationKind};
use crate::{Generated, Identity};

/// `CREATE_TABLE_LIKE_GENERATED` of `TableLikeOption`, the bit of `INCLUDING GENERATED`:
/// https://github.com/postgres/postgres/blob/REL_18_6/src/include/nodes/parsenodes.h#L791
const CREATE_TABLE_LIKE_GENERATED: u32 = 1 << 4;
/// `CREATE_TABLE_LIKE_IDENTITY`, the bit of `INCLUDING IDENTITY`.
const CREATE_TABLE_LIKE_IDENTITY: u32 = 1 << 5;

/// `CREATE TABLE`. Models [`DefineRelation`], which merges inherited columns with
/// [`MergeAttributes`], and the columns copied by `LIKE` ([`transformTableLikeClause`]).
///
/// [`DefineRelation`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L765
/// [`MergeAttributes`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L2555
/// [`transformTableLikeClause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_utilcmd.c#L1126
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
    // Whether a column is defined more than once, or an unknown one might be.
    let mut merged = false;
    let mut add_columns = |new: Option<Vec<ColumnInfo>>| match (&mut columns, new) {
        (Some(columns), Some(new)) => {
            for column in new {
                match columns.iter_mut().find(|c| c.name == column.name) {
                    Some(existing) => {
                        merge_column(existing, &column);
                        merged = true;
                    }
                    None => columns.push(column),
                }
            }
        }
        _ => {
            columns = None;
            merged = true;
        }
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

    // Inherited columns (`INHERITS`, `PARTITION OF`) come first. A column inherited from a
    // generated column is generated too. Only partitions inherit identity, since Postgres 17.
    let inherits_identity = n.partbound.is_some() && c.partitions_inherit_identity();
    let mut parents = Vec::new();
    for parent in &n.inh_relations {
        let parent = match parent.node.as_ref() {
            Some(NodeEnum::RangeVar(parent)) => c.relation_of(parent, search_path).found(),
            _ => None,
        };
        if let Some(parent) = &parent {
            parents.push(key(&parent.schema, &parent.name));
        }
        let mut columns = parent.and_then(|parent| parent.columns);
        if !inherits_identity {
            for column in columns.iter_mut().flatten() {
                column.identity = Identity::No;
            }
        }
        add_columns(columns);
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
                let mut columns = source.and_then(|source| source.columns);
                // Columns copied by `LIKE` are only generated with `INCLUDING GENERATED`, and
                // only identity columns with `INCLUDING IDENTITY`, which foreign tables ignore.
                let copies_identity = like.options & CREATE_TABLE_LIKE_IDENTITY != 0
                    && kind != RelationKind::ForeignTable;
                for column in columns.iter_mut().flatten() {
                    if like.options & CREATE_TABLE_LIKE_GENERATED == 0 {
                        column.generated = Generated::No;
                    }
                    if !copies_identity {
                        column.identity = Identity::No;
                    }
                }
                add_columns(columns);
            }
            _ => {}
        }
    }

    // A column of an inheritance child that is also defined by the table or inherited from
    // another parent is kept when the parent drops it. Partitions have a single parent and no
    // columns of their own.
    if merged && n.partbound.is_none() && !parents.is_empty() {
        c.children_with_own_columns.insert(new_key.clone());
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

/// Merges a column definition into an earlier one with the same name: an inherited column
/// redeclared by the table, or inherited from several parents. The first definition keeps its
/// place and type. The merged column is generated if either definition is ([`MergeChildAttribute`],
/// [`MergeInheritedAttribute`]): a generated parent column makes the redeclared column generated,
/// and on Postgres 15 a generation expression of the table makes an ordinary parent column
/// generated. The combinations Postgres rejects (a generated parent column redeclared with
/// another kind, on Postgres 16+ a generation expression for an ordinary parent column, and
/// parents that disagree) fail the `CREATE TABLE`, so they don't matter.
///
/// The identity is the one the table declares: inheritance children don't inherit identity, and
/// partitions, which do, can't declare one.
///
/// [`MergeChildAttribute`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L3255
/// [`MergeInheritedAttribute`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L3425
fn merge_column(existing: &mut ColumnInfo, column: &ColumnInfo) {
    if existing.generated == Generated::No {
        existing.generated = column.generated;
    }
    if column.identity != Identity::No {
        existing.identity = column.identity;
    }
}
