use std::collections::HashMap;

pub(crate) use crate::codegen::token_kind::TokenKind;
use crate::{Comment, FormatConfig};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LineType {
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
pub(crate) enum LayoutEvent {
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
pub(crate) struct EventEmitter<'a> {
    pub events: Vec<LayoutEvent>,
    /// The configuration as given. Its `layout` is the configured one; `layout` below is the one
    /// in effect.
    config: &'a FormatConfig,
    /// Comments still waiting to be emitted before a node, by source location.
    /// Entries are removed as they are emitted so that a comment cannot be printed twice, and so
    /// that the caller can check the map is empty afterwards.
    leading_comments: HashMap<i32, Vec<Comment>>,
    /// Comments still waiting to be emitted after a node, by source location.
    trailing_comments: HashMap<i32, Vec<Comment>>,
    /// Comments waiting for the END of the CASE at this source location.
    comments_before_case_end: HashMap<i32, Vec<Comment>>,
    /// Boolean operands need standalone line comments to start their own line, otherwise a
    /// comment can join the preceding operand and drift on the next formatting pass.
    leading_line_comments_require_break: bool,
    /// Line comments written after a list comma, waiting for the emitter to reach that comma.
    comments_after_separator: Vec<Comment>,
    /// The layout in effect: `config.layout`, unless `with_layout` overrides it for part of a
    /// statement.
    layout: crate::Layout,
    /// Identifiers the parser truncated, by their truncated name, as written in the source.
    truncated_identifiers: HashMap<String, String>,
    /// The locations comments are attached to, in source order.
    node_locations: Vec<i32>,
}

impl<'a> EventEmitter<'a> {
    /// An emitter without comments, for unit tests that look at the events.
    #[cfg(test)]
    pub(crate) fn new(config: &'a FormatConfig) -> Self {
        Self::with_comments(config, HashMap::new(), HashMap::new())
    }

    /// The configuration. Read the layout in effect with `layout`, not from here.
    pub(crate) fn config(&self) -> &FormatConfig {
        self.config
    }

    /// The layout in effect, which `with_layout` overrides for part of a statement.
    pub(crate) fn layout(&self) -> &crate::Layout {
        &self.layout
    }

    /// The layout from the configuration, which `with_layout` does not change.
    pub(crate) fn configured_layout(&self) -> crate::Layout {
        self.config.layout.clone()
    }

    /// Emits `body` with `layout` in place of the configured one.
    pub(crate) fn with_layout(&mut self, layout: crate::Layout, body: impl FnOnce(&mut Self)) {
        let previous = std::mem::replace(&mut self.layout, layout);
        body(self);
        self.layout = previous;
    }

    pub(crate) fn with_comments(
        config: &'a FormatConfig,
        leading_comments: HashMap<i32, Vec<Comment>>,
        trailing_comments: HashMap<i32, Vec<Comment>>,
    ) -> Self {
        Self {
            events: Vec::new(),
            layout: config.layout.clone(),
            config,
            leading_comments,
            trailing_comments,
            comments_before_case_end: HashMap::new(),
            truncated_identifiers: HashMap::new(),
            node_locations: Vec::new(),
            leading_line_comments_require_break: false,
            comments_after_separator: Vec::new(),
        }
    }

    /// Comments anchored to a CASE's closing keyword rather than to a node inside its last branch.
    pub(crate) fn set_comments_before_case_end(&mut self, comments: HashMap<i32, Vec<Comment>>) {
        self.comments_before_case_end = comments;
    }

    pub(crate) fn take_comments_before_case_end_at(&mut self, location: i32) {
        let comments = self
            .comments_before_case_end
            .remove(&location)
            .unwrap_or_default();
        self.emit_trailing_comments(comments);
    }

    /// Identifiers of the source that the parser truncated, by their truncated name. The emitter
    /// writes them as written rather than as the AST has them.
    pub(crate) fn set_truncated_identifiers(&mut self, identifiers: HashMap<String, String>) {
        self.truncated_identifiers = identifiers;
    }

    /// The locations comments are attached to, in source order.
    pub(crate) fn set_node_locations(&mut self, locations: Vec<i32>) {
        self.node_locations = locations;
    }

    /// The location of the node that follows the one at `location` in the source, which holds the
    /// comments written between the two.
    pub(crate) fn next_node_location(&self, location: i32) -> Option<i32> {
        let index = self
            .node_locations
            .partition_point(|other| *other <= location);
        self.node_locations.get(index).copied()
    }

    /// `name` as written in the source, when the parser truncated it.
    pub(crate) fn source_identifier<'n>(&'n self, name: &'n str) -> &'n str {
        self.truncated_identifiers
            .get(name)
            .map_or(name, String::as_str)
    }

    pub(crate) fn token(&mut self, token: TokenKind) {
        // A comment held for a comma follows it. Other tokens closing the item before the comma,
        // a parenthesis or an alias for instance, are written first. A statement never ends on a
        // comma, so the terminator flushes what is left.
        let is_comma = matches!(token, TokenKind::COMMA);
        if matches!(token, TokenKind::SEMICOLON) {
            self.flush_comments_after_separator();
        }
        // A comment before the comma, `a /* c */, b`, is followed by a space meant for the next
        // token, which the comma doesn't take. With leading commas, the break before the comma
        // comes in between.
        if is_comma
            && let Some(index) = self
                .events
                .iter()
                .rposition(|event| !matches!(event, LayoutEvent::Line(_)))
            && matches!(self.events.get(index), Some(LayoutEvent::Space))
        {
            self.events.remove(index);
        }
        self.events.push(LayoutEvent::Token(token));
        if is_comma {
            self.flush_comments_after_separator();
        }
    }

    pub(crate) fn space(&mut self) {
        self.events.push(LayoutEvent::Space);
    }

    pub(crate) fn line(&mut self, line_type: LineType) {
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

    pub(crate) fn comment(&mut self, text: String, line_comment: bool) {
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
    pub(crate) fn take_leading_comments_at(&mut self, location: i32) {
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
    pub(crate) fn take_own_line_leading_comments_at(&mut self, location: i32) -> bool {
        let Some(comments) = self.leading_comments.get(&location) else {
            return false;
        };
        if !comments.iter().any(|comment| comment.own_line) {
            return false;
        }

        // Checked above.
        let Some(comments) = self.leading_comments.remove(&location) else {
            return false;
        };
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
    pub(crate) fn take_comments_before_keyword_at(&mut self, location: i32, keyword: &str) {
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

    /// Emits and consumes the comments before `location` the way trailing comments of the previous
    /// node are emitted.
    pub(crate) fn take_leading_comments_as_trailing_at(&mut self, location: i32) {
        let comments = self.leading_comments.remove(&location).unwrap_or_default();
        self.emit_trailing_comments(comments);
    }

    /// Emits and consumes comments that follow `location`, if any.
    pub(crate) fn take_trailing_comments_at(&mut self, location: i32) {
        let comments = self.remove_trailing_comments_at(location);
        self.emit_trailing_comments(comments);
    }

    /// Consumes the comments that follow `location` without emitting them, so that the caller
    /// can emit them somewhere else with `emit_trailing_comments`.
    pub(crate) fn remove_trailing_comments_at(&mut self, location: i32) -> Vec<Comment> {
        self.trailing_comments.remove(&location).unwrap_or_default()
    }

    /// Puts back comments taken with `remove_trailing_comments_at`.
    pub(crate) fn restore_trailing_comments_at(&mut self, location: i32, comments: Vec<Comment>) {
        if !comments.is_empty() {
            self.trailing_comments.insert(location, comments);
        }
    }

    /// Emits comments that follow the node just emitted.
    pub(crate) fn emit_trailing_comments(&mut self, comments: Vec<Comment>) {
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

    pub(crate) fn pending_comments(&self) -> usize {
        self.leading_comments.values().map(Vec::len).sum::<usize>()
            + self.trailing_comments.values().map(Vec::len).sum::<usize>()
            + self
                .comments_before_case_end
                .values()
                .map(Vec::len)
                .sum::<usize>()
            + self.comments_after_separator.len()
    }

    pub(crate) fn with_leading_comment_line_break(&mut self, body: impl FnOnce(&mut EventEmitter)) {
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

    pub(crate) fn group_start(&mut self) {
        self.events.push(LayoutEvent::GroupStart);
    }

    pub(crate) fn group_end(&mut self) {
        self.events.push(LayoutEvent::GroupEnd);
    }

    pub(crate) fn expression_group_end(&mut self) {
        // A comment after the final operand should not force the whole expression to expand.
        // Keep its suffix outside the group while preserving its position in the SQL.
        let suffix_start = self.events.len().checked_sub(3);
        if let Some(start) = suffix_start
            && matches!(
                self.events.get(start..),
                Some([
                    LayoutEvent::Space,
                    LayoutEvent::Comment {
                        line_comment: true,
                        ..
                    },
                    LayoutEvent::Line(LineType::Hard)
                ])
            )
        {
            self.events.insert(start, LayoutEvent::GroupEnd);
            return;
        }
        self.group_end();
    }

    pub(crate) fn indent_start(&mut self) {
        self.events.push(LayoutEvent::IndentStart);
    }

    pub(crate) fn indent_end(&mut self) {
        self.events.push(LayoutEvent::IndentEnd);
    }

    pub(crate) fn align_start(&mut self) {
        self.events.push(LayoutEvent::AlignStart);
    }

    pub(crate) fn align_end(&mut self) {
        self.events.push(LayoutEvent::AlignEnd);
    }
}
