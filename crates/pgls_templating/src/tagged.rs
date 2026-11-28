//! Printing a statement with placeholders, and putting the text of the file back in place of
//! them.

use pgls_lexer::SyntaxKind;
use pgls_text_size::TextRange;

use crate::Preprocessed;

/// A statement of a [`Preprocessed`] text in which each placeholder that is a named parameter
/// has a tag of its own, `$pgls_t0`, `$pgls_t1`, …, and each string with placeholders too,
/// `'pgls_k0'`, `'pgls_k1'`, …, in text order.
///
/// Go's `%s` and Rust's `{}` take their arguments by position, and two `%s` have the same
/// placeholder, so a printer that moved one of them would go unnoticed, like the printer that
/// writes `limit '%d' offset '%d'` for `offset '%d' limit '%d'`. With the tags, the tree of the
/// statement shows where each placeholder is: a tree of the printed statement that is the same
/// as the tree of [`Tagged::text`] has each placeholder at its place, and [`Tagged::restore`]
/// checks that the printed statement has the placeholders in the same order, also those in
/// comments, which the tree doesn't have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tagged {
    /// The statement as the analysis reads it, with the tags.
    pub text: String,
    /// The text of the file of each tagged placeholder, by tag.
    named: Vec<String>,
    /// The placeholders that the statement holds as the host writes them, like Go's `%%` as
    /// `%`: what is printed for them, and their text in the file.
    written: Vec<(String, String)>,
    /// The strings and comments with placeholders, as written.
    kept: Vec<String>,
    /// The placeholders in text order, with one [`Slot::Kept`] for each string or comment.
    order: Vec<Slot>,
}

/// A placeholder of a [`Tagged`] statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Named(usize),
    Written(usize),
    Kept(usize),
}

impl Tagged {
    /// The start of the tag of a named parameter.
    const TAG: &'static str = "$pgls_t";

    /// The start of the tag of a string, between its quotes.
    const STRING_TAG: &'static str = "pgls_k";

    /// Puts the text of the file back in place of the placeholders of `printed`, the statement
    /// as a printer printed [`Tagged::text`] with the tags kept, or `None` if the printed
    /// placeholders differ from those of the statement: another order, one that is missing, or
    /// a string or a comment with placeholders that isn't the same.
    ///
    /// Cost: one lex of `printed`.
    pub fn restore(&self, printed: &str) -> Option<String> {
        let lexed = pgls_lexer::lex(printed);
        let mut restored = String::with_capacity(printed.len());
        let mut next = 0;
        for (index, kind) in lexed.tokens().enumerate() {
            if kind == SyntaxKind::EOF {
                break;
            }
            let text = lexed.text(index);
            let slot = if kind == SyntaxKind::NAMED_PARAM
                && let Some(tag) = text.strip_prefix(Self::TAG)
            {
                Some(Slot::Named(tag.parse().ok()?))
            } else if Self::is_string(kind) {
                // Strings without placeholders aren't in the order.
                Self::string_tag(text).map(Slot::Kept)
            } else if kind == SyntaxKind::COMMENT {
                // Comments without placeholders aren't in the order.
                match self.order.get(next) {
                    Some(Slot::Kept(kept)) if self.kept.get(*kept).is_some_and(|t| t == text) => {
                        Some(Slot::Kept(*kept))
                    }
                    _ => None,
                }
            } else if self.written.iter().any(|(written, _)| written == text) {
                // Every such token is a placeholder, also one that the statement had without
                // one, like a lone `%` of a Go template, which then fails the order.
                Some(match self.order.get(next)? {
                    Slot::Written(written) => Slot::Written(*written),
                    _ => return None,
                })
            } else {
                None
            };
            let Some(slot) = slot else {
                restored.push_str(text);
                continue;
            };
            if self.order.get(next) != Some(&slot) {
                return None;
            }
            next += 1;
            match slot {
                Slot::Named(tag) => restored.push_str(self.named.get(tag)?),
                Slot::Written(written) => restored.push_str(&self.written.get(written)?.1),
                Slot::Kept(kept) => restored.push_str(self.kept.get(kept)?),
            }
        }
        (next == self.order.len()).then_some(restored)
    }

    /// `text` with each tag of a named parameter as a plain name, `pgls_t0`, which the parser
    /// reads as written, unlike `$pgls_t0`, so that the statement can be compared with how
    /// Postgres reads it.
    pub fn readable(text: &str) -> String {
        text.replace(Self::TAG, &Self::TAG[1..])
    }

