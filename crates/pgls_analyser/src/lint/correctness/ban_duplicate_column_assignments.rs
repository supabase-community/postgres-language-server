use crate::fix::StatementTokens;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{
    AlterTableStmt, AlterTableType, CreateTableAsStmt, Node, RangeVar, ResTarget, Token,
};
use pgls_query::{NodeEnum, NodeRef};
use pgls_query_ext::utils::{alter_table_cmds, cmd_def};
use pgls_resolve::{QueryOutput, Star};
use pgls_text_size::TextRange;

declare_lint_rule! {
    /// A column is assigned or declared more than once.
    ///
    /// Postgres rejects statements that assign the same column twice in a SET clause, like
    /// `update t set a = 1, a = 2`, with `42601 syntax_error`. It rejects column lists that name
    /// the same column twice, like `insert into t (a, a)` or `copy t (a, a)`, tables, views and
    /// composite types that declare the same column twice, and `ALTER TABLE` commands that add
    /// the same column twice with `42701 duplicate_column`. This includes view columns that get
    /// the same name from their expressions, like `select a.id, b.id`, or from the tables a `*`
    /// expands to, like `select a.*, b.*` when both tables have an `id`.
    ///
    /// Assigning different elements or fields of the same column, like `set a[1] = 1, a[2] = 2`,
    /// is fine.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// update users set name = 'a', name = 'b';
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// create table users (id bigint, id text);
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// create view user_ids as select 1 as id, 2 as id;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// update users set name = 'b';
    /// ```
    ///
    pub BanDuplicateColumnAssignments {
        version: "next",
        name: "banDuplicateColumnAssignments",
        severity: Severity::Error,
        recommended: true,
        sources: &[RuleSource::Squawk("ban-duplicate-column-assignments")],
        fix_kind: FixKind::Unsafe,
    }
}

impl LinterRule for BanDuplicateColumnAssignments {
    type Options = ();
    // Postgres' rewriter raises `multiple assignments to same column`, without a position.
    const REPORTS_DATABASE_ERROR: bool = true;

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let tokens = ctx.tokens();
        let mut diagnostics = Vec::new();

        for node in ctx.stmt().iter() {
            match node {
                NodeRef::UpdateStmt(stmt) => {
                    report_assignments(&stmt.target_list, tokens, &mut diagnostics)
                }
                NodeRef::OnConflictClause(clause) => {
                    report_assignments(&clause.target_list, tokens, &mut diagnostics)
                }
                NodeRef::MergeWhenClause(clause) => {
                    report_assignments(&clause.target_list, tokens, &mut diagnostics)
                }
                NodeRef::InsertStmt(stmt) => {
                    report_assignments(&stmt.cols, tokens, &mut diagnostics)
                }
                NodeRef::CopyStmt(stmt) => report_definitions(
                    column_list(&stmt.attlist, stmt.relation.as_ref(), tokens),
                    Definition::ColumnList,
                    &mut diagnostics,
                ),
                NodeRef::CreateStmt(stmt) => report_definitions(
                    column_definitions(&stmt.table_elts, tokens),
                    Definition::Relation,
                    &mut diagnostics,
                ),
                NodeRef::CompositeTypeStmt(stmt) => report_definitions(
                    column_definitions(&stmt.coldeflist, tokens),
                    Definition::Relation,
                    &mut diagnostics,
                ),
                NodeRef::AlterTableStmt(stmt) => report_definitions(
                    added_columns(stmt, tokens),
                    Definition::AddedColumn,
                    &mut diagnostics,
                ),
                // The columns of the relation that the statement creates come from the output of
                // its query, so only the statement itself counts, not a view among the elements
                // of `CREATE SCHEMA`.
                NodeRef::ViewStmt(stmt) if is_statement(ctx, node) => report_definitions(
                    output_columns(
                        column_list(&stmt.aliases, stmt.view.as_ref(), tokens),
                        ctx.output(),
                    ),
                    Definition::Relation,
                    &mut diagnostics,
                ),
                // Postgres skips the statement without looking at its columns if the relation
                // exists.
                NodeRef::CreateTableAsStmt(stmt) if may_skip(stmt, ctx) => {}
                NodeRef::CreateTableAsStmt(stmt) if is_statement(ctx, node) => {
                    let names = stmt.into.as_ref().map_or(Vec::new(), |into| {
                        column_list(&into.col_names, into.rel.as_ref(), tokens)
                    });
                    report_definitions(
                        output_columns(names, ctx.output()),
                        Definition::Relation,
                        &mut diagnostics,
                    );
                }
                // The `INTO` of a set operation is on its left-most select.
                NodeRef::SelectStmt(stmt)
                    if stmt.into_clause.is_some()
                        && matches!(ctx.stmt(), NodeEnum::SelectStmt(_)) =>
                {
                    report_definitions(
                        output_columns(Vec::new(), ctx.output()),
                        Definition::Relation,
                        &mut diagnostics,
                    )
                }
                _ => {}
            }
        }

