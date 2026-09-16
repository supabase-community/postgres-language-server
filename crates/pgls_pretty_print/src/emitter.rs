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
}

impl EventEmitter {
    pub fn new() -> Self {
        Self::default()
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
