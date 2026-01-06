use crate::emitter::{LayoutEvent, LineType};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub enum IndentStyle {
    Spaces,
    Tabs,
}

#[derive(Debug, Clone, Default)]
pub enum KeywordCase {
    Upper,
    #[default]
    Lower,
}

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub max_line_length: usize,
    pub indent_size: usize,
    pub indent_style: IndentStyle,
    /// Casing for SQL keywords (SELECT, FROM, WHERE, etc.)
    pub keyword_case: KeywordCase,
    /// Casing for constants (NULL, TRUE, FALSE)
    pub constant_case: KeywordCase,
    /// Casing for data types (text, varchar, int, etc.)
    pub type_case: KeywordCase,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            max_line_length: 100,
            indent_size: 2,
            indent_style: IndentStyle::Spaces,
            keyword_case: KeywordCase::default(),
            constant_case: KeywordCase::default(),
            type_case: KeywordCase::default(),
        }
    }
}

pub struct Renderer<W: Write> {
    config: RenderConfig,
    writer: W,
    current_line_length: usize,
    indent_level: usize,
    at_line_start: bool,
}

impl<W: Write> Renderer<W> {
    pub fn new(writer: W, config: RenderConfig) -> Self {
        Self {
            config,
            writer,
            current_line_length: 0,
            indent_level: 0,
            at_line_start: true,
        }
    }

    pub fn render(&mut self, events: Vec<LayoutEvent>) -> Result<(), std::fmt::Error> {
        self.render_events(&events)
    }

    fn render_events(&mut self, events: &[LayoutEvent]) -> Result<(), std::fmt::Error> {
        let mut i = 0;
        while i < events.len() {
            match &events[i] {
                LayoutEvent::Token(token) => {
                    let token_text = token.render(&self.config);
                    self.write_text(&token_text)?;
                    i += 1;
                }
                LayoutEvent::Space => {
                    self.write_space()?;
                    i += 1;
                }
                LayoutEvent::Line(line_type) => {
                    self.handle_line(line_type)?;
                    i += 1;
                }
                LayoutEvent::GroupStart { .. } => {
                    let group_end = self.find_group_end(events, i);
                    let group_slice = &events[i..=group_end];
                    self.render_group(group_slice)?;
                    i = group_end + 1;
                }
                LayoutEvent::GroupEnd => {
                    unreachable!("Unmatched group end");
                }
                LayoutEvent::IndentStart => {
                    self.indent_level += 1;
                    i += 1;
                }
                LayoutEvent::IndentEnd => {
                    self.indent_level = self.indent_level.saturating_sub(1);
                    i += 1;
                }
            }
        }
        Ok(())
    }

    fn render_group(&mut self, group_events: &[LayoutEvent]) -> Result<(), std::fmt::Error> {
        if let Some(single_line) = self.try_single_line(group_events) {
            let would_fit =
                self.current_line_length + single_line.len() <= self.config.max_line_length;
            if would_fit {
                self.write_text(&single_line)?;
                return Ok(());
            }
        }

        self.render_events_with_breaks(group_events)
    }

    /// Render an inner group, allowing it to try single-line independently of parent's break status.
    /// This enables nested groups to stay compact even when the parent group is breaking.
    fn render_inner_group(&mut self, group_events: &[LayoutEvent]) -> Result<(), std::fmt::Error> {
        // Extract inner events (skip GroupStart and GroupEnd markers)
        let inner_events = &group_events[1..group_events.len() - 1];

        // Try single-line first, independent of parent's break status
        if let Some(single_line) = self.try_single_line(group_events) {
            let would_fit =
                self.current_line_length + single_line.len() <= self.config.max_line_length;
            if would_fit {
                self.write_text(&single_line)?;
                return Ok(());
            }
        }

        // Fall back to breaking
        self.render_events_with_breaks(inner_events)
    }

