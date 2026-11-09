use crate::linter_context::builtin_type_name;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_catalog::Generated;
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Changing a column type may require a table rewrite and break existing clients.
    ///
    /// Most column type changes require an exclusive lock on the table while the entire
    /// table is rewritten. This can take a long time for large tables and will block
    /// reads and writes.
    ///
    /// Some type changes are safe and don't require a table rewrite:
    /// - Changing to `text` (binary compatible with varchar/char types)
    /// - Changing to `varchar` without a length limit
    /// - Dropping a `numeric` precision constraint (e.g., `numeric(10,2)` to `numeric`)
    ///
    /// Changing the type of a virtual generated column (Postgres 18) doesn't rewrite the table
    /// either: its values are computed when rows are read.
    ///
    /// For unsafe type changes, consider creating a new column with the desired type,
    /// migrating the data, and then dropping the old column.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE "core_recipe" ALTER COLUMN "count" TYPE bigint;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// ALTER TABLE "core_recipe" ALTER COLUMN "edits" TYPE text;
    /// ```
    ///
    pub ChangingColumnType {
        version: "0.15.0",
        name: "changingColumnType",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("changing-column-type")],
    }
}

impl LinterRule for ChangingColumnType {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node
                    && cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAlterColumnType
                {
                    if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                        cmd.def.as_ref().and_then(|d| d.node.as_ref())
                        && is_safe_type_widening(col_def)
                    {
                        continue;
                    }
                    if is_virtual_column(ctx, stmt.relation.as_ref(), &cmd.name) {
                        continue;
                    }

                    diagnostics.push(LinterDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Changing a column type requires a table rewrite and blocks reads and writes."
                            }
                        ).detail(None, "Consider creating a new column with the desired type, migrating data, and then dropping the old column."));
                }
            }
        }

        diagnostics
    }
}

/// Whether the column is a virtual generated column, which [`ATExecAlterColumnType`] doesn't
/// rewrite.
///
/// [`ATExecAlterColumnType`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/tablecmds.c#L14747
fn is_virtual_column(
    ctx: &LinterRuleContext<ChangingColumnType>,
    relation: Option<&pgls_query::protobuf::RangeVar>,
    column: &str,
) -> bool {
    let Some(relation) = relation else {
        return false;
    };
    ctx.catalog()
        .relation_of(relation, ctx.session().search_path())
        .found()
        .and_then(|relation| relation.columns)
        .is_some_and(|columns| {
            columns.iter().any(|candidate| {
                candidate.name == column && candidate.generated == Generated::Virtual
            })
        })
}

fn is_safe_type_widening(col_def: &pgls_query::protobuf::ColumnDef) -> bool {
    let Some(type_name) = &col_def.type_name else {
        return false;
    };

    // An array of the type is a different type.
    let Some(target_type) =
        builtin_type_name(type_name).filter(|_| type_name.array_bounds.is_empty())
    else {
        return false;
    };

    let has_type_modifier = !type_name.typmods.is_empty();

    match target_type {
        // text is always safe — binary compatible with varchar/char
        "text" => true,
        // varchar without length is safe (dropping a length constraint)
        "varchar" if !has_type_modifier => true,
        // numeric (or decimal, which the parser turns into numeric) without precision is safe
        // (dropping precision constraint)
        "numeric" if !has_type_modifier => true,
        _ => false,
    }
}
