use crate::utils::{alter_table_cmds, altered_relation, cmd_def, column_constraints};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{AlterTableCmd, AlterTableType, ColumnDef, ConstrType};
use pgls_query_ext::utils::string_value;

declare_lint_rule! {
    /// Adding a new column that is NOT NULL and has no default value to an existing table effectively makes it required.
    ///
    /// This will fail immediately upon running for any populated table. Furthermore, old application code that is unaware of this column will fail to INSERT to this table.
    ///
    /// Make new columns optional initially by omitting the NOT NULL constraint until all existing data and application code has been updated. Once no NULL values are written to or persisted in the database, set it to NOT NULL.
    /// Alternatively, if using Postgres version 11 or later, add a DEFAULT value that is not volatile. This allows the column to keep its NOT NULL constraint.
    ///
    /// Generated, identity and `serial` columns get their values from Postgres, so they are not reported.
    ///
    /// ## Invalid
    /// alter table test add column count int not null;
    ///
    /// ## Valid in Postgres >= 11
    /// alter table test add column count int not null default 0;
    pub AddingRequiredField {
        version: "0.1.0",
        name: "addingRequiredField",
        severity: Severity::Error,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("adding-required-field")],
    }
}

impl LinterRule for AddingRequiredField {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        // No other transaction has written to a table created earlier in the transaction.
        if altered_relation(ctx.stmt()).is_some_and(|relation| ctx.is_new_relation(relation)) {
            return vec![];
        }
        let commands: Vec<&AlterTableCmd> = alter_table_cmds(ctx.stmt()).collect();

        // Columns made NOT NULL by `ADD CONSTRAINT ... NOT NULL column` (Postgres 18) in the
        // same statement.
        let not_null_columns: Vec<&str> = commands
            .iter()
            .filter(|command| command.subtype() == AlterTableType::AtAddConstraint)
            .filter_map(|command| match cmd_def(command)? {
                pgls_query::NodeEnum::Constraint(constraint)
                    if constraint.contype() == ConstrType::ConstrNotnull =>
                {
                    Some(constraint)
                }
                _ => None,
            })
            .flat_map(|constraint| &constraint.keys)
            .filter_map(string_value)
            .collect();

        commands
            .iter()
            .filter(|command| command.subtype() == AlterTableType::AtAddColumn)
            .filter_map(|command| match cmd_def(command)? {
                pgls_query::NodeEnum::ColumnDef(column) => Some(column),
                _ => None,
            })
            .filter(|column| is_required(column, &not_null_columns))
            .map(|_| {
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Adding a new column that is NOT NULL and has no default value to an existing table effectively makes it required."
                    },
                )
                .detail(
                    None,
                    "Make new columns optional initially by omitting the NOT NULL constraint until all existing data and application code has been updated. Once no NULL values are written to or persisted in the database, set it to NOT NULL. Alternatively, if using Postgres version 11 or later, add a DEFAULT value that is not volatile. This allows the column to keep its NOT NULL constraint.",
                )
            })
            .collect()
    }
}

/// Whether a new column is NOT NULL and gets no value from Postgres: no default, not generated,
/// not an identity column, and not `serial`.
fn is_required(column: &ColumnDef, not_null_columns: &[&str]) -> bool {
    let constraint_types: Vec<ConstrType> = column_constraints(column)
        .map(|constraint| constraint.contype())
        .collect();
    let is_not_null = constraint_types.contains(&ConstrType::ConstrNotnull)
        || not_null_columns.contains(&column.colname.as_str());
    let has_value = constraint_types.iter().any(|contype| {
        matches!(
            contype,
            ConstrType::ConstrDefault | ConstrType::ConstrGenerated | ConstrType::ConstrIdentity
        )
    }) || column
        .type_name
        .as_ref()
        .is_some_and(|type_name| pgls_catalog::serial_type(type_name).is_some());
    is_not_null && !has_value
}
