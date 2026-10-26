use pgls_query::{
    NodeEnum,
    protobuf::{AlterTableType, ConstrType, IndexStmt, Node, ObjectType},
};

use crate::catalog::{Catalog, Entry, key};
use crate::lookup::{IndexInfo, Lookup, Origin, RelationKind};

/// `CREATE INDEX`. Models where [`DefineIndex`] puts the index: in the schema of its table.
///
/// [`DefineIndex`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/indexcmds.c#L542
pub(super) fn apply_index_stmt(c: &mut Catalog, n: &IndexStmt, search_path: &[String]) {
    let Some(range_var) = &n.relation else {
        return;
    };
    let table = match c.relation_of(range_var, search_path) {
        // Postgres rejects indexes on system catalogs.
        Lookup::Found(table) if table.schema == "pg_catalog" => return,
        Lookup::Found(table) => table,
        Lookup::Missing => return,
        Lookup::Unknown => {
            c.indexes_incomplete = true;
            return;
        }
    };
    // Without a name, Postgres generates one (`ChooseIndexName`). On a partitioned table, it
    // also creates an index with a generated name on each partition (`DefineIndex`).
    if n.idxname.is_empty() || table.kind == RelationKind::PartitionedTable {
        c.indexes_incomplete = true;
    }
    if n.idxname.is_empty() {
        return;
    }
    let index_key = key(&table.schema, &n.idxname);
    if n.if_not_exists {
        match c.index_in(&index_key.0, &index_key.1) {
            Lookup::Found(_) => return,
            Lookup::Unknown => {
                c.indexes.insert(index_key, Entry::Unknown);
                return;
            }
            Lookup::Missing => {}
        }
    }
    c.indexes.insert(
        index_key,
        Entry::Defined(IndexInfo {
            schema: table.schema,
            name: n.idxname.clone(),
            table: table.name,
            origin: Origin::File,
        }),
    );
}

/// Whether a statement may create or rename indexes other than with `CREATE INDEX` and
/// `ALTER INDEX ... RENAME`: primary keys, unique and exclusion constraints, `LIKE ... INCLUDING
/// INDEXES`, partitions that inherit the indexes of their parent, and `RENAME CONSTRAINT`. Their
/// names are generated or come from the constraint, so we don't track them.
pub(super) fn creates_implicit_indexes(stmt: &NodeEnum) -> bool {
    match stmt {
        NodeEnum::CreateStmt(n) => {
            n.partbound.is_some()
                || n.table_elts.iter().any(|element| match &element.node {
                    Some(NodeEnum::Constraint(constraint)) => constraint_creates_index(constraint),
                    Some(NodeEnum::ColumnDef(column)) => column_creates_index(&column.constraints),
                    Some(NodeEnum::TableLikeClause(_)) => true,
                    _ => false,
                })
        }
        NodeEnum::AlterTableStmt(n) => n.cmds.iter().any(|command| {
            let Some(NodeEnum::AlterTableCmd(command)) = &command.node else {
                return false;
            };
            match command.subtype() {
                AlterTableType::AtAddIndex
                | AlterTableType::AtAddIndexConstraint
                | AlterTableType::AtAttachPartition => true,
                AlterTableType::AtAddConstraint => matches!(
                    command.def.as_ref().and_then(|def| def.node.as_ref()),
                    Some(NodeEnum::Constraint(constraint)) if constraint_creates_index(constraint)
                ),
                AlterTableType::AtAddColumn => matches!(
                    command.def.as_ref().and_then(|def| def.node.as_ref()),
                    Some(NodeEnum::ColumnDef(column)) if column_creates_index(&column.constraints)
                ),
                _ => false,
            }
        }),
        NodeEnum::RenameStmt(n) => n.rename_type() == ObjectType::ObjectTabconstraint,
        _ => false,
    }
}

fn constraint_creates_index(constraint: &pgls_query::protobuf::Constraint) -> bool {
    matches!(
        constraint.contype(),
        ConstrType::ConstrPrimary | ConstrType::ConstrUnique | ConstrType::ConstrExclusion
    )
}

fn column_creates_index(constraints: &[Node]) -> bool {
    constraints.iter().any(|constraint| {
        matches!(&constraint.node, Some(NodeEnum::Constraint(constraint)) if constraint_creates_index(constraint))
    })
}