    /// Whether a token of `kind` holds placeholders as written.
    fn is_kept(kind: SyntaxKind) -> bool {
        Self::is_string(kind) || kind == SyntaxKind::COMMENT
    }

    fn is_string(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::STRING
                | SyntaxKind::ESC_STRING
                | SyntaxKind::BYTE_STRING
                | SyntaxKind::BIT_STRING
                | SyntaxKind::DOLLAR_QUOTED_STRING
        )
    }

    /// The tag of a printed string, `'pgls_k0'` as written or requoted as `$$pgls_k0$$`.
    fn string_tag(text: &str) -> Option<usize> {
        let inner = text
            .strip_prefix('\'')
            .and_then(|text| text.strip_suffix('\''))
            .or_else(|| {
                text.strip_prefix("$$")
                    .and_then(|text| text.strip_suffix("$$"))
            })?;
        inner.trim().strip_prefix(Self::STRING_TAG)?.parse().ok()
    }
}

impl Preprocessed {
    /// The statement at `range` of the file `source`, with tags for its placeholders, see
    /// [`Tagged`], or `None` if it has a placeholder that can't be printed, like one that is
    /// part of a number.
    ///
    /// Cost: one lex of the statement.
    pub fn tag(&self, source: &str, range: TextRange) -> Option<Tagged> {
        let analysed = self.text.get(std::ops::Range::<usize>::from(range))?;
        if analysed.contains(Tagged::TAG) || analysed.contains(Tagged::STRING_TAG) {
            return None;
        }
        let first = self
            .placeholders
            .partition_point(|placeholder| placeholder.end() <= range.start());
        let placeholders = self
            .placeholders
            .get(first..)?
            .iter()
            .take_while(|placeholder| placeholder.start() < range.end())
            .map(|placeholder| placeholder.checked_sub(range.start()))
            .collect::<Option<Vec<_>>>()?;

        let lexed = pgls_lexer::lex(analysed);
        let mut tagged = Tagged {
            text: String::with_capacity(analysed.len() + placeholders.len() * Tagged::TAG.len()),
            named: Vec::new(),
            written: Vec::new(),
            kept: Vec::new(),
            order: Vec::new(),
        };
        let mut placeholders = placeholders.into_iter().peekable();
        for (index, kind) in lexed.tokens().enumerate() {
            let token = lexed.range(index);
            let text = lexed.text(index);
            let mut in_token = std::iter::from_fn(|| {
                placeholders.next_if(|placeholder| placeholder.start() < token.end())
            })
            .peekable();
            let Some(placeholder) = in_token.next() else {
                tagged.text.push_str(text);
                continue;
            };
            let written = source.get(std::ops::Range::<usize>::from(
                placeholder.checked_add(range.start())?,
            ))?;
            if Tagged::is_kept(kind) {
                // A string or a comment keeps all of its placeholders as written. A string gets
                // a tag of its own, a comment, which isn't in the tree, stays. A comment that
                // the text doesn't hold as written, see [`Preprocessed::substitute`], can't be
                // put back.
                in_token.for_each(drop);
                if source.get(std::ops::Range::<usize>::from(token.checked_add(range.start())?))
                    != Some(text)
                {
                    return None;
                }
                if Tagged::is_string(kind) {
                    tagged
                        .text
                        .push_str(&format!("'{}{}'", Tagged::STRING_TAG, tagged.kept.len()));
                } else {
                    tagged.text.push_str(text);
                }
                tagged.order.push(Slot::Kept(tagged.kept.len()));
                tagged.kept.push(text.to_string());
            } else if kind == SyntaxKind::NAMED_PARAM && placeholder == token {
                tagged.order.push(Slot::Named(tagged.named.len()));
                tagged
                    .text
                    .push_str(&format!("{}{}", Tagged::TAG, tagged.named.len()));
                tagged.named.push(written.to_string());
            } else if placeholder.start() == token.start()
                && text != written
                // Not a placeholder that the text holds as written, like the `%c` of the
                // parameter `@mpo%ca`, whose `%` would read as the `%` of `%%`.
                && analysed.get(std::ops::Range::<usize>::from(placeholder)) != Some(written)
                && placeholder.end() >= token.end()
                && in_token.peek().is_none()
            {
                // What the host writes, like the `%` of `%%`, with its padding.
                tagged.order.push(Slot::Written(tagged.written.len()));
                tagged.written.push((text.to_string(), written.to_string()));
                tagged.text.push_str(text);
            } else {
                return None;
            }
            debug_assert!(token.start() <= placeholder.start() || Tagged::is_kept(kind));
        }
        // A placeholder past the last token, which can't be.
        if placeholders.peek().is_some() {
            return None;
        }
        Some(tagged)
    }
}

