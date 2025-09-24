use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Adding a new column that is NOT NULL and has no default value to an existing table effectively makes it required.
    ///
    /// This will fail immediately upon running for any populated table. Furthermore, old application code that is unaware of this column will fail to INSERT to this table.
    ///
    /// Make new columns optional initially by omitting the NOT NULL constraint until all existing data and application code has been updated. Once no NULL values are written to or persisted in the database, set it to NOT NULL.
    /// Alternatively, if using Postgres version 11 or later, add a DEFAULT value that is not volatile. This allows the column to keep its NOT NULL constraint.
    ///
    /// ## Invalid
    /// alter table test add column count int not null;
    ///
    /// ## Valid in Postgres >= 11
    /// alter table test add column count int not null default 0;
    pub AddingRequiredField {
        version: "next",
        name: "addingRequiredField",
        severity: Severity::Error,
        recommended: false,
        sources: &[RuleSource::Squawk("adding-required-field")],
    }
}

impl Rule for AddingRequiredField {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = vec![];

        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = ctx.stmt() {
            // We are currently lacking a way to check if a `AtAddColumn` subtype sets a
            // not null constraint – so we'll need to check the plain SQL.
            let plain_sql = ctx.stmt().to_ref().deparse().unwrap().to_ascii_lowercase();
            let is_nullable = !plain_sql.contains("not null");
            let has_set_default = plain_sql.contains("default");
            if is_nullable || has_set_default {
                return diagnostics;
            }

            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(alter_table_cmd)) = &cmd.node {
                    if alter_table_cmd.subtype()
                        == pgls_query::protobuf::AlterTableType::AtAddColumn
                    {
                        diagnostics.push(
                            RuleDiagnostic::new(
                                rule_category!(),
                                None,
                                markup! {
                                    "Adding a new column that is NOT NULL and has no default value to an existing table effectively makes it required."
                                },
                            )
                            .detail(
                                None,
                                "Make new columns optional initially by omitting the NOT NULL constraint until all existing data and application code has been updated. Once no NULL values are written to or persisted in the database, set it to NOT NULL. Alternatively, if using Postgres version 11 or later, add a DEFAULT value that is not volatile. This allows the column to keep its NOT NULL constraint.
                                ",
                        ),
                    );
                    }
                }
            }
        }

        diagnostics
    }
}
