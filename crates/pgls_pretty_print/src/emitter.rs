use std::collections::HashMap;

use crate::Comment;
pub use crate::codegen::group_kind::GroupKind;
pub use crate::codegen::token_kind::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum LineType {
    /// Must break (semicolon, etc.)
    Hard,
    /// Break if group doesn't fit
    Soft,
    /// Break if group doesn't fit, but collapse to space if it does
    SoftOrSpace,
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

#[derive(Debug, Default)]
pub struct EventEmitter {
    pub events: Vec<LayoutEvent>,
    /// Comments still waiting to be emitted before a node, by source location.
    /// Entries are removed as they are emitted so that a comment cannot be printed twice, and so
    /// that the caller can check the map is empty afterwards.
    leading_comments: HashMap<i32, Vec<Comment>>,
    /// Comments still waiting to be emitted after a node, by source location.
    trailing_comments: HashMap<i32, Vec<Comment>>,
    /// Boolean operands need standalone line comments to start their own line, otherwise a
    /// comment can join the preceding operand and drift on the next formatting pass.
    leading_line_comments_require_break: bool,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_comments(
        leading_comments: HashMap<i32, Vec<Comment>>,
        trailing_comments: HashMap<i32, Vec<Comment>>,
    ) -> Self {
        Self {
            events: Vec::new(),
            leading_comments,
            trailing_comments,
            leading_line_comments_require_break: false,
        }
    }

    pub fn token(&mut self, token: TokenKind) {
        self.events.push(LayoutEvent::Token(token));
    }

    pub fn space(&mut self) {
        self.events.push(LayoutEvent::Space);
    }

    pub fn line(&mut self, line_type: LineType) {
        if let Some(LayoutEvent::Line(previous)) = self.events.last_mut()
            && (matches!(&*previous, LineType::Hard) || matches!(&line_type, LineType::Hard))
        {
            *previous = LineType::Hard;
            return;
        }

        self.events.push(LayoutEvent::Line(line_type));
    }

    pub fn comment(&mut self, text: String, line_comment: bool) {
        self.events
            .push(LayoutEvent::Comment { text, line_comment });
    }

    /// Emits and consumes comments that precede `location`, if any.
    pub fn take_leading_comments_at(&mut self, location: i32) {
        let Some(comments) = self.leading_comments.remove(&location) else {
            return;
        };

        for comment in comments {
            let line_comment = comment.line_comment;
            if line_comment && self.leading_line_comments_require_break {
                self.force_current_line_break();
            }
            self.comment(comment.text, line_comment);
            if line_comment {
                self.line(LineType::Hard);
            } else {
                self.space();
            }
        }
    }

    /// Emits and consumes comments that follow `location`, if any.
    pub fn take_trailing_comments_at(&mut self, location: i32) {
        let Some(comments) = self.trailing_comments.remove(&location) else {
            return;
        };

        for comment in comments {
            let line_comment = comment.line_comment;
            self.space();
            self.comment(comment.text, line_comment);
            if line_comment {
                self.line(LineType::Hard);
            } else {
                self.space();
            }
        }
    }

    pub fn pending_comments(&self) -> usize {
        self.leading_comments.values().map(Vec::len).sum::<usize>()
            + self.trailing_comments.values().map(Vec::len).sum::<usize>()
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

    pub fn indent_start(&mut self) {
        self.events.push(LayoutEvent::IndentStart);
    }

    pub fn indent_end(&mut self) {
        self.events.push(LayoutEvent::IndentEnd);
    }
}
