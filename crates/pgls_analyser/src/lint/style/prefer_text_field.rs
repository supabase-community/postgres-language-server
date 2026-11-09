use crate::fix::{StatementTokens, in_case_of};
use crate::utils::{builtin_type_name, column_defs};
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
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
        version: "0.15.0",
        name: "preferTextField",
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
        recommended: false,
        sources: &[RuleSource::Squawk("prefer-text-field")],
    }
}

impl LinterRule for PreferTextField {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();
        for col_def in column_defs(ctx.stmt()) {
            check_column_def(&mut diagnostics, col_def, ctx.tokens());
        }
        diagnostics
    }
}

fn check_column_def(
    diagnostics: &mut Vec<LinterDiagnostic>,
    col_def: &pgls_query::protobuf::ColumnDef,
    tokens: Option<StatementTokens>,
) {
    // Check if it's varchar with a size limit
    if let Some(type_name) = &col_def.type_name
        && builtin_type_name(type_name) == Some("varchar")
        && !type_name.typmods.is_empty()
    {
        diagnostics.push(
            LinterDiagnostic::new(
                rule_category!(),
                None,
                markup! {
                    "Changing the size of a varchar field requires an ACCESS EXCLUSIVE lock."
                },
            )
            .note("Use a text field with a check constraint.")
            .maybe_fix(tokens.and_then(|tokens| use_text(tokens, type_name.location))),
        );
    }
}

/// Replaces `varchar(n)` with `text`. It is unsafe because it drops the length limit.
fn use_text(tokens: StatementTokens, location: i32) -> Option<RuleFix> {
    let start = tokens.at(location)?;
    let (_, close) = tokens.type_modifiers(tokens.type_name_end(start))?;
    Some(tokens.replace_keeping_comments(
        RuleFix::new("Use TEXT"),
        tokens.range(start).cover(tokens.range(close)),
        &in_case_of("text", tokens.text(start)),
    ))
}
