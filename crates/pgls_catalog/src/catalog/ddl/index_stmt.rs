use pgls_query::{
    NodeEnum, NodeRef,
    protobuf::{AlterTableType, ConstrType, IndexStmt, Node, ObjectType},
};
use pgls_query_ext::utils::{alter_table_cmds, cmd_def};

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
        c.relation_or_index_in(&index_key.0, &index_key.1)
    } else {
        Lookup::Missing
    };
    if let Lookup::Found(()) = taken {
        c.skipped_existing = true;
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
        _ => Entry::defined(IndexInfo {
            schema: table.schema,
            name: n.idxname.clone(),
            table: table.name,
            columns,
            origin: Origin::File,
        }),
    };
    c.indexes.insert(index_key, entry);
}

/// The columns an index uses: those of its keys and `INCLUDE` list, and those its expressions
/// and predicate reference. [`index_create`] makes the index depend on each of them, so dropping
/// one drops the index. A whole-row reference adds no column: [`find_expr_references_walker`]
/// records none for it, so dropping a column keeps the index. But `ROW(t.*)` expands to all
/// columns ([`transformRowExpr`]), and `(t).b` selects the column `b` ([`ParseComplexProjection`]).
/// `None` if an expression references a name we can't resolve, and for `b(t)`, which is the
/// column `b` unless there is a function `b` ([`ParseFuncOrColumn`]).
///
/// [`index_create`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/index.c#L1132
/// [`find_expr_references_walker`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/dependency.c#L1782
/// [`transformRowExpr`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_expr.c#L2188
/// [`ParseComplexProjection`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_func.c#L1967
/// [`ParseFuncOrColumn`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_func.c#L223
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
            match node {
                NodeRef::ColumnRef(column) => match column_ref(&column.fields, table)? {
                    ColumnUse::Column(name) => columns.push(name),
                    ColumnUse::WholeRow => {}
                },
                NodeRef::RowExpr(row) => {
                    for arg in &row.args {
                        let Some(NodeEnum::ColumnRef(column)) = arg.node.as_ref() else {
                            continue;
                        };
                        let is_star = matches!(
                            column.fields.last().and_then(|field| field.node.as_ref()),
                            Some(NodeEnum::AStar(_))
                        );
                        if is_star && is_whole_row(arg, table) {
                            let all = table.columns.as_ref()?;
                            columns.extend(all.iter().map(|column| column.name.clone()));
                        }
                    }
                }
                NodeRef::AIndirection(indirection) => {
                    let first = indirection.indirection.first()?.node.as_ref()?;
                    let whole_row = indirection
                        .arg
                        .as_deref()
                        .is_some_and(|arg| is_whole_row(arg, table));
                    match first {
                        NodeEnum::String(name) if whole_row => {
                            // Otherwise a function on the whole row, or an error.
                            let columns_of_table = table.columns.as_ref()?;
                            if !columns_of_table.iter().any(|known| known.name == name.sval) {
                                return None;
                            }
                            columns.push(name.sval.clone());
                        }
                        // `(t).*` expands to all columns in `ROW`, and fails elsewhere.
                        NodeEnum::AStar(_) => return None,
                        _ if whole_row => return None,
                        _ => {}
                    }
                }
                NodeRef::FuncCall(call) => {
                    let could_be_projection = call.funcname.len() == 1
                        && call.agg_order.is_empty()
                        && call.agg_filter.is_none()
                        && !call.agg_star
                        && !call.agg_distinct
                        && call.over.is_none()
                        && !call.func_variadic
                        && matches!(call.args.as_slice(), [arg] if is_whole_row(arg, table));
                    let name = call
                        .funcname
                        .first()
                        .and_then(|name| match name.node.as_ref() {
                            Some(NodeEnum::String(name)) => Some(name.sval.as_str()),
                            _ => None,
                        });
                    let is_column = |name: &str| {
                        table
                            .columns
                            .as_ref()
                            .is_none_or(|known| known.iter().any(|column| column.name == name))
                    };
                    if could_be_projection && name.is_none_or(is_column) {
                        return None;
                    }
                }
                _ => {}
            }
        }
    }
    Some(columns)
}

/// Whether an expression is a whole-row reference to the table: `t` or `t.*`.
fn is_whole_row(node: &Node, table: &RelationInfo) -> bool {
    match node.node.as_ref() {
        Some(NodeEnum::ColumnRef(column)) => {
            matches!(column_ref(&column.fields, table), Some(ColumnUse::WholeRow))
        }
        _ => false,
    }
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
        NodeEnum::AlterTableStmt(n) => alter_table_cmds(n).any(|command| match command.subtype() {
            AlterTableType::AtAddIndex
            | AlterTableType::AtAddIndexConstraint
            | AlterTableType::AtAttachPartition => true,
            AlterTableType::AtAddConstraint => matches!(
                cmd_def(command),
                Some(NodeEnum::Constraint(constraint)) if constraint_creates_index(constraint)
            ),
            AlterTableType::AtAddColumn => matches!(
                cmd_def(command),
                Some(NodeEnum::ColumnDef(column)) if column_creates_index(&column.constraints)
            ),
            _ => false,
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