        diagnostics
    }
}

/// Whether `node` is the statement itself rather than a node inside it.
fn is_statement(ctx: &LinterRuleContext<BanDuplicateColumnAssignments>, node: NodeRef) -> bool {
    match (node, ctx.stmt()) {
        (NodeRef::ViewStmt(node), NodeEnum::ViewStmt(stmt)) => std::ptr::eq(node, stmt.as_ref()),
        (NodeRef::CreateTableAsStmt(node), NodeEnum::CreateTableAsStmt(stmt)) => {
            std::ptr::eq(node, stmt.as_ref())
        }
        _ => false,
    }
}

/// Whether a `CREATE TABLE AS` or `CREATE MATERIALIZED VIEW` with `IF NOT EXISTS` may be
/// skipped, because the catalog doesn't know that the relation is missing.
fn may_skip(
    stmt: &CreateTableAsStmt,
    ctx: &LinterRuleContext<BanDuplicateColumnAssignments>,
) -> bool {
    stmt.if_not_exists
        && !stmt
            .into
            .as_ref()
            .and_then(|into| into.rel.as_ref())
            .is_some_and(|relation| {
                ctx.catalog()
                    .relation_at_creation(relation, ctx.session().search_path())
                    .is_missing()
            })
}

/// A column name, and where it is written if we know that.
struct Column {
    name: String,
    span: Option<TextRange>,
}

/// Reports the columns of a SET clause or an INSERT column list that are assigned more than
/// once. Assigning distinct subscripts or fields of a column is allowed, so a column is only
/// reported if one of its assignments is to the whole column.
fn report_assignments(
    targets: &[Node],
    tokens: Option<StatementTokens>,
    diagnostics: &mut Vec<LinterDiagnostic>,
) {
    let targets: Vec<&ResTarget> = targets
        .iter()
        .filter_map(|target| match &target.node {
            Some(NodeEnum::ResTarget(target)) => Some(target.as_ref()),
            _ => None,
        })
        .collect();
    let assignments = targets.iter().map(|target| {
        (
            Column {
                name: target.name.clone(),
                span: tokens.and_then(|tokens| tokens.range_at(target.location)),
            },
            target.indirection.is_empty(),
        )
    });

    for column in duplicates(assignments) {
        let name = &column.name;
        diagnostics.push(
            LinterDiagnostic::new(
                rule_category!(),
                column.span,
                markup! {
                    "Multiple assignments to column "<Emphasis>{name}</Emphasis>"."
                },
            )
            .detail(
                None,
                "Postgres rejects statements that assign a column more than once.",
            )
            .note("Remove all but one of the assignments.")
            .maybe_fix(keep_last_assignment(&targets, name, tokens)),
        );
    }
}

/// Removes all but the last assignment to `column` from a SET clause, if each of them assigns
/// the whole column on its own, like `name = 'a'`. We can't tell which value was meant, so the
/// fix is unsafe.
fn keep_last_assignment(
    targets: &[&ResTarget],
    column: &str,
    tokens: Option<StatementTokens>,
) -> Option<RuleFix> {
    let tokens = tokens?;
    let assignments: Vec<&ResTarget> = targets
        .iter()
        .copied()
        .filter(|target| target.name == column)
        .collect();
    // INSERT column lists have no values, and `(a, b) = ...` assigns several columns at once.
    let simple = assignments.iter().all(|target| {
        target.indirection.is_empty()
            && target.val.as_deref().is_some_and(|value| {
                !matches!(value.node, Some(NodeEnum::MultiAssignRef(_)) | None)
            })
    });
    if !simple {
        return None;
    }

    let (_, earlier) = assignments.split_last()?;
    let mut fix = RuleFix::new(format!("Keep only the last assignment to {column}"));
    for target in earlier {
        // Up to the next assignment, which a later one always is.
        let start = tokens.at(target.location)?;
        let mut comma = tokens.element_end(start) + 1;
        while tokens.kind(comma)? != Token::Ascii44 {
            comma += 1;
        }
        tokens.kind(comma + 1)?;
        fix = fix.delete(TextRange::new(
            tokens.range(start).start(),
            tokens.range(comma + 1).start(),
        ));
    }
    Some(fix)
}

