use crate::fix::StatementTokens;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_catalog::resolve::{figure_column_name, query_output_columns};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{AlterTableType, Node, ResTarget, SelectStmt, SetOperation, Token};
use pgls_query::{NodeEnum, NodeRef};
use pgls_text_size::TextRange;
use std::cell::OnceCell;

declare_lint_rule! {
    /// A column is assigned or declared more than once.
    ///
    /// Postgres rejects statements that assign the same column twice in a SET clause, like
    /// `update t set a = 1, a = 2`, with `42601 syntax_error`. It rejects column lists that name
    /// the same column twice, like `insert into t (a, a)` or `copy t (a, a)`, tables, views and
    /// composite types that declare the same column twice, and `ALTER TABLE` commands that add
    /// the same column twice with `42701 duplicate_column`. This includes view columns that get
    /// the same name from their expressions, like `select a.id, b.id`.
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

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let tokens = ctx.tokens();
        let mut diagnostics = Vec::new();
        // The output column names of a query, as far as the catalog knows them.
        let catalog_names =
            |query: &Node| query_output_columns(query, ctx.catalog(), ctx.session().search_path());

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
                    string_values(&stmt.attlist),
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
                    added_columns(&stmt.cmds, tokens),
                    Definition::AddedColumn,
                    &mut diagnostics,
                ),
                NodeRef::ViewStmt(stmt) => report_definitions(
                    output_columns(&stmt.aliases, stmt.query.as_deref(), tokens, &catalog_names),
                    Definition::Relation,
                    &mut diagnostics,
                ),
                NodeRef::CreateTableAsStmt(stmt) => {
                    let names = stmt.into.as_ref().map_or(&[][..], |into| &into.col_names);
                    report_definitions(
                        output_columns(names, stmt.query.as_deref(), tokens, &catalog_names),
                        Definition::Relation,
                        &mut diagnostics,
                    );
                }
                NodeRef::SelectStmt(stmt) if stmt.into_clause.is_some() => report_definitions(
                    select_columns(&[], stmt, tokens, &|| {
                        catalog_names(&Node {
                            node: Some(NodeEnum::SelectStmt(Box::new(stmt.clone()))),
                        })
                    }),
                    Definition::Relation,
                    &mut diagnostics,
                ),
                _ => {}
            }
        }

        diagnostics
    }
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
                span: token_range(tokens, target.location),
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
                span: token_range(tokens, column.location),
            }),
            _ => None,
        })
        .collect()
}

/// The columns added by `ADD COLUMN`. Postgres skips an `ADD COLUMN IF NOT EXISTS` of a column
/// that an earlier command added, so only the other ones count as duplicates.
fn added_columns(cmds: &[Node], tokens: Option<StatementTokens>) -> Vec<Column> {
    let mut columns = Vec::new();
    for cmd in cmds {
        let Some(NodeEnum::AlterTableCmd(cmd)) = &cmd.node else {
            continue;
        };
        if cmd.subtype() != AlterTableType::AtAddColumn {
            continue;
        }
        let Some(NodeEnum::ColumnDef(column)) =
            cmd.def.as_deref().and_then(|def| def.node.as_ref())
        else {
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
            span: token_range(tokens, column.location),
        });
    }
    columns
}

/// The column names of a view or `CREATE TABLE AS`: the explicit names, then the names of the
/// remaining entries of the select list.
fn output_columns(
    names: &[Node],
    query: Option<&Node>,
    tokens: Option<StatementTokens>,
    catalog_names: &dyn Fn(&Node) -> Option<Vec<String>>,
) -> Vec<Column> {
    match query.and_then(|query| query.node.as_ref().map(|node| (query, node))) {
        Some((query, NodeEnum::SelectStmt(select))) => {
            select_columns(names, select, tokens, &|| catalog_names(query))
        }
        _ => string_values(names),
    }
}

