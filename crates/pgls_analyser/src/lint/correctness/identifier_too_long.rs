use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_catalog::{
    MAX_IDENTIFIER_BYTES, truncate_identifier, uescape_char, unicode_identifier_value,
    untruncated_identifier_name,
};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{ScanToken, Token};
use pgls_text_size::{TextRange, TextSize};

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
        fix_kind: FixKind::Safe,
    }
}

impl LinterRule for IdentifierTooLong {
    type Options = ();

    fn run(ctx: &LinterRuleContext<Self>) -> Vec<LinterDiagnostic> {
        // The parser already truncated the names in the syntax tree, so we read the tokens.
        let Some(sql) = ctx.sql() else {
            return vec![];
        };
        // Scanning costs about as much as parsing, and almost no statement has a long name.
        if !may_contain_long_identifier(sql) {
            return vec![];
        }
        let Ok(scan) = pgls_query::scan(sql) else {
            return vec![];
        };

        let tokens = &scan.tokens;
        let text = |token: &ScanToken| {
            let range = TextRange::new(
                TextSize::from(u32::try_from(token.start).ok()?),
                TextSize::from(u32::try_from(token.end).ok()?),
            );
            Some((range, sql.get(range.start().into()..range.end().into())?))
        };

        // Each long name, with the identifiers that spell it.
        let mut names: Vec<(String, Vec<Occurrence>)> = Vec::new();
        for (i, token) in tokens.iter().enumerate() {
            let Some((range, token_text)) = text(token) else {
                continue;
            };
            let (identifier, replaceable) = match token.token() {
                Token::Ident => (untruncated_identifier_name(token_text), true),
                Token::Uident => {
                    let escape = match (tokens.get(i + 1), tokens.get(i + 2)) {
                        (Some(uescape), Some(escape)) if uescape.token() == Token::Uescape => {
                            match text(escape).and_then(|(_, escape)| uescape_char(escape)) {
                                Some(escape) => escape,
                                None => continue,
                            }
                        }
                        _ => '\\',
                    };
                    match unicode_identifier_value(token_text, escape) {
                        // Rewriting a `U&"..."` identifier would leave its UESCAPE clause behind.
                        Some(identifier) => (identifier, false),
                        None => continue,
                    }
                }
                _ => continue,
            };
            if identifier.len() <= MAX_IDENTIFIER_BYTES {
                continue;
            }
            let occurrence = Occurrence {
                range,
                quoted: token_text.starts_with('"'),
                replaceable,
            };
            match names.iter_mut().find(|(name, _)| *name == identifier) {
                Some((_, occurrences)) => occurrences.push(occurrence),
                None => names.push((identifier, vec![occurrence])),
            }
        }

        names
            .into_iter()
            .map(|(identifier, occurrences)| {
                let truncated = truncate_identifier(&identifier);
                LinterDiagnostic::new(
                    rule_category!(),
                    Some(occurrences[0].range),
                    markup! {
                        "Identifier "<Emphasis>{identifier}</Emphasis>" is longer than 63 bytes."
                    },
                )
                .detail(None, format!("Postgres truncates it to `{truncated}`."))
                .note("Use a shorter name.")
                .maybe_fix(truncate(sql, truncated, &occurrences))
            })
            .collect()
    }
}

/// An identifier that spells a long name.
struct Occurrence {
    range: TextRange,
    quoted: bool,
    /// Whether the fix can rewrite it.
    replaceable: bool,
}

/// Spells out the name Postgres uses, in every identifier of the statement that stands for it.
/// This doesn't change what the statement does.
fn truncate(sql: &str, truncated: &str, occurrences: &[Occurrence]) -> Option<RuleFix> {
    let mut fix = RuleFix::new(format!("Use the truncated name `{truncated}`"));
    for occurrence in occurrences
        .iter()
        .filter(|occurrence| occurrence.replaceable)
    {
        let replacement = if occurrence.quoted {
            format!("\"{}\"", truncated.replace('"', "\"\""))
        } else {
            // Keep the case as written, downcasing it doesn't change the length.
            let text = &sql[occurrence.range];
            text[..text.floor_char_boundary(MAX_IDENTIFIER_BYTES)].to_owned()
        };
        fix = fix.replace(occurrence.range, replacement);
    }
    (!fix.edits.is_empty()).then_some(fix)
}

/// Whether `sql` may contain an identifier longer than 63 bytes, without scanning it. A name is
/// never longer than its identifier: downcasing keeps the length, and `""` and Unicode escapes
/// are longer than the characters they stand for. So an unquoted identifier needs a run of more
/// than 63 bytes that can be part of one ([`ident_cont`]), and a quoted one needs more than 63
/// bytes between two quotes. Inside a quoted identifier, quotes come in pairs, so only an odd
/// number of consecutive quotes can start or end one.
///
/// [`ident_cont`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scan.l#L333
fn may_contain_long_identifier(sql: &str) -> bool {
    let bytes = sql.as_bytes();
    // The current run of identifier bytes.
    let mut word = 0;
    // The bytes since the last odd run of quotes, if there was one.
    let mut quoted: Option<usize> = None;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let quotes = bytes[i..].iter().take_while(|byte| **byte == b'"').count();
            word = 0;
            i += quotes;
            if quotes % 2 == 0 {
                quoted = quoted.map(|length| length + quotes);
            } else {
                // The other quotes of the run belong to the identifier that ends or starts here.
                if quoted.is_some_and(|length| length + quotes - 1 > MAX_IDENTIFIER_BYTES) {
                    return true;
                }
                quoted = Some(quotes - 1);
            }
            continue;
        }
        let byte = bytes[i];
        word = if byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || byte >= 0x80 {
            word + 1
        } else {
            0
        };
        if word > MAX_IDENTIFIER_BYTES {
            return true;
        }
        quoted = quoted.map(|length| length + 1);
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefilter_finds_every_long_identifier() {
        let long = "a".repeat(64);
        for sql in [
            format!("select {long}"),
            format!("select \"{long}\""),
            format!("select \"{}\"\"\"", "a".repeat(62)),
            format!("select \"\"\"{}\"", "a".repeat(62)),
            format!("select \"{} b\"", "a".repeat(62)),
            format!("select '\"', \"{} b c\" /* \" */", "a".repeat(60)),
            format!("select U&\"{}\\0061\"", "a".repeat(60)),
        ] {
            assert!(may_contain_long_identifier(&sql), "{sql}");
        }
    }

    #[test]
    fn prefilter_skips_short_identifiers() {
        for sql in [
            "select id, name from public.users where id = 1".to_owned(),
            format!("select \"{0}\", \"{0}\" from \"{0}\"", "a".repeat(63)),
            format!("select \"a\" from t where name = '{}'", "b ".repeat(40)),
            format!("select {}", "a ".repeat(100)),
        ] {
            assert!(!may_contain_long_identifier(&sql), "{sql}");
        }
    }
}
