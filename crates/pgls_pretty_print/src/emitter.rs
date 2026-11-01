use std::collections::HashMap;

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
    GroupStart,
    GroupEnd,
    IndentStart,
    IndentEnd,
    /// Align continuation lines with the current column until `AlignEnd`.
    AlignStart,
    AlignEnd,
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
    /// `config.layout` as configured, while `with_layout` overrides it for part of a statement.
    configured_layout: crate::Layout,
    /// Identifiers the parser truncated, by their truncated name, as written in the source.
    truncated_identifiers: HashMap<String, String>,
}

impl EventEmitter {
    /// An emitter without comments, for unit tests that look at the events.
    #[cfg(test)]
    pub fn new(config: FormatConfig) -> Self {
        Self::with_comments(config, HashMap::new(), HashMap::new())
    }

    // Later option PRs inspect this while deciding which layout events to emit.
    #[allow(dead_code)]
    pub fn config(&self) -> &FormatConfig {
        &self.config
    }

    /// The layout from the configuration, which `with_layout` does not change.
    pub fn configured_layout(&self) -> crate::Layout {
        self.configured_layout.clone()
    }

    /// Emits `body` with `layout` in place of the configured one.
    pub fn with_layout(&mut self, layout: crate::Layout, body: impl FnOnce(&mut Self)) {
        let previous = std::mem::replace(&mut self.config.layout, layout);
        body(self);
        self.config.layout = previous;
    }

    pub fn with_comments(
        config: FormatConfig,
        leading_comments: HashMap<i32, Vec<Comment>>,
        trailing_comments: HashMap<i32, Vec<Comment>>,
    ) -> Self {
        Self {
            events: Vec::new(),
            configured_layout: config.layout.clone(),
            config,
            leading_comments,
            trailing_comments,
            truncated_identifiers: HashMap::new(),
            leading_line_comments_require_break: false,
            comments_after_separator: Vec::new(),
        }
    }

    /// Identifiers of the source that the parser truncated, by their truncated name. The emitter
    /// writes them as written rather than as the AST has them.
    pub fn set_truncated_identifiers(&mut self, identifiers: HashMap<String, String>) {
        self.truncated_identifiers = identifiers;
    }

    /// `name` as written in the source, when the parser truncated it.
    pub fn source_identifier<'a>(&'a self, name: &'a str) -> &'a str {
        self.truncated_identifiers
            .get(name)
            .map_or(name, String::as_str)
    }

    pub fn token(&mut self, token: TokenKind) {
        // A comment held for a comma follows it. Other tokens closing the item before the comma,
        // a parenthesis or an alias for instance, are written first. A statement never ends on a
        // comma, so the terminator flushes what is left.
        let is_comma = matches!(token, TokenKind::COMMA);
        if matches!(token, TokenKind::SEMICOLON) {
            self.flush_comments_after_separator();
        }
        self.events.push(LayoutEvent::Token(token));
        if is_comma {
            self.flush_comments_after_separator();
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

    /// Emits and consumes the comments before `location` that were written before `keyword`,
    /// the way trailing comments of the previous node are emitted.
    pub fn take_comments_before_keyword_at(&mut self, location: i32, keyword: &str) {
        let Some(comments) = self.leading_comments.get_mut(&location) else {
            return;
        };
        let (before_keyword, rest): (Vec<_>, Vec<_>) = std::mem::take(comments)
            .into_iter()
            .partition(|comment| comment.before_keyword.as_deref() == Some(keyword));
        if rest.is_empty() {
            self.leading_comments.remove(&location);
        } else {
            *comments = rest;
        }
        self.emit_trailing_comments(before_keyword);
    }

    /// Emits and consumes comments that follow `location`, if any.
    pub fn take_trailing_comments_at(&mut self, location: i32) {
        let comments = self.remove_trailing_comments_at(location);
        self.emit_trailing_comments(comments);
    }

    /// Consumes the comments that follow `location` without emitting them, so that the caller
    /// can emit them somewhere else with `emit_trailing_comments`.
    pub fn remove_trailing_comments_at(&mut self, location: i32) -> Vec<Comment> {
        self.trailing_comments.remove(&location).unwrap_or_default()
    }

    /// Puts back comments taken with `remove_trailing_comments_at`.
    pub fn restore_trailing_comments_at(&mut self, location: i32, comments: Vec<Comment>) {
        if !comments.is_empty() {
            self.trailing_comments.insert(location, comments);
        }
    }

    /// Emits comments that follow the node just emitted.
    pub fn emit_trailing_comments(&mut self, comments: Vec<Comment>) {
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

    pub fn group_start(&mut self) {
        self.events.push(LayoutEvent::GroupStart);
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

    pub fn align_start(&mut self) {
        self.events.push(LayoutEvent::AlignStart);
    }

    pub fn align_end(&mut self) {
        self.events.push(LayoutEvent::AlignEnd);
    }
}
