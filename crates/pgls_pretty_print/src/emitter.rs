use std::collections::HashMap;

pub use crate::codegen::group_kind::GroupKind;
pub use crate::codegen::token_kind::TokenKind;
use crate::{Comment, FormatConfig};

#[derive(Debug, Clone, PartialEq)]
pub enum LineType {
    /// Must break (semicolon, etc.)
    Hard,
    /// Break if group doesn't fit
    Soft,
    /// Break if group doesn't fit, but collapse to space if it does
    SoftOrSpace,
    /// Break only when the next item would exceed the configured line width.
    Fill,
    /// Like `Fill`, but disappears instead of becoming a space when the item fits.
    FillNoSpace,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutEvent {
    Token(TokenKind),
    Space,
    Line(LineType),
    /// A comment from the source. `line_comment` is true for `--`, which runs to the end of the
    /// line and therefore forbids collapsing the enclosing group.
    Comment {
        text: String,
        line_comment: bool,
    },
    GroupStart {
        kind: GroupKind,
    },
    GroupEnd,
    IndentStart,
    IndentEnd,
}

/// Collects layout events for the renderer.
///
/// The emitter holds the configuration because some options decide which tokens exist at all,
/// such as where a comma sits in a list, and not merely how a token is rendered.
#[derive(Debug)]
pub struct EventEmitter {
    pub events: Vec<LayoutEvent>,
    config: FormatConfig,
    /// Comments still waiting to be emitted before a node, by source location.
    /// Entries are removed as they are emitted so that a comment cannot be printed twice, and so
    /// that the caller can check the map is empty afterwards.
    leading_comments: HashMap<i32, Vec<Comment>>,
    /// Comments still waiting to be emitted after a node, by source location.
    trailing_comments: HashMap<i32, Vec<Comment>>,
    /// Boolean operands need standalone line comments to start their own line, otherwise a
    /// comment can join the preceding operand and drift on the next formatting pass.
    leading_line_comments_require_break: bool,
    /// Line comments written after a list comma, waiting for the emitter to reach that comma.
    comments_after_separator: Vec<Comment>,
    /// The text the AST was parsed from. The AST drops how a literal was quoted, so this is where
    /// a function body gets its dollar-quote tag back from.
    source: Option<Source>,
}

#[derive(Debug)]
struct Source {
    text: String,
    /// Byte ranges of the string literals in `text`, in source order.
    string_literals: Vec<(usize, usize)>,
}

impl EventEmitter {
    pub fn new(config: FormatConfig) -> Self {
        Self {
            events: Vec::new(),
            config,
            leading_comments: HashMap::new(),
            trailing_comments: HashMap::new(),
            leading_line_comments_require_break: false,
            comments_after_separator: Vec::new(),
            source: None,
        }
    }

