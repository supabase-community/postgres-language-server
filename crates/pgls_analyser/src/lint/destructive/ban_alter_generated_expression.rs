use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::NodeEnum;
use pgls_query::protobuf::{AlterTableType, ConstrType, Constraint};

declare_lint_rule! {
    /// Adding a generated column or changing its expression may break existing clients.
    ///
    /// `SET EXPRESSION` changes the values that existing clients read from the column. A new
    /// generated column changes the columns of `SELECT *`, and statements that copy rows with
    /// `INSERT ... SELECT *` then try to write the generated column, which fails.
    ///
    /// Dropping the expression is reported by `banDropGeneratedExpression`. `SET EXPRESSION` is
    /// only checked on Postgres 17 and later, which added it.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table line_items alter column total set expression as (price * quantity);
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter table line_items add column total numeric generated always as (price * quantity) stored;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter table line_items add column total numeric;
    /// ```
    ///
    pub BanAlterGeneratedExpression {
        version: "next",
        name: "banAlterGeneratedExpression",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-alter-generated-expression")],
    }
}

impl LinterRule for BanAlterGeneratedExpression {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = vec![];

        for cmd in ctx.stmt().alter_table_commands() {
            match cmd.subtype() {
                AlterTableType::AtSetExpression
                    if ctx.catalog().server_version_at_least(17) != Some(false) =>
                {
                    diagnostics.push(
                        LinterDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Changing the expression of generated column "<Emphasis>{cmd.name}</Emphasis>" may break existing clients."
                            },
                        )
                        .detail(None, "Existing clients read different values from the column.")
                        .note("Check that clients can handle the new values before changing the expression."),
                    );
                }
                AlterTableType::AtAddColumn => {
                    let Some(NodeEnum::ColumnDef(col_def)) = cmd.definition() else {
                        continue;
                    };
                    if Constraint::in_list(&col_def.constraints)
                        .any(|constraint| constraint.contype() == ConstrType::ConstrGenerated)
                    {
                        diagnostics.push(
                            LinterDiagnostic::new(
                                rule_category!(),
                                None,
                                markup! {
                                    "Adding generated column "<Emphasis>{col_def.colname}</Emphasis>" may break existing clients."
                                },
                            )
                            .detail(
                                None,
                                "SELECT * returns an additional column, and INSERT ... SELECT * tries to write the generated column, which fails.",
                            )
                            .note("Make sure that clients list the columns they insert and select before adding the column."),
                        );
                    }
                }
                _ => {}
            }
        }

        diagnostics
    }
}