/// What a list of column definitions or names belongs to.
#[derive(Clone, Copy)]
enum Definition {
    /// The columns of a table, view or composite type.
    Relation,
    /// The columns added by an `ALTER TABLE`.
    AddedColumn,
    /// The column list of a `COPY`.
    ColumnList,
}

fn report_definitions(
    columns: impl IntoIterator<Item = Column>,
    definition: Definition,
    diagnostics: &mut Vec<LinterDiagnostic>,
) {
    let (detail, note) = match definition {
        Definition::Relation => (
            "Postgres rejects relations with duplicate column names.",
            "Give each column a distinct name.",
        ),
        Definition::AddedColumn => (
            "Postgres rejects adding a column that the statement already added.",
            "Add the column once.",
        ),
        Definition::ColumnList => (
            "Postgres rejects column lists that name a column more than once.",
            "Name each column once.",
        ),
    };
    for column in duplicates(columns.into_iter().map(|column| (column, true))) {
        let name = &column.name;
        diagnostics.push(
            LinterDiagnostic::new(
                rule_category!(),
                column.span,
                markup! {
                    "Column "<Emphasis>{name}</Emphasis>" is specified more than once."
                },
            )
            .detail(None, detail)
            .note(note),
        );
    }
}

/// The second occurrence of each name that occurs more than once, in order of the first
/// occurrence, given whether each occurrence is to the whole column.
fn duplicates(columns: impl IntoIterator<Item = (Column, bool)>) -> Vec<Column> {
    // The first occurrence, the second one, and whether any occurrence is to the whole column.
    let mut seen: Vec<(Column, Option<Column>, bool)> = Vec::new();
    for (column, whole) in columns {
        match seen
            .iter_mut()
            .find(|(first, ..)| first.name == column.name)
        {
            Some((_, second, any_whole)) => {
                second.get_or_insert(column);
                *any_whole |= whole;
            }
            None => seen.push((column, None, whole)),
        }
    }
    seen.into_iter()
        .filter_map(|(_, second, any_whole)| second.filter(|_| any_whole))
        .collect()
}

fn column_definitions(elements: &[Node], tokens: Option<StatementTokens>) -> Vec<Column> {
    elements
        .iter()
        .filter_map(|element| match &element.node {
            Some(NodeEnum::ColumnDef(column)) => Some(Column {
                name: column.colname.clone(),
                span: tokens.and_then(|tokens| tokens.range_at(column.location)),
            }),
            _ => None,
        })
        .collect()
}

/// The columns added by `ADD COLUMN`. Postgres skips an `ADD COLUMN IF NOT EXISTS` of a column
/// that an earlier command added, so only the other ones count as duplicates.
fn added_columns(stmt: &AlterTableStmt, tokens: Option<StatementTokens>) -> Vec<Column> {
    let mut columns = Vec::new();
    for cmd in alter_table_cmds(stmt) {
        if cmd.subtype() != AlterTableType::AtAddColumn {
            continue;
        }
        let Some(NodeEnum::ColumnDef(column)) = cmd_def(cmd) else {
            continue;
        };
        let added = columns
            .iter()
            .any(|added: &Column| added.name == column.colname);
        if cmd.missing_ok && added {
            continue;
        }
        columns.push(Column {
            name: column.colname.clone(),
            span: tokens.and_then(|tokens| tokens.range_at(column.location)),
        });
    }
    columns
}

