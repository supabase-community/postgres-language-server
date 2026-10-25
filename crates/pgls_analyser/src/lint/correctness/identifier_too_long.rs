use crate::{LinterDiagnostic, LinterRule, LinterRuleContext};
use pgls_analyse::{RuleSource, declare_lint_rule};
use pgls_catalog::identifier_name;
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::Token;
use pgls_text_size::{TextRange, TextSize};

/// `NAMEDATALEN - 1`, the longest identifier Postgres keeps.
///
/// See <https://github.com/postgres/postgres/blob/REL_18_6/src/include/pg_config_manual.h#L29>.
const MAX_IDENTIFIER_BYTES: usize = 63;

declare_lint_rule! {
    /// Identifiers longer than 63 bytes are silently truncated.
    ///
    /// Postgres truncates longer identifiers to 63 bytes and only raises a notice, so the names
    /// you write aren't the names Postgres uses. Two long names that share their first 63 bytes
    /// even end up as the same name.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// create table customer_subscription_billing_cycle_adjustments_history_archives_v2 (id bigint);
    /// ```
    ///
    /// ### Valid
    ///
    /// ```sql
    /// create table subscription_adjustments_archive (id bigint);
    /// ```
    ///
    pub IdentifierTooLong {
        version: "next",
        name: "identifierTooLong",
        severity: Severity::Warning,
        recommended: true,
        sources: &[RuleSource::Squawk("identifier-too-long")],
    }
}

impl LinterRule for IdentifierTooLong {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        // The parser already truncated the names in the syntax tree, so we read the tokens.
        // Downcasing an unquoted identifier doesn't change its length.
        let Some(sql) = ctx.sql() else {
            return vec![];
        };
        let Ok(scan) = pgls_query::scan(sql) else {
            return vec![];
        };

        scan.tokens
            .iter()
            .filter(|token| token.token() == Token::Ident)
            .filter_map(|token| {
                let range = TextRange::new(
                    TextSize::from(u32::try_from(token.start).ok()?),
                    TextSize::from(u32::try_from(token.end).ok()?),
                );
                let identifier =
                    identifier_name(sql.get(range.start().into()..range.end().into())?);
                (identifier.len() > MAX_IDENTIFIER_BYTES).then_some((range, identifier))
            })
            .map(|(range, identifier)| {
                let truncated = &identifier[..identifier.floor_char_boundary(MAX_IDENTIFIER_BYTES)];
                LinterDiagnostic::new(
                    rule_category!(),
                    Some(range),
                    markup! {
                        "Identifier "<Emphasis>{identifier}</Emphasis>" is longer than 63 bytes."
                    },
                )
                .detail(None, format!("Postgres truncates it to `{truncated}`."))
                .note("Use a shorter name.")
            })
            .collect()
    }
}