#[cfg(test)]
mod tests {
    use pgls_text_size::TextSize;

    use crate::Template;

    use super::*;

    /// The tagged statement of a template that is one statement.
    fn tagged(template: Template, text: &str) -> Option<Tagged> {
        template
            .read(text)?
            .tag(text, TextRange::up_to(TextSize::of(text)))
    }

    #[test]
    fn placeholders_get_a_tag_each_and_come_back_in_order() {
        let text = "select   %s, %s  from app_%s where x = '%d' and y %% 2 = 0 -- %v\n";
        let tagged = tagged(Template::GoPrintf, text).unwrap();
        assert_eq!(
            tagged.text,
            "select   $pgls_t0, $pgls_t1  from $pgls_t2 where x = 'pgls_k0' and y %  2 = 0 -- %v\n"
        );
        // A printer that only changes the layout gets the directives back.
        assert_eq!(
            tagged
                .restore(
                    "SELECT $pgls_t0, $pgls_t1 FROM $pgls_t2 WHERE x = 'pgls_k0' AND y % 2 = 0 -- %v\n"
                )
                .as_deref(),
            Some("SELECT %s, %s FROM app_%s WHERE x = '%d' AND y %% 2 = 0 -- %v\n")
        );
        for printed in [
            // Swapped, missing or repeated placeholders.
            "SELECT $pgls_t1, $pgls_t0 FROM $pgls_t2 WHERE x = 'pgls_k0' AND y % 2 = 0 -- %v\n",
            "SELECT $pgls_t0 FROM $pgls_t2 WHERE x = 'pgls_k0' AND y % 2 = 0 -- %v\n",
            "SELECT $pgls_t0, $pgls_t0, $pgls_t1 FROM $pgls_t2 WHERE x = 'pgls_k0' AND y % 2 = 0 -- %v\n",
            // A string or a comment with directives that changed, or a lost comment.
            "SELECT $pgls_t0, $pgls_t1 FROM $pgls_t2 WHERE x = E'pgls_k0' AND y % 2 = 0 -- %v\n",
            "SELECT $pgls_t0, $pgls_t1 FROM $pgls_t2 WHERE x = 'pgls_k0' AND y % 2 = 0\n",
            // A `%` more than the statement had.
            "SELECT $pgls_t0, $pgls_t1 FROM $pgls_t2 WHERE x = 'pgls_k0' AND y % 2 % 1 = 0 -- %v\n",
            // A comment that moved before a directive.
            "SELECT -- %v\n $pgls_t0, $pgls_t1 FROM $pgls_t2 WHERE x = 'pgls_k0' AND y % 2 = 0\n",
        ] {
            assert_eq!(tagged.restore(printed), None, "{printed}");
        }
    }

    #[test]
    fn rust_and_envsub_placeholders_are_tagged_too() {
        let tagged = tagged(
            Template::RustFmt,
            "select {}  from {schema}.t where n = {0:>4};",
        )
        .unwrap();
        assert_eq!(
            tagged.text,
            "select $pgls_t0  from $pgls_t1.t where n = $pgls_t2;"
        );
        assert_eq!(
            tagged
                .restore("select $pgls_t0 from $pgls_t1.t where n = $pgls_t2;")
                .as_deref(),
            Some("select {} from {schema}.t where n = {0:>4};")
        );
        let text = "create table ${SCHEMA}.events (id int, note text default '${NOTE}');";
        let envsub = Preprocessed::substitute(
            text,
            &[
                crate::Expansion {
                    range: TextRange::at(TextSize::from(13), TextSize::from(9)),
                    value: crate::Value::variable("SCHEMA"),
                },
                crate::Expansion {
                    range: TextRange::at(TextSize::from(60), TextSize::from(7)),
                    value: crate::Value::variable("NOTE"),
                },
            ],
        );
        let tagged = envsub
            .tag(text, TextRange::up_to(TextSize::of(text)))
            .unwrap();
        assert_eq!(
            tagged.text,
            "create table $pgls_t0.events (id int, note text default 'pgls_k0');"
        );
    }

    #[test]
    fn statements_that_hold_a_tag_are_not_tagged() {
        assert_eq!(tagged(Template::GoPrintf, "select $pgls_t0, %d;"), None);
    }
}
