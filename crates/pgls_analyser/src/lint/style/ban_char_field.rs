use crate::fix::{StatementTokens, in_case_of};
use crate::linter_context::builtin_type_name;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Using CHAR(n) or CHARACTER(n) types is discouraged.
    ///
    /// CHAR types are fixed-length and padded with spaces, which can lead to unexpected behavior
    /// when comparing strings or concatenating values. They also waste storage space when values
    /// are shorter than the declared length.
    ///
    /// Use VARCHAR or TEXT instead for variable-length character data.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE "core_bar" (
    ///     "id" serial NOT NULL PRIMARY KEY,
    ///     "alpha" char(100) NOT NULL
    /// );
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE TABLE "core_bar" (
    ///     "id" serial NOT NULL PRIMARY KEY,
    ///     "alpha" varchar(100) NOT NULL
    /// );
    /// ```
    ///
    pub BanCharField {
        version: "0.15.0",
        name: "banCharField",
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
        recommended: false,
        sources: &[RuleSource::Squawk("ban-char-field")],
    }
}

impl LinterRule for BanCharField {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        if let pgls_query::NodeEnum::CreateStmt(stmt) = &ctx.stmt() {
            for table_elt in &stmt.table_elts {
                if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) = &table_elt.node
                    && let Some(diagnostic) = check_column_for_char_type(col_def, ctx.tokens())
                {
                    diagnostics.push(diagnostic);
                }
            }
        }

        // Also check ALTER TABLE ADD COLUMN statements
        if let pgls_query::NodeEnum::AlterTableStmt(stmt) = &ctx.stmt() {
            for cmd in &stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node
                    && cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAddColumn
                    && let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                        &cmd.def.as_ref().and_then(|d| d.node.as_ref())
                    && let Some(diagnostic) = check_column_for_char_type(col_def, ctx.tokens())
                {
                    diagnostics.push(diagnostic);
                }
            }
        }

        diagnostics
    }
}

fn check_column_for_char_type(
    col_def: &pgls_query::protobuf::ColumnDef,
    tokens: Option<StatementTokens>,
) -> Option<LinterDiagnostic> {
    let type_name = col_def.type_name.as_ref()?;
    // `char(n)` and `character(n)` are `bpchar`. The quoted `"char"` is a single-byte internal
    // type.
    if builtin_type_name(type_name)? != "bpchar" {
        return None;
    }
    Some(
        LinterDiagnostic::new(
            rule_category!(),
            None,
            markup! {
                "CHAR type is discouraged due to space padding behavior."
            },
        )
        .detail(None, "CHAR types are fixed-length and padded with spaces, which can lead to unexpected behavior.")
        .note("Use VARCHAR or TEXT instead for variable-length character data.")
        .maybe_fix(tokens.and_then(|tokens| use_varchar(tokens, type_name.location))),
    )
}

/// Replaces `char(n)` with `varchar(n)`. A `char` without a length is left alone, because it
/// means `char(1)`. It is unsafe because values are no longer padded with spaces, which changes
/// how they compare.
fn use_varchar(tokens: StatementTokens, location: i32) -> Option<RuleFix> {
    let start = tokens.at(location)?;
    let name = tokens.type_name_end(start);
    tokens.type_modifiers(name)?;
    Some(tokens.replace_keeping_comments(
        RuleFix::new("Use VARCHAR"),
        tokens.range(start).cover(tokens.range(name)),
        &in_case_of("varchar", tokens.text(start)),
    ))
}
