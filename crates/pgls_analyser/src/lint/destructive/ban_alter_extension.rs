use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::NodeEnum;

declare_lint_rule! {
    /// Updating an extension or removing an object from it may break existing clients.
    ///
    /// `ALTER EXTENSION ... UPDATE` runs the upgrade scripts of the extension, which may remove
    /// functions, change their signatures, or change their results. `ALTER EXTENSION ... DROP`
    /// removes an object from the extension, so later updates of the extension no longer manage
    /// it.
    ///
    /// Adding an object to an extension is not reported. Moving an extension to another schema
    /// is reported by `banSetSchema`.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter extension postgis update to '3.4.0';
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// alter extension postgis drop function legacy_distance(geometry, geometry);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// alter extension postgis add function legacy_distance(geometry, geometry);
    /// ```
    ///
    pub BanAlterExtension {
        version: "next",
        name: "banAlterExtension",
        severity: Severity::Warning,
        recommended: false,
        applies_to: pgls_analyse::AppliesTo::Migration,
        sources: &[RuleSource::Squawk("ban-alter-extension")],
    }
}

impl LinterRule for BanAlterExtension {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let diagnostic = match ctx.stmt() {
            NodeEnum::AlterExtensionStmt(stmt) => LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Updating extension "<Emphasis>{stmt.extname}</Emphasis>" may break existing clients."
                },
            )
            .detail(
                None,
                "The upgrade scripts of the extension may remove functions, change their signatures, or change their results.",
            )
            .note("Read the upgrade notes of the extension and update clients before updating it."),
            // `action` is -1 for `DROP` and 1 for `ADD`.
            NodeEnum::AlterExtensionContentsStmt(stmt) if stmt.action < 0 => LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Removing an object from extension "<Emphasis>{stmt.extname}</Emphasis>" may break existing clients."
                },
            )
            .detail(
                None,
                "Later updates of the extension no longer manage the object, so it keeps its old definition.",
            )
            .note("Make sure that nothing relies on the extension managing the object before removing it."),
            _ => return vec![],
        };

        vec![diagnostic]
    }
}