    fn render_events_with_breaks(&mut self, events: &[LayoutEvent]) -> Result<(), std::fmt::Error> {
        let mut i = 0;
        while i < events.len() {
            match &events[i] {
                LayoutEvent::Token(token) => {
                    let text = token.render(&self.config);
                    self.write_text(&text)?;
                    i += 1;
                }
                LayoutEvent::Space => {
                    self.write_space()?;
                    i += 1;
                }
                LayoutEvent::Line(_) => {
                    self.write_line_break()?;
                    i += 1;
                }
                LayoutEvent::GroupStart { .. } => {
                    let group_end = self.find_group_end(events, i);
                    let group_slice = &events[i..=group_end];

                    // Decide whether to try single-line for this nested group.
                    // We try single-line in two cases:
                    // 1. Followed by a Hard break (semantic boundary like RETURNS after
                    //    function signature)
                    // 2. The group's single-line form is small enough to fit on its own
                    //    (allows compact column lists, function args, etc.)
                    let followed_by_hard_break = events
                        .get(group_end + 1)
                        .is_some_and(|e| matches!(e, LayoutEvent::Line(LineType::Hard)));

                    let try_single_line = if followed_by_hard_break {
                        true
                    } else if let Some(single_line) = self.try_single_line(group_slice) {
                        // Allow small groups to stay on one line even without Hard break
                        // This keeps column lists, function args compact
                        single_line.len() <= self.config.max_line_length / 2
                    } else {
                        false
                    };

                    if try_single_line {
                        self.render_inner_group(group_slice)?;
                    } else {
                        // Force nested group to break
                        let inner_events = &events[i + 1..group_end];
                        self.render_events_with_breaks(inner_events)?;
                    }
                    i = group_end + 1;
                }
                LayoutEvent::GroupEnd => {
                    unreachable!("Unmatched group end");
                }
                LayoutEvent::IndentStart => {
                    self.indent_level += 1;
                    i += 1;
                }
                LayoutEvent::IndentEnd => {
                    self.indent_level = self.indent_level.saturating_sub(1);
                    i += 1;
                }
            }
        }
        Ok(())
    }

    fn try_single_line(&self, group_events: &[LayoutEvent]) -> Option<String> {
        let mut buffer = String::new();
        let mut has_hard_breaks = false;

        for event in group_events {
            match event {
                LayoutEvent::Token(token) => {
                    let text = token.render(&self.config);
                    buffer.push_str(&text);
                }
                LayoutEvent::Space => {
                    buffer.push(' ');
                }
                LayoutEvent::Line(LineType::Hard) => {
                    has_hard_breaks = true;
                    break;
                }
                LayoutEvent::Line(LineType::Soft) => {
                    // soft lines disappear in single-line mode (no space)
                }
                LayoutEvent::Line(LineType::SoftOrSpace) => {
                    buffer.push(' '); // Becomes space in single-line mode
                }
                LayoutEvent::GroupStart { .. } | LayoutEvent::GroupEnd => {
                    // skip group markers for single line test
                }
                LayoutEvent::IndentStart | LayoutEvent::IndentEnd => {
                    // skip indent changes for single line test
                }
            }
        }

        if has_hard_breaks { None } else { Some(buffer) }
    }

    fn handle_line(&mut self, line_type: &LineType) -> Result<(), std::fmt::Error> {
        match line_type {
            LineType::Hard => {
                self.write_line_break()?;
            }
            LineType::Soft | LineType::SoftOrSpace => {
                // For now, just treat as space outside groups
                self.write_space()?;
            }
        }
        Ok(())
    }

