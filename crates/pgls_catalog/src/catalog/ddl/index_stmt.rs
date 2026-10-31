use pgls_query::{
    NodeEnum, NodeRef,
    protobuf::{AlterTableType, ConstrType, IndexStmt, Node, ObjectType},
};

use crate::catalog::{Catalog, Entry, key};
use crate::lookup::{IndexInfo, Lookup, Origin, RelationInfo, RelationKind};

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
    // Without a name, Postgres generates one (`ChooseIndexName`).
    if n.idxname.is_empty() {
        c.indexes_incomplete = true;
        return;
    }
    let index_key = key(&table.schema, &n.idxname);
    // With `IF NOT EXISTS`, `index_create` skips the index if any relation in the schema has its
    // name, not only an index, and `DefineIndex` then skips the indexes of the partitions too.
    // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/index.c#L894
    // https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/indexcmds.c#L1276
    let taken = if n.if_not_exists {
        name_taken(c, &index_key.0, &index_key.1)
    } else {
        Lookup::Missing
    };
    if let Lookup::Found(()) = taken {
        return;
    }
    // On a partitioned table, Postgres also creates an index with a generated name on each
    // partition.
    if table.kind == RelationKind::PartitionedTable {
        c.indexes_incomplete = true;
    }
    let columns = index_columns(n, &table);
    let entry = match taken {
        Lookup::Unknown => Entry::Unknown,
        _ => Entry::Defined(IndexInfo {
            schema: table.schema,
            name: n.idxname.clone(),
            table: table.name,
            columns,
            origin: Origin::File,
        }),
    };
    c.indexes.insert(index_key, entry);
}

/// Whether a relation of any kind has the name in the schema. Indexes share their namespace with
/// tables, views, sequences and composite types.
fn name_taken(c: &Catalog, schema: &str, name: &str) -> Lookup<()> {
    match (c.index_in(schema, name), c.relation_in(schema, name)) {
        (Lookup::Found(_), _) | (_, Lookup::Found(_)) => Lookup::Found(()),
        (Lookup::Unknown, _) | (_, Lookup::Unknown) => Lookup::Unknown,
        (Lookup::Missing, Lookup::Missing) => Lookup::Missing,
    }
}

/// The columns an index uses: those of its keys and `INCLUDE` list, and those its expressions
/// and predicate reference. [`index_create`] makes the index depend on each of them, so dropping
/// one drops the index. A whole-row reference adds no column: [`find_expr_references_walker`]
/// records none for it, so dropping a column keeps the index. `None` if an expression references
/// a name we can't resolve.
///
/// [`index_create`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/index.c#L1132
/// [`find_expr_references_walker`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/dependency.c#L1782
fn index_columns(n: &IndexStmt, table: &RelationInfo) -> Option<Vec<String>> {
    let mut columns = Vec::new();
    let mut expressions = Vec::new();
    for element in n.index_params.iter().chain(&n.index_including_params) {
        let NodeEnum::IndexElem(element) = element.node.as_ref()? else {
            return None;
        };
        if element.name.is_empty() {
            expressions.push(element.expr.as_deref()?.node.as_ref()?);
        } else {
            columns.push(element.name.clone());
        }
    }
    expressions.extend(
        n.where_clause
            .as_deref()
            .and_then(|clause| clause.node.as_ref()),
    );
    for expression in expressions {
        for node in expression.iter() {
            let NodeRef::ColumnRef(column) = node else {
                continue;
            };
            match column_ref(&column.fields, table)? {
                ColumnUse::Column(name) => columns.push(name),
                ColumnUse::WholeRow => {}
            }
        }
    }
    Some(columns)
}

/// What a column reference in an index expression uses of the table.
enum ColumnUse {
    Column(String),
    WholeRow,
}

/// What a column reference in an index expression uses, like [`transformColumnRef`] resolves it
/// with the table as the only relation in scope: `a` is the column `a`, or else the whole row of
/// a table named `a`, `t.*` is the whole row of the table `t`, and `a.b` is the column `b` of the
/// table `a`, or else the field `b` of the column `a`. `None` for names we don't resolve.
///
/// [`transformColumnRef`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L509
fn column_ref(fields: &[Node], table: &RelationInfo) -> Option<ColumnUse> {
    // A name, or `None` for `*`.
    let names = fields
        .iter()
        .map(|field| match field.node.as_ref()? {
            NodeEnum::String(name) => Some(Some(name.sval.as_str())),
            NodeEnum::AStar(_) => Some(None),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let columns = table.columns.as_ref()?;
    let is_column = |name: &str| columns.iter().any(|column| column.name == name);
    let column = match names.as_slice() {
        [Some(name)] if !is_column(name) && *name == table.name => {
            return Some(ColumnUse::WholeRow);
        }
        [Some(name)] => name,
        [Some(qualifier), None] if *qualifier == table.name => return Some(ColumnUse::WholeRow),
        [Some(qualifier), Some(name)] if *qualifier == table.name => name,
        [Some(column), Some(_)] => column,
        _ => return None,
    };
    is_column(column).then(|| ColumnUse::Column((*column).to_owned()))
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
