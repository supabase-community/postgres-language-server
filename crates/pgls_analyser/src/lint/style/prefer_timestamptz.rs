use crate::fix::StatementTokens;
use crate::linter_context::builtin_type_name;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_text_size::TextRange;

declare_lint_rule! {
    /// Prefer TIMESTAMPTZ over TIMESTAMP types.
    ///
    /// Using TIMESTAMP WITHOUT TIME ZONE can lead to issues when dealing with time zones.
    /// TIMESTAMPTZ (TIMESTAMP WITH TIME ZONE) stores timestamps with time zone information,
    /// making it safer for applications that handle multiple time zones or need to track
    /// when events occurred in absolute time.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE app.users (
    ///     created_ts timestamp
    /// );
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE app.accounts (
    ///     created_ts timestamp without time zone
    /// );
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE app.users ALTER COLUMN created_ts TYPE timestamp;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE TABLE app.users (
    ///     created_ts timestamptz
    /// );
    /// ```
    ///
    /// ```sql
    /// CREATE TABLE app.accounts (
    ///     created_ts timestamp with time zone
    /// );
    /// ```
    ///
    /// ```sql
    /// ALTER TABLE app.users ALTER COLUMN created_ts TYPE timestamptz;
    /// ```
    ///
    pub PreferTimestamptz {
        version: "0.15.0",
        name: "preferTimestamptz",
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
        recommended: false,
        sources: &[RuleSource::Squawk("prefer-timestamptz")],
    }
}

impl LinterRule for PreferTimestamptz {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        let mut diagnostics = Vec::new();

        match &ctx.stmt() {
            pgls_query::NodeEnum::CreateStmt(stmt) => {
                for table_elt in &stmt.table_elts {
                    if let Some(pgls_query::NodeEnum::ColumnDef(col_def)) = &table_elt.node {
                        check_column_def(&mut diagnostics, col_def, ctx.tokens());
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
                                    check_column_def(&mut diagnostics, col_def, ctx.tokens());
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
    diagnostics: &mut Vec<LinterDiagnostic>,
    col_def: &pgls_query::protobuf::ColumnDef,
    tokens: Option<StatementTokens>,
) {
    if let Some(type_name) = &col_def.type_name {
        // Check for "timestamp" (without timezone)
        if builtin_type_name(type_name) == Some("timestamp") {
            diagnostics.push(
                LinterDiagnostic::new(
                    rule_category!(),
                    None,
                    markup! {
                        "Prefer TIMESTAMPTZ over TIMESTAMP for better timezone handling."
                    },
                )
                .detail(
                    None,
                    "TIMESTAMP WITHOUT TIME ZONE can lead to issues when dealing with time zones.",
                )
                .note("Use TIMESTAMPTZ (TIMESTAMP WITH TIME ZONE) instead.")
                .maybe_fix(tokens.and_then(|tokens| use_timestamptz(tokens, type_name.location))),
            );
        }
    }
}

/// Replaces `timestamp [(p)] [without time zone]` with `timestamptz [(p)]`. It is unsafe because
/// it changes how values are stored and compared.
fn use_timestamptz(tokens: StatementTokens, location: i32) -> Option<RuleFix> {
    use pgls_query::protobuf::Token;

    let name = tokens.type_name_end(tokens.at(location)?);
    if !tokens.text(name).eq_ignore_ascii_case("timestamp") {
        return None;
    }
    let mut fix = tokens.replace(name, "Use TIMESTAMPTZ", "timestamptz");

    let last = tokens.type_modifiers(name).map_or(name, |(_, close)| close);
    if tokens.kind(last + 1) == Some(Token::Without)
        && tokens.kind(last + 2) == Some(Token::Time)
        && tokens.kind(last + 3) == Some(Token::Zone)
    {
        let without_time_zone =
            TextRange::new(tokens.range(last).end(), tokens.range(last + 3).end());
        fix = tokens.replace_keeping_comments(fix, without_time_zone, "");
    }
    Some(fix)
}
