use crate::fix::StatementTokens;
use crate::linter_context::builtin_type_name;
use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;

declare_lint_rule! {
    /// Prefer JSONB over JSON types.
    ///
    /// JSONB is the binary JSON data type in Postgres that is more efficient for most use cases.
    /// Unlike JSON, JSONB stores data in a decomposed binary format which provides several advantages:
    /// - Significantly faster query performance for operations like indexing and searching
    /// - Support for indexing (GIN indexes)
    /// - Duplicate keys are automatically removed
    /// - Keys are sorted
    ///
    /// The only reasons to use JSON instead of JSONB are:
    /// - You need to preserve exact input formatting (whitespace, key order)
    /// - You need to preserve duplicate keys
    /// - You have very specific performance requirements where JSON's lack of parsing overhead matters
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// CREATE TABLE users (
    ///     data json
    /// );
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE users ADD COLUMN metadata json;
    /// ```
    ///
    /// ```sql,expect_diagnostic
    /// ALTER TABLE users ALTER COLUMN data TYPE json;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// CREATE TABLE users (
    ///     data jsonb
    /// );
    /// ```
    ///
    /// ```sql
    /// ALTER TABLE users ADD COLUMN metadata jsonb;
    /// ```
    ///
    /// ```sql
    /// ALTER TABLE users ALTER COLUMN data TYPE jsonb;
    /// ```
    ///
    pub PreferJsonb {
        version: "0.15.0",
        name: "preferJsonb",
        severity: Severity::Warning,
        fix_kind: FixKind::Unsafe,
        recommended: false,
        sources: &[RuleSource::Eugene("E3")],
    }
}

impl LinterRule for PreferJsonb {
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
                    if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node
                        && (cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAddColumn
                            || cmd.subtype()
                                == pgls_query::protobuf::AlterTableType::AtAlterColumnType)
                        && let Some(pgls_query::NodeEnum::ColumnDef(col_def)) =
                            &cmd.def.as_ref().and_then(|d| d.node.as_ref())
                    {
                        check_column_def(&mut diagnostics, col_def, ctx.tokens());
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
    let Some(type_name) = &col_def.type_name else {
        return;
    };

    if builtin_type_name(type_name) != Some("json") {
        return;
    }

    diagnostics.push(
        LinterDiagnostic::new(
            rule_category!(),
            None,
            markup! {
                "Prefer JSONB over JSON for better performance and functionality."
            },
        )
        .detail(None, "JSON stores exact text representation while JSONB stores parsed binary format. JSONB is faster for queries, supports indexing, and removes duplicate keys.")
        .note("Consider using JSONB instead unless you specifically need to preserve formatting or duplicate keys.")
        .maybe_fix(tokens.and_then(|tokens| use_jsonb(tokens, type_name.location))),
    );
}

/// Replaces `json` with `jsonb`. It is unsafe because JSONB doesn't keep the formatting, key order
/// and duplicate keys of the input.
fn use_jsonb(tokens: StatementTokens, location: i32) -> Option<RuleFix> {
    let name = tokens.type_name_end(tokens.at(location)?);
    if !tokens.text(name).eq_ignore_ascii_case("json") {
        return None;
    }
    Some(tokens.replace(name, "Use JSONB", "jsonb"))
}