    fn find_group_end(&self, events: &[LayoutEvent], start: usize) -> usize {
        let mut depth = 0;
        for (i, event) in events.iter().enumerate().skip(start) {
            match event {
                LayoutEvent::GroupStart { .. } => depth += 1,
                LayoutEvent::GroupEnd => {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
        panic!("Unmatched group start");
    }

    fn write_text(&mut self, text: &str) -> Result<(), std::fmt::Error> {
        if self.at_line_start {
            self.write_indentation()?;
            self.at_line_start = false;
        }

        write!(self.writer, "{text}")?;
        self.current_line_length += text.len();
        Ok(())
    }

    fn write_space(&mut self) -> Result<(), std::fmt::Error> {
        if !self.at_line_start {
            write!(self.writer, " ")?;
            self.current_line_length += 1;
        }
        Ok(())
    }

    fn write_line_break(&mut self) -> Result<(), std::fmt::Error> {
        writeln!(self.writer)?;
        self.current_line_length = 0;
        self.at_line_start = true;
        Ok(())
    }

    fn write_indentation(&mut self) -> Result<(), std::fmt::Error> {
        let indent_str = match self.config.indent_style {
            IndentStyle::Spaces => {
                let spaces = " ".repeat(self.indent_level * self.config.indent_size);
                self.current_line_length += spaces.len();
                spaces
            }
            IndentStyle::Tabs => {
                let tabs = "\t".repeat(self.indent_level);
                self.current_line_length += self.indent_level * self.config.indent_size; // Approximate
                tabs
            }
        };
        write!(self.writer, "{indent_str}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::token_kind::TokenKind;
    use crate::emitter::EventEmitter;

    fn render_events(events: Vec<LayoutEvent>, config: RenderConfig) -> String {
        let mut output = String::new();
        let mut renderer = Renderer::new(&mut output, config);
        renderer.render(events).unwrap();
        output
    }

    #[test]
    fn test_keyword_case_upper() {
        let mut emitter = EventEmitter::new();
        emitter.token(TokenKind::SELECT_KW);
        emitter.space();
        emitter.token(TokenKind::INT_NUMBER(1));

        let config = RenderConfig {
            keyword_case: KeywordCase::Upper,
            constant_case: KeywordCase::Lower,
            ..Default::default()
        };

        let output = render_events(emitter.events, config);
        assert_eq!(output, "SELECT 1");
    }

    #[test]
    fn test_keyword_case_lower() {
        let mut emitter = EventEmitter::new();
        emitter.token(TokenKind::SELECT_KW);
        emitter.space();
        emitter.token(TokenKind::INT_NUMBER(1));

        let config = RenderConfig {
            keyword_case: KeywordCase::Lower,
            constant_case: KeywordCase::Lower,
            ..Default::default()
        };

        let output = render_events(emitter.events, config);
        assert_eq!(output, "select 1");
    }

    #[test]
    fn test_constant_case_upper() {
        let mut emitter = EventEmitter::new();
        emitter.token(TokenKind::SELECT_KW);
        emitter.space();
        emitter.token(TokenKind::NULL);
        emitter.token(TokenKind::COMMA);
        emitter.space();
        emitter.token(TokenKind::BOOLEAN(true));
        emitter.token(TokenKind::COMMA);
        emitter.space();
        emitter.token(TokenKind::BOOLEAN(false));

        let config = RenderConfig {
            keyword_case: KeywordCase::Lower,
            constant_case: KeywordCase::Upper,
            ..Default::default()
        };

        let output = render_events(emitter.events, config);
        assert_eq!(output, "select NULL, TRUE, FALSE");
    }

    #[test]
    fn test_constant_case_lower() {
        let mut emitter = EventEmitter::new();
        emitter.token(TokenKind::SELECT_KW);
        emitter.space();
        emitter.token(TokenKind::NULL);
        emitter.token(TokenKind::COMMA);
        emitter.space();
        emitter.token(TokenKind::BOOLEAN(true));

        let config = RenderConfig {
            keyword_case: KeywordCase::Upper,
            constant_case: KeywordCase::Lower,
            ..Default::default()
        };

        let output = render_events(emitter.events, config);
        assert_eq!(output, "SELECT null, true");
    }

    #[test]
    fn test_mixed_case_settings() {
        let mut emitter = EventEmitter::new();
        emitter.token(TokenKind::SELECT_KW);
        emitter.space();
        emitter.token(TokenKind::INT_NUMBER(1));
        emitter.space();
        emitter.token(TokenKind::WHERE_KW);
        emitter.space();
        emitter.token(TokenKind::IDENT("name".to_string()));
        emitter.space();
        emitter.token(TokenKind::IS_KW);
        emitter.space();
        emitter.token(TokenKind::NOT_KW);
        emitter.space();
        emitter.token(TokenKind::NULL);
        emitter.space();
        emitter.token(TokenKind::AND_KW);
        emitter.space();
        emitter.token(TokenKind::IDENT("active".to_string()));
        emitter.space();
        emitter.token(TokenKind::IDENT("=".to_string()));
        emitter.space();
        emitter.token(TokenKind::BOOLEAN(true));

        // Upper keywords, lower constants
        let config = RenderConfig {
            keyword_case: KeywordCase::Upper,
            constant_case: KeywordCase::Lower,
            ..Default::default()
        };

        let output = render_events(emitter.events, config);
        assert_eq!(output, "SELECT 1 WHERE name IS NOT null AND active = true");
    }
}