/// Columns we can't name, like the expansion of `*`, are left out. `catalog_names` gives the
/// output column names of the query that the catalog knows, for the entries whose name depends
/// on the catalog, like `(select * from t)`, which is named after the first column of `t`.
fn select_columns(
    names: &[Node],
    select: &SelectStmt,
    tokens: Option<StatementTokens>,
    catalog_names: &dyn Fn() -> Option<Vec<String>>,
) -> Vec<Column> {
    let mut select = select;
    // The left-most select of a set operation names the columns.
    while select.op() != SetOperation::SetopNone {
        match select.larg.as_deref() {
            Some(larg) => select = larg,
            None => return string_values(names),
        }
    }

    let mut columns = string_values(names);
    if !select.values_lists.is_empty() {
        return columns;
    }
    let targets: Vec<_> = select
        .target_list
        .iter()
        .filter_map(|target| match &target.node {
            Some(NodeEnum::ResTarget(target)) => Some(target),
            _ => None,
        })
        .collect();
    // Explicit names apply to the entries in order, and a `*` expands to an unknown number of
    // them. If the explicit names reach into a `*`, we can't tell which of the entries after it
    // keep their own names.
    let first_star = targets.iter().position(|target| {
        target
            .val
            .as_deref()
            .and_then(|value| value.node.as_ref())
            .is_some_and(is_star)
    });
    if first_star.is_some_and(|star| star < names.len()) {
        return columns;
    }

    let known_names = OnceCell::new();
    // The output columns before the first `*` are the entries of the select list.
    let columns_before_star = first_star.unwrap_or(targets.len());
    let name = |index: usize, target: &ResTarget| -> Option<String> {
        if !target.name.is_empty() {
            return Some(target.name.clone());
        }
        if let Some(name) = figure_column_name(target.val.as_deref()?.node.as_ref()?) {
            return Some(name);
        }
        if index >= columns_before_star {
            return None;
        }
        known_names
            .get_or_init(catalog_names)
            .as_ref()?
            .get(index)
            .cloned()
    };
    columns.extend(
        targets
            .iter()
            .enumerate()
            .skip(names.len())
            .filter_map(|(index, target)| {
                Some(Column {
                    name: name(index, target)?,
                    span: select_entry_range(tokens, target.location),
                })
            }),
    );
    columns
}

fn is_star(value: &NodeEnum) -> bool {
    let last = match value {
        NodeEnum::ColumnRef(column) => column.fields.last(),
        NodeEnum::AIndirection(indirection) => indirection.indirection.last(),
        _ => None,
    };
    matches!(
        last.and_then(|last| last.node.as_ref()),
        Some(NodeEnum::AStar(_))
    )
}

fn string_values(names: &[Node]) -> Vec<Column> {
    names
        .iter()
        .filter_map(|name| match &name.node {
            Some(NodeEnum::String(name)) => Some(Column {
                name: name.sval.clone(),
                span: None,
            }),
            _ => None,
        })
        .collect()
}

/// The range of the token at `location`, e.g. the name of a column definition.
fn token_range(tokens: Option<StatementTokens>, location: i32) -> Option<TextRange> {
    let tokens = tokens?;
    Some(tokens.range(tokens.at(location)?))
}

/// The range of the select list entry that starts at `location`: up to the next `,` outside
/// parentheses, or the keyword that ends the select list.
fn select_entry_range(tokens: Option<StatementTokens>, location: i32) -> Option<TextRange> {
    let tokens = tokens?;
    let start = tokens.at(location)?;
    let mut depth = 0usize;
    let mut last = start;
    let mut index = start;
    while let Some(kind) = tokens.kind(index) {
        match kind {
            Token::Ascii40 => depth += 1,
            Token::Ascii41 if depth == 0 => break,
            Token::Ascii41 => depth -= 1,
            Token::Ascii44
            | Token::Ascii59
            | Token::From
            | Token::Into
            | Token::Where
            | Token::GroupP
            | Token::Having
            | Token::Window
            | Token::Union
            | Token::Intersect
            | Token::Except
            | Token::Order
            | Token::Limit
            | Token::Offset
            | Token::Fetch
            | Token::For
                if depth == 0 =>
            {
                break;
            }
            _ => {}
        }
        if !matches!(kind, Token::SqlComment | Token::CComment) {
            last = index;
        }
        index += 1;
    }
    Some(tokens.range(start).cover(tokens.range(last)))
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
    fn statement_without_locations() {
        let sql = "copy users (id, id) from stdin";
        assert_eq!(spans(sql), [sql]);
    }
}
