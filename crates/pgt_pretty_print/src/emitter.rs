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
    GroupStart { kind: GroupKind },
    GroupEnd,
    IndentStart,
    IndentEnd,
}

pub struct EventEmitter {
    pub events: Vec<LayoutEvent>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self { events: Vec::new() }
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

    pub fn group_start(&mut self, kind: GroupKind) {
        self.events.push(LayoutEvent::GroupStart { kind });
    }

    pub fn is_within_group(&self, target_kind: GroupKind) -> bool {
        let mut depth = 0;
        for event in self.events.iter().rev() {
            match event {
                LayoutEvent::GroupEnd => depth += 1,
                LayoutEvent::GroupStart { kind, .. } => {
                    if depth == 0 && *kind == target_kind {
                        return true;
                    }
                    if depth > 0 {
                        depth -= 1;
                    }
                }
                _ => {}
            }
        }
        false
    }

    pub fn parent_group(&self) -> Option<GroupKind> {
        let mut depth = 0;
        for event in self.events.iter().rev() {
            match event {
                LayoutEvent::GroupEnd => depth += 1,
                LayoutEvent::GroupStart { kind, .. } => {
                    if depth == 1 {
                        return Some(kind.clone());
                    }
                    if depth > 0 {
                        depth -= 1;
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn is_top_level(&self) -> bool {
        let mut depth = 0;
        for event in &self.events {
            match event {
                LayoutEvent::GroupStart { .. } => depth += 1,
                LayoutEvent::GroupEnd => depth -= 1,
                _ => {}
            }
        }
        depth == 1
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

pub trait ToTokens {
    fn to_tokens(&self, emitter: &mut EventEmitter);
}
