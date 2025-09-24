use pgls_analyse::{Rule, RuleDiagnostic, RuleSource, context::RuleContext, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Prefer using TEXT over VARCHAR(n) types.
    ///
    /// Changing the size of a VARCHAR field requires an ACCESS EXCLUSIVE lock, which blocks all
    /// reads and writes to the table. It's easier to update a check constraint on a TEXT field
    /// than a VARCHAR() size since the check constraint can use NOT VALID with a separate
    /// VALIDATE call.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE "core_bar" (
    ///     "id" serial NOT NULL PRIMARY KEY,
    ///     "alpha" varchar(100) NOT NULL
    /// );
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE "core_bar" ALTER COLUMN "kind" TYPE varchar(1000) USING "kind"::varchar(1000);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE TABLE "core_bar" (
    ///     "id" serial NOT NULL PRIMARY KEY,
    ///     "bravo" text NOT NULL
    /// );
    /// ALTER TABLE "core_bar" ADD CONSTRAINT "text_size" CHECK (LENGTH("bravo") <= 100);
    /// ```
    ///
    pub PreferTextField {
        version: "next",
        name: "preferTextField",
        severity: Severity::Warning,
        recommended: false,
        sources: &[RuleSource::Squawk("prefer-text-field")],
    }
}

impl Rule for PreferTextField {
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        let mut diagnostics = Vec::new();

        match &ctx.stmt() {
            pgls_query::NodeEnum::CreateStmt(stmt) => {
                for table_elt in &stmt.table_elts {
                    if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) = &table_elt.node {
                        check_column_def(&mut diagnostics, col_def);
                    }
                }
            }
            pgls_query::NodeEnum::AlterTableStmt(stmt) => {
                for cmd in &stmt.cmds {
                    if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                        match cmd.subtype() {
                            pgls_query::protobuf::AlterTableType::AtAddColumn
                            | pgls_query::protobuf::AlterTableType::AtAlterColumnType => {
                                if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                                    &cmd.def.as_ref().and_then(|d| d.node.as_ref())
                                {
                                    check_column_def(&mut diagnostics, col_def);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        diagnostics
    }
}

fn check_column_def(
    diagnostics: &mut Vec<RuleDiagnostic>,
    col_def: &pgls_query::protobuf::ColumnDef,
) {
    if let Some(type_name) = &col_def.type_name {
        for name_node in &type_name.names {
            if let Some(pgls_query::NodeEnum::String(name)) = &name_node.node {
                // Check if it's varchar with a size limit
                if name.sval.to_lowercase() == "varchar" && !type_name.typmods.is_empty() {
                    diagnostics.push(
                        RuleDiagnostic::new(
                            rule_category!(),
                            None,
                            markup! {
                                "Changing the size of a varchar field requires an ACCESS EXCLUSIVE lock."
                            },
                        )
                        .note("Use a text field with a check constraint."),
                    );
                }
            }
        }
    }
}