    /// Attaches the text the AST was parsed from, so that literals keep the quoting they were
    /// written with. Without it, a literal is quoted the default way.
    pub fn with_source(mut self, sql: &str) -> Self {
        let string_literals = pgls_query::scan(sql)
            .map(|scan| {
                scan.tokens
                    .iter()
                    .filter(|token| token.token == pgls_query::protobuf::Token::Sconst as i32)
                    .filter_map(|token| {
                        Some((
                            usize::try_from(token.start).ok()?,
                            usize::try_from(token.end).ok()?,
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.source = Some(Source {
            text: sql.to_string(),
            string_literals,
        });
        self
    }

    /// The source text of the first string literal at or after `location`.
    pub fn source_string_literal_from(&self, location: i32) -> Option<&str> {
        let source = self.source.as_ref()?;
        let location = usize::try_from(location).ok()?;
        let index = source
            .string_literals
            .partition_point(|(start, _)| *start < location);
        let (start, end) = *source.string_literals.get(index)?;
        source.text.get(start..end)
    }

    // Later option PRs inspect this while deciding which layout events to emit.
    #[allow(dead_code)]
    pub fn config(&self) -> &FormatConfig {
        &self.config
    }

    pub fn with_comments(
        config: FormatConfig,
        leading_comments: HashMap<i32, Vec<Comment>>,
        trailing_comments: HashMap<i32, Vec<Comment>>,
    ) -> Self {
        Self {
            events: Vec::new(),
            config,
            leading_comments,
            trailing_comments,
            leading_line_comments_require_break: false,
            comments_after_separator: Vec::new(),
            source: None,
        }
    }

    pub fn token(&mut self, token: TokenKind) {
        // A comment held for a comma follows it. Other tokens closing the item before the comma,
        // a parenthesis or an alias for instance, are written first. A statement never ends on a
        // comma, so the terminator flushes what is left.
        if self.comments_after_separator.is_empty() {
            self.events.push(LayoutEvent::Token(token));
        } else if matches!(token, TokenKind::COMMA) {
            self.events.push(LayoutEvent::Token(token));
            self.flush_comments_after_separator();
        } else {
            if matches!(token, TokenKind::SEMICOLON) {
                self.flush_comments_after_separator();
            }
            self.events.push(LayoutEvent::Token(token));
        }
    }

    pub fn space(&mut self) {
        self.events.push(LayoutEvent::Space);
    }

    pub fn line(&mut self, line_type: LineType) {
        // With leading commas, the comma follows a line break: the comment ends the item's line.
        if matches!(self.config.comma_style, crate::CommaStyle::Leading) {
            self.flush_comments_after_separator();
        }

        if let Some(LayoutEvent::Line(previous)) = self.events.last_mut()
            && (matches!(&*previous, LineType::Hard) || matches!(&line_type, LineType::Hard))
        {
            *previous = LineType::Hard;
            return;
        }

        self.events.push(LayoutEvent::Line(line_type));
    }

    pub fn comment(&mut self, text: String, line_comment: bool) {
        self.flush_comments_after_separator();
        self.events
            .push(LayoutEvent::Comment { text, line_comment });
    }

    fn flush_comments_after_separator(&mut self) {
        for comment in std::mem::take(&mut self.comments_after_separator) {
            self.events.push(LayoutEvent::Space);
            self.events.push(LayoutEvent::Comment {
                text: comment.text,
                line_comment: true,
            });
            self.line(LineType::Hard);
        }
    }

    /// Emits and consumes comments that precede `location`, if any.
    pub fn take_leading_comments_at(&mut self, location: i32) {
        let Some(comments) = self.leading_comments.remove(&location) else {
            return;
        };

        for comment in comments {
            let line_comment = comment.line_comment;
            let multiline = comment.text.contains('\n');
            if line_comment && comment.own_line && self.leading_line_comments_require_break {
                self.force_current_line_break();
            }
            self.comment(comment.text, line_comment);
            if line_comment || multiline {
                self.line(LineType::Hard);
            } else {
                self.space();
            }
        }
    }

    /// Emits leading comments before a leading list separator when they were written on their
    /// own line.
    ///
    /// With leading commas, the separator belongs visually to the following item. An own-line
    /// comment documenting that item must remain before the comma instead of becoming `, /* ... */`.
    pub fn take_own_line_leading_comments_at(&mut self, location: i32) -> bool {
        let Some(comments) = self.leading_comments.get(&location) else {
            return false;
        };
        if !comments.iter().any(|comment| comment.own_line) {
            return false;
        }

        let comments = self
            .leading_comments
            .remove(&location)
            .expect("leading comments checked above");
        self.force_current_line_break();

        for comment in comments {
            let line_comment = comment.line_comment;
            self.comment(comment.text, line_comment);
            self.line(LineType::Hard);
        }

        true
    }

    /// Emits and consumes comments that follow `location`, if any.
    pub fn take_trailing_comments_at(&mut self, location: i32) {
        let Some(comments) = self.trailing_comments.remove(&location) else {
            return;
        };

        for comment in comments {
            if comment.after_separator {
                self.comments_after_separator.push(comment);
                continue;
            }

            let line_comment = comment.line_comment;
            let multiline = comment.text.contains('\n');
            if comment.own_line {
                self.force_current_line_break();
            } else {
                self.space();
            }
            self.comment(comment.text, line_comment);
            if line_comment || multiline {
                self.line(LineType::Hard);
            } else {
                self.space();
            }
        }
    }

    pub fn pending_comments(&self) -> usize {
        self.leading_comments.values().map(Vec::len).sum::<usize>()
            + self.trailing_comments.values().map(Vec::len).sum::<usize>()
            + self.comments_after_separator.len()
    }

    pub fn with_leading_comment_line_break(&mut self, body: impl FnOnce(&mut EventEmitter)) {
        let previous = std::mem::replace(&mut self.leading_line_comments_require_break, true);
        body(self);
        self.leading_line_comments_require_break = previous;
    }

    fn force_current_line_break(&mut self) {
        match self.events.last_mut() {
            None => {}
            Some(LayoutEvent::Line(line_type)) => *line_type = LineType::Hard,
            Some(_) => self.line(LineType::Hard),
        }
    }

    pub fn group_start(&mut self, kind: GroupKind) {
        self.events.push(LayoutEvent::GroupStart { kind });
    }

    pub fn group_end(&mut self) {
        self.events.push(LayoutEvent::GroupEnd);
    }

    pub fn expression_group_end(&mut self) {
        // A comment after the final operand should not force the whole expression to expand.
        // Keep its suffix outside the group while preserving its position in the SQL.
        let suffix_start = self.events.len().checked_sub(3);
        if let Some(start) = suffix_start
            && matches!(
                &self.events[start..],
                [
                    LayoutEvent::Space,
                    LayoutEvent::Comment {
                        line_comment: true,
                        ..
                    },
                    LayoutEvent::Line(LineType::Hard)
                ]
            )
        {
            self.events.insert(start, LayoutEvent::GroupEnd);
            return;
        }
        self.group_end();
    }

    pub fn indent_start(&mut self) {
        self.events.push(LayoutEvent::IndentStart);
    }

    pub fn indent_end(&mut self) {
        self.events.push(LayoutEvent::IndentEnd);
    }
}
