mod codegen;

pub use crate::codegen::token_kind::TokenKind;

use std::collections::VecDeque;

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
    GroupStart {
        id: Option<String>,
        break_parent: bool,
    },
    GroupEnd,
    IndentStart,
    IndentEnd,
}

pub struct EventEmitter {
    pub events: VecDeque<LayoutEvent>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    /// Helper methods for emitting events
    pub fn token(&mut self, token: TokenKind) {
        self.events.push_back(LayoutEvent::Token(token));
    }

    pub fn space(&mut self) {
        self.events.push_back(LayoutEvent::Space);
    }

    pub fn line(&mut self, line_type: LineType) {
        self.events.push_back(LayoutEvent::Line(line_type));
    }

    pub fn group_start(&mut self, id: Option<String>, break_parent: bool) {
        self.events
            .push_back(LayoutEvent::GroupStart { id, break_parent });
    }

    pub fn group_end(&mut self) {
        self.events.push_back(LayoutEvent::GroupEnd);
    }

    pub fn indent_start(&mut self) {
        self.events.push_back(LayoutEvent::IndentStart);
    }

    pub fn indent_end(&mut self) {
        self.events.push_back(LayoutEvent::IndentEnd);
    }
}
