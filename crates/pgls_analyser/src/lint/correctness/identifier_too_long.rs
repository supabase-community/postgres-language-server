use crate::{LinterDiagnostic, LinterRule, LinterRuleContext, RuleFix};
use pgls_analyse::{FixKind, RuleSource, declare_lint_rule};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_query::protobuf::{ScanToken, Token};
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
                Token::Ident => (identifier_value(token_text), true),
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
                let truncated = &identifier[..identifier.floor_char_boundary(MAX_IDENTIFIER_BYTES)];
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

/// The name a quoted or unquoted identifier stands for. Port of [`downcase_identifier`], which
/// only downcases ASCII letters in UTF-8.
///
/// [`downcase_identifier`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/scansup.c#L46
fn identifier_value(text: &str) -> String {
    match text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
    {
        Some(quoted) => quoted.replace("\"\"", "\""),
        None => text.to_ascii_lowercase(),
    }
}

/// The escape character of a `UESCAPE 'c'` clause. Port of [`check_uescapechar`].
///
/// [`check_uescapechar`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parser.c#L352
fn uescape_char(literal: &str) -> Option<char> {
    let value = literal
        .strip_prefix('\'')?
        .strip_suffix('\'')?
        .replace("''", "'");
    let mut chars = value.chars();
    let escape = chars.next()?;
    (chars.next().is_none()
        && escape.is_ascii()
        && !escape.is_ascii_hexdigit()
        && !matches!(
            escape,
            '+' | '\'' | '"' | ' ' | '\t' | '\n' | '\r' | '\x0b' | '\x0c'
        ))
    .then_some(escape)
}

/// The name a `U&"..."` identifier stands for, or `None` if Postgres rejects its escapes. Port of
/// [`str_udeescape`].
///
/// [`str_udeescape`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parser.c#L372
fn unicode_identifier_value(text: &str, escape: char) -> Option<String> {
    let quoted = text.get(2..)?.strip_prefix('"')?.strip_suffix('"')?;
    let value = quoted.replace("\"\"", "\"");
    let mut chars = value.chars().peekable();
    let mut out = String::with_capacity(value.len());
    let mut high_surrogate: Option<u32> = None;
    while let Some(c) = chars.next() {
        if c != escape {
            if high_surrogate.is_some() {
                return None;
            }
            out.push(c);
            continue;
        }
        if chars.peek() == Some(&escape) {
            if high_surrogate.is_some() {
                return None;
            }
            chars.next();
            out.push(escape);
            continue;
        }
        let digits = if chars.peek() == Some(&'+') {
            chars.next();
            6
        } else {
            4
        };
        let hex: String = chars.by_ref().take(digits).collect();
        if hex.len() != digits || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let code = u32::from_str_radix(&hex, 16).ok()?;
        match (high_surrogate.take(), code) {
            (None, 0xD800..=0xDBFF) => high_surrogate = Some(code),
            (Some(high), 0xDC00..=0xDFFF) => out.push(char::from_u32(
                0x10000 + ((high - 0xD800) << 10) + (code - 0xDC00),
            )?),
            (Some(_), _) | (None, 0xDC00..=0xDFFF) => return None,
            (None, _) => out.push(char::from_u32(code).filter(|c| *c != '\0')?),
        }
    }
    high_surrogate.is_none().then_some(out)
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

    #[test]
    fn unicode_identifiers() {
        assert_eq!(
            unicode_identifier_value(r#"U&"d\0061t\+000061""#, '\\').as_deref(),
            Some("data")
        );
        assert_eq!(
            unicode_identifier_value(r#"u&"a!!b!0063""#, '!').as_deref(),
            Some("a!bc")
        );
        assert_eq!(
            unicode_identifier_value(r#"U&"a""b""#, '\\').as_deref(),
            Some("a\"b")
        );
        assert_eq!(
            unicode_identifier_value(r#"U&"\D83D\DE00""#, '\\').as_deref(),
            Some("😀")
        );
        assert_eq!(unicode_identifier_value(r#"U&"\D83D""#, '\\'), None);
        assert_eq!(unicode_identifier_value(r#"U&"\00""#, '\\'), None);
        assert_eq!(unicode_identifier_value(r#"U&"\0000""#, '\\'), None);
        assert_eq!(uescape_char("'!'"), Some('!'));
        assert_eq!(uescape_char("'a'"), None);
        assert_eq!(uescape_char("'!!'"), None);
    }
}
