use std::collections::HashMap;

pub use crate::codegen::group_kind::GroupKind;
pub use crate::codegen::token_kind::TokenKind;
use crate::Comment;

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
        }
    }

    pub fn token(&mut self, token: TokenKind) {
        self.events.push(LayoutEvent::Token(token));
    }

    pub fn space(&mut self) {
        self.events.push(LayoutEvent::Space);
    }

    pub fn line(&mut self, line_type: LineType) {
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
