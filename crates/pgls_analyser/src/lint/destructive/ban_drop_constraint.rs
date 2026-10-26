use crate::utils::alter_table_cmds;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{AlterTableType, Token};

declare_lint_rule! {
    /// Dropping a constraint may break existing clients.
    ///
    /// A constraint is a guarantee that clients can depend on. For example, an application that
    /// uses `INSERT ... ON CONFLICT ON CONSTRAINT my_constraint`, or whose `ON CONFLICT (col)`
    /// relies on a unique constraint, fails as soon as the constraint is gone.
    ///
    /// Update clients to no longer depend on the constraint before dropping it.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table users drop constraint users_email_key;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter domain positive_int drop constraint positive_int_check;
    /// ```
    ///
    pub BanDropConstraint {
        version: "next",
        name: "banDropConstraint",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-drop-constraint")],
    }
}

impl LinterRule for BanDropConstraint {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let constraints: Vec<&str> = match ctx.stmt() {
            pgls_query::NodeEnum::AlterTableStmt(_) => alter_table_cmds(ctx.stmt())
                .filter(|cmd| cmd.subtype() == AlterTableType::AtDropConstraint)
                .map(|cmd| cmd.name.as_str())
                .collect(),
            // 'X' is `DROP CONSTRAINT`.
            pgls_query::NodeEnum::AlterDomainStmt(stmt) if stmt.subtype == "X" => {
                vec![stmt.name.as_str()]
            }
            _ => return vec![],
        };

        // Each command has a `DROP CONSTRAINT`, in the order of the commands.
        let mut spans = ctx
            .tokens()
            .map(|tokens| tokens.keyword_pairs(Token::Drop, Token::Constraint))
            .unwrap_or_default()
            .into_iter();
        constraints
            .into_iter()
            .map(|constraint| {
                LinterDiagnostic::new(
                    rule_category!(),
                    spans.next(),
                    markup! {
                        "Dropping constraint "<Emphasis>{constraint}</Emphasis>" may break existing clients."
                    },
                )
                .detail(
                    None,
                    "Clients may depend on the guarantee of the constraint, e.g. as the arbiter of INSERT ... ON CONFLICT.",
                )
                .note("Update clients to no longer depend on the constraint, then drop it in a later migration.")
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn points_at_each_drop() {
        assert_eq!(
            crate::spanned_texts(
                "destructive",
                "banDropConstraint",
                "alter table users drop constraint a, add check (b > 0), drop  constraint if exists c"
            ),
            ["drop constraint", "drop  constraint"]
        );
    }
}