/// The columns of a view, `CREATE TABLE AS` or `SELECT INTO`: the explicit names, then the names
/// of the remaining output columns of its query. The columns that a `*` expands to all point at
/// the `*`. Columns whose names are not known are left out.
fn output_columns(names: Vec<Column>, output: Option<&QueryOutput>) -> Vec<Column> {
    let Some(output) = output else {
        return names;
    };
    let explicit = names.len();
    let mut columns = names;
    // The explicit names rename the first output columns. The position of the next column,
    // while we know it.
    let mut position = Some(0);
    for target in &output.targets {
        let Some(entry_columns) = &target.columns else {
            match target.star {
                // An expression is one column, whose name we don't know.
                None => position = position.map(|position| position + 1),
                // If the explicit names reach into a `*` whose columns we don't know, we can't
                // tell which of the entries after it keep their own names.
                Some(Star::Columns(_) | Star::Fields) => {
                    if position.is_some_and(|position| position < explicit) {
                        return columns;
                    }
                    position = None;
                }
            }
            continue;
        };
        for column in entry_columns {
            if !position.is_some_and(|position| position < explicit) {
                columns.push(Column {
                    name: column.name.clone(),
                    span: target.span,
                });
            }
            position = position.map(|position| position + 1);
        }
    }
    columns
}

/// The names of a column list, like `copy t (a, b)` or `create view v (a, b)`, which directly
/// follows the name of the relation at `relation`. The names have no locations, so their spans
/// come from the tokens of the list.
fn column_list(
    names: &[Node],
    relation: Option<&RangeVar>,
    tokens: Option<StatementTokens>,
) -> Vec<Column> {
    let spans = relation
        .zip(tokens)
        .and_then(|(relation, tokens)| list_element_ranges(tokens, relation.location))
        .filter(|spans| spans.len() == names.len());
    names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| match &name.node {
            Some(NodeEnum::String(name)) => Some(Column {
                name: name.sval.clone(),
                span: spans.as_ref().map(|spans| spans[index]),
            }),
            _ => None,
        })
        .collect()
}

/// The ranges of the elements of the first parenthesized list after the token at `location`.
fn list_element_ranges(tokens: StatementTokens, location: i32) -> Option<Vec<TextRange>> {
    let start = tokens.at(location)?;
    let open = (start + 1..)
        .map_while(|index| Some((index, tokens.kind(index)?)))
        .find(|(_, kind)| *kind == Token::Ascii40)?
        .0;
    tokens.list_elements(open + 1, tokens.closing_paren(open)?)
}

#[cfg(test)]
mod tests {
    fn spans(sql: &str) -> Vec<String> {
        crate::spanned_texts("correctness", "banDuplicateColumnAssignments", sql)
    }

    #[test]
    fn points_at_the_second_occurrence() {
        assert_eq!(
            spans("update users set name = 'a', email = 'b', \"name\" = 'c'"),
            ["\"name\""]
        );
        assert_eq!(
            spans("insert into users (id, name, id) values (1, 'a', 2)"),
            ["id"]
        );
        assert_eq!(
            spans("create table users (id bigint, name text, id text)"),
            ["id"]
        );
        assert_eq!(
            spans("alter table users add column email text, add email text"),
            ["email"]
        );
        assert_eq!(
            spans("create view v as select a.id, (b.id) /* id */ from a, b"),
            ["(b.id)"]
        );
        assert_eq!(
            spans("create view v as select 1 as x, coalesce(a, b) as x, 3 from t"),
            ["coalesce(a, b) as x"]
        );
        assert_eq!(
            spans("select 1 as x, 2 as x into t union select 3, 4"),
            ["2 as x"]
        );
    }

    #[test]
    fn columns_of_a_star_point_at_the_star() {
        let tables = "create table a (id int, x int); create table b (id int, y int);";
        assert_eq!(
            spans(&format!(
                "{tables} create view v as select a.*, b.* /* b */ from a, b"
            )),
            ["b.*"]
        );
        assert_eq!(
            spans(&format!(
                "{tables} create view v as select a.*, 1 as x from a"
            )),
            ["1 as x"]
        );
        assert_eq!(
            spans(&format!("{tables} create view v as select * from a, b")),
            ["*"]
        );
    }

    #[test]
    fn points_at_the_second_name_of_a_column_list() {
        assert_eq!(
            spans("copy users (id, name, \"id\") from stdin"),
            ["\"id\""]
        );
        assert_eq!(
            spans("copy public.users (id, id) to stdout with (format csv)"),
            ["id"]
        );
        assert_eq!(
            spans("create view v (a, b, \"a\") with (security_barrier) as select 1, 2, 3"),
            ["\"a\""]
        );
        assert_eq!(
            spans("create table t (x, y, \"x\") as select 1, 2, 3"),
            ["\"x\""]
        );
        // An explicit name and the name of a select list entry.
        assert_eq!(
            spans("create view v (id) as select 1, 2 as id"),
            ["2 as id"]
        );
    }
}
