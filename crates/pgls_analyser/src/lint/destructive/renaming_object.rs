use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, utils::object_type_noun};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::ObjectType;

declare_lint_rule! {
    /// Renaming an object or an enum value may break existing clients.
    ///
    /// Clients that still use the old name of a view, function, type, sequence, schema,
    /// constraint, role, or any other object fail after the rename. The same goes for enum values
    /// renamed with `ALTER TYPE ... RENAME VALUE`.
    ///
    /// Tables and columns are covered by `renamingTable` and `renamingColumn`.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter view active_users rename to enabled_users;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter type order_status rename value 'shipped' to 'dispatched';
    /// ```
    ///
    pub RenamingObject {
        version: "next",
        name: "renamingObject",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("renaming-object")],
    }
}

impl LinterRule for RenamingObject {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let object = match ctx.stmt() {
            pgls_query::NodeEnum::RenameStmt(stmt)
                if !matches!(
                    stmt.rename_type(),
                    ObjectType::ObjectTable | ObjectType::ObjectColumn
                ) =>
            {
                object_type_noun(stmt.rename_type())
            }
            pgls_query::NodeEnum::AlterEnumStmt(stmt) if !stmt.old_val.is_empty() => {
                "an enum value"
            }
            _ => return vec![],
        };

        vec![
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Renaming "{object}" may break existing clients."
                },
            )
            .detail(
                None,
                "Clients that still use the old name fail after the rename.",
            )
            .note("Update clients to use the new name before renaming."),
        ]
    }
}
