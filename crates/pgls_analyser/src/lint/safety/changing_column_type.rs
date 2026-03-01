use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
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
        version: "next",
        name: "changingColumnType",
        severity: Severity::Warning,
        recommended: false,
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
                    {
                        if is_safe_type_widening(col_def) {
                            continue;
                        }
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

fn is_safe_type_widening(col_def: &pgls_query::protobuf::ColumnDef) -> bool {
    let Some(type_name) = &col_def.type_name else {
        return false;
    };

    let target_type = type_name
        .names
        .iter()
        .filter_map(|n| {
            if let Some(pgls_query::NodeEnum::String(s)) = &n.node {
                Some(s.sval.as_str())
            } else {
                None
            }
        })
        .last();

    let Some(target_type) = target_type else {
        return false;
    };

    let has_type_modifier = !type_name.typmods.is_empty();

    match target_type.to_lowercase().as_str() {
        // text is always safe — binary compatible with varchar/char
        "text" => true,
        // varchar without length is safe (dropping a length constraint)
        "varchar" if !has_type_modifier => true,
        // numeric without precision is safe (dropping precision constraint)
        "numeric" | "decimal" if !has_type_modifier => true,
        _ => false,
    }
}
