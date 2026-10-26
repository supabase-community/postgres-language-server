use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::AlterTableType;

declare_lint_rule! {
    /// Dropping the expression of a generated column may break existing clients.
    ///
    /// `DROP EXPRESSION` turns a stored generated column into an ordinary column. Existing values
    /// remain, but Postgres no longer computes the value for new or updated rows, so clients that
    /// omit the column write `NULL` instead of the computed value.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table line_items alter column total drop expression;
    /// ```
    ///
    pub BanDropGeneratedExpression {
        version: "next",
        name: "banDropGeneratedExpression",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-drop-generated-expression")],
    }
}

impl LinterRule for BanDropGeneratedExpression {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let pgls_query::NodeEnum::AlterTableStmt(stmt) = ctx.stmt() else {
            return vec![];
        };

        stmt.cmds
            .iter()
            .filter_map(|cmd| match &cmd.node {
                Some(pgls_query::NodeEnum::AlterTableCmd(cmd))
                    if cmd.subtype() == AlterTableType::AtDropExpression =>
                {
                    Some(cmd)
                }
                _ => None,
            })
            .map(|cmd| {
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Dropping the expression of generated column "<Emphasis>{cmd.name}</Emphasis>" may break existing clients."
                    },
                )
                .detail(
                    None,
                    "Postgres no longer computes the column, so inserts and updates that omit it write NULL.",
                )
                .note("Update clients to write the column themselves before dropping the expression.")
            })
            .collect()
    }
}
