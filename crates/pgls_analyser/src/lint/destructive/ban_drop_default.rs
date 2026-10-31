use crate::utils::alter_table_cmds;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{AlterTableType, Token};

declare_lint_rule! {
    /// Dropping a column or domain default may break existing clients.
    ///
    /// Inserts that omit the column fail with `23502 not_null_violation` if the column is
    /// `NOT NULL`, and silently write `NULL` otherwise.
    ///
    /// Update your application code to provide a value for the column, then drop the default in a
    /// later migration.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table users alter column created_at drop default;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter domain status drop default;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter table users alter column created_at set default now();
    /// ```
    ///
    pub BanDropDefault {
        version: "next",
        name: "banDropDefault",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-drop-default")],
    }
}

impl LinterRule for BanDropDefault {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let defaults: Vec<(&str, &str)> = match ctx.stmt() {
            pgls_query::NodeEnum::AlterTableStmt(_) => alter_table_cmds(ctx.stmt())
                .filter(|cmd| cmd.subtype() == AlterTableType::AtColumnDefault && cmd.def.is_none())
                .map(|cmd| ("column", cmd.name.as_str()))
                .collect(),
            // 'T' is `SET DEFAULT` or, without a default, `DROP DEFAULT`.
            pgls_query::NodeEnum::AlterDomainStmt(stmt)
                if stmt.subtype == "T" && stmt.def.is_none() =>
            {
                let name = stmt.type_name.last().and_then(|name| match &name.node {
                    Some(pgls_query::NodeEnum::String(name)) => Some(name.sval.as_str()),
                    _ => None,
                });
                vec![("domain", name.unwrap_or_default())]
            }
            _ => return vec![],
        };

        // Each command has a `DROP DEFAULT`, in the order of the commands.
        let mut spans = ctx
            .tokens()
            .map(|tokens| tokens.keyword_pairs(Token::Drop, Token::Default))
            .unwrap_or_default()
            .into_iter();
        defaults
            .into_iter()
            .map(|(kind, name)| {
                LinterDiagnostic::new(
                    rule_category!(),
                    spans.next(),
                    markup! {
                        "Dropping the default of "{kind}" "<Emphasis>{name}</Emphasis>" may break existing clients."
                    },
                )
                .detail(
                    None,
                    "Inserts that omit the column fail if it is NOT NULL, and write NULL otherwise.",
                )
                .note("Update your application code to provide a value, then drop the default in a later migration.")
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
                "banDropDefault",
                "alter table users alter a drop default, alter b set default 1, alter c drop  default"
            ),
            ["drop default", "drop  default"]
        );
    }
}
