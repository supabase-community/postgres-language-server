use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_catalog::resolve::figure_column_name;
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{Node, SelectStmt, SetOperation};
use pgls_query::{NodeEnum, NodeRef};

declare_lint_rule! {
    /// A column is assigned or declared more than once.
    ///
    /// Postgres rejects statements that assign the same column twice, like
    /// `update t set a = 1, a = 2` or `insert into t (a, a)`, with `42601 syntax_error`. It
    /// rejects tables and views that declare the same column twice with `42701
    /// duplicate_column`. This includes view columns that get the same name from their
    /// expressions, like `select a.id, b.id`.
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
    }
}

impl LinterRule for BanDuplicateColumnAssignments {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        for node in ctx.stmt().iter() {
            match node {
                NodeRef::UpdateStmt(stmt) => {
                    report_assignments(&stmt.target_list, &mut diagnostics)
                }
                NodeRef::OnConflictClause(clause) => {
                    report_assignments(&clause.target_list, &mut diagnostics)
                }
                NodeRef::MergeWhenClause(clause) => {
                    report_assignments(&clause.target_list, &mut diagnostics)
                }
                NodeRef::InsertStmt(stmt) => report_assignments(&stmt.cols, &mut diagnostics),
                NodeRef::CreateStmt(stmt) => {
                    let columns =
                        stmt.table_elts
                            .iter()
                            .filter_map(|element| match &element.node {
                                Some(NodeEnum::ColumnDef(column)) => Some(column.colname.clone()),
                                _ => None,
                            });
                    report_definitions(columns, &mut diagnostics);
                }
                NodeRef::ViewStmt(stmt) => report_definitions(
                    output_columns(&stmt.aliases, stmt.query.as_deref()),
                    &mut diagnostics,
                ),
                NodeRef::CreateTableAsStmt(stmt) => {
                    let names = stmt.into.as_ref().map_or(&[][..], |into| &into.col_names);
                    report_definitions(
                        output_columns(names, stmt.query.as_deref()),
                        &mut diagnostics,
                    );
                }
                NodeRef::SelectStmt(stmt) if stmt.into_clause.is_some() => {
                    report_definitions(select_columns(&[], stmt), &mut diagnostics)
                }
                _ => {}
            }
        }

        diagnostics
    }
}

/// Reports the columns of a SET clause or an INSERT column list that are assigned more than
/// once. Assigning distinct subscripts or fields of a column is allowed, so a column is only
/// reported if one of its assignments is to the whole column.
fn report_assignments(targets: &[Node], diagnostics: &mut Vec<LinterDiagnostic>) {
    let assignments = targets.iter().filter_map(|target| match &target.node {
        Some(NodeEnum::ResTarget(target)) => {
            Some((target.name.as_str(), target.indirection.is_empty()))
        }
        _ => None,
    });

    for column in duplicates(assignments) {
        diagnostics.push(
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Multiple assignments to column "<Emphasis>{column}</Emphasis>"."
                },
            )
            .detail(
                None,
                "Postgres rejects statements that assign a column more than once.",
            )
            .note("Remove all but one of the assignments."),
        );
    }
}

fn report_definitions(
    columns: impl IntoIterator<Item = String>,
    diagnostics: &mut Vec<LinterDiagnostic>,
) {
    let columns: Vec<String> = columns.into_iter().collect();
    for column in duplicates(columns.iter().map(|column| (column.as_str(), true))) {
        diagnostics.push(
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Column "<Emphasis>{column}</Emphasis>" is specified more than once."
                },
            )
            .detail(
                None,
                "Postgres rejects relations with duplicate column names.",
            )
            .note("Give each column a distinct name."),
        );
    }
}

/// The names that occur more than once, in order of their first occurrence, given whether each
/// occurrence is to the whole column.
fn duplicates<'a>(names: impl IntoIterator<Item = (&'a str, bool)>) -> Vec<&'a str> {
    let mut seen: Vec<(&str, usize, bool)> = Vec::new();
    for (name, whole) in names {
        match seen.iter_mut().find(|(seen, ..)| *seen == name) {
            Some((_, count, any_whole)) => {
                *count += 1;
                *any_whole |= whole;
            }
            None => seen.push((name, 1, whole)),
        }
    }
    seen.into_iter()
        .filter(|(_, count, any_whole)| *count > 1 && *any_whole)
        .map(|(name, ..)| name)
        .collect()
}

/// The column names of a view or `CREATE TABLE AS`: the explicit names, then the names of the
/// remaining entries of the select list.
fn output_columns(names: &[Node], query: Option<&Node>) -> Vec<String> {
    match query.and_then(|query| query.node.as_ref()) {
        Some(NodeEnum::SelectStmt(select)) => select_columns(names, select),
        _ => string_values(names),
    }
}

/// Columns we can't name, like the expansion of `*`, are left out.
fn select_columns(names: &[Node], select: &SelectStmt) -> Vec<String> {
    let mut select = select;
    // The left-most select of a set operation names the columns.
    while select.op() != SetOperation::SetopNone {
        match select.larg.as_deref() {
            Some(larg) => select = larg,
            None => return string_values(names),
        }
    }

    let mut columns = string_values(names);
    let targets: Vec<_> = select
        .target_list
        .iter()
        .filter_map(|target| match &target.node {
            Some(NodeEnum::ResTarget(target)) => Some(target),
            _ => None,
        })
        .collect();
    // Explicit names apply to the expansion of a `*` too, so with a `*` we can't tell which
    // entries keep their own names.
    let has_star = targets.iter().any(|target| {
        matches!(
            target.val.as_deref().and_then(|value| value.node.as_ref()),
            Some(NodeEnum::ColumnRef(column))
                if matches!(column.fields.last().and_then(|field| field.node.as_ref()), Some(NodeEnum::AStar(_)))
        )
    });
    if has_star || !select.values_lists.is_empty() {
        return columns;
    }

    columns.extend(targets.iter().skip(names.len()).filter_map(|target| {
        if target.name.is_empty() {
            figure_column_name(target.val.as_deref()?.node.as_ref()?)
        } else {
            Some(target.name.clone())
        }
    }));
    columns
}

fn string_values(names: &[Node]) -> Vec<String> {
    names
        .iter()
        .filter_map(|name| match &name.node {
            Some(NodeEnum::String(name)) => Some(name.sval.clone()),
            _ => None,
        })
        .collect()
}
