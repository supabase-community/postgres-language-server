use crate::TokenKind;
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
    /// Put the terminating semicolon on its own line when the statement spans several lines.
    pub isolate_semicolon: bool,
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
            isolate_semicolon: false,
        }
    }
}

/// The column continuation lines align with, from `AlignStart`.
struct Alignment {
    column: usize,
    /// The indent level when the alignment started.
    indent_level: usize,
}

pub struct Renderer<W: Write> {
    config: RenderConfig,
    writer: W,
    current_line_length: usize,
    indent_level: usize,
    /// Open alignments, innermost last.
    alignments: Vec<Alignment>,
    at_line_start: bool,
    /// A space written after the last text. It is only written out once more text follows on the
    /// same line, so a line never ends in whitespace.
    pending_space: bool,
}

impl<W: Write> Renderer<W> {
    pub fn new(writer: W, config: RenderConfig) -> Self {
        Self {
            config,
            writer,
            current_line_length: 0,
            indent_level: 0,
            alignments: Vec::new(),
            at_line_start: true,
            pending_space: false,
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
                LayoutEvent::Comment { text, .. } => {
                    self.write_text(text)?;
                    i += 1;
                }
                LayoutEvent::GroupStart { .. } => {
                    let group_end = self.find_group_end(events, i);
                    let trailing_width = self.trailing_width(&events[group_end + 1..], 0);
                    self.render_group(&events[i..=group_end], trailing_width)?;
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
                LayoutEvent::AlignStart => {
                    self.start_alignment();
                    i += 1;
                }
                LayoutEvent::AlignEnd => {
                    self.alignments.pop();
                    i += 1;
                }
            }
        }
        Ok(())
    }

    /// Renders a group on one line when it fits, `trailing_width` included, and breaks it
    /// otherwise. A nested group decides on its own, whether or not its parent broke.
    fn render_group(
        &mut self,
        group_events: &[LayoutEvent],
        trailing_width: usize,
    ) -> Result<(), std::fmt::Error> {
        if let Some(single_line) = self.single_line_if_fits(group_events, trailing_width) {
            // A space that ends the group waits for the next text, like any other space.
            let text = single_line.trim_end_matches(' ');
            if !text.is_empty() {
                self.write_text(text)?;
            }
            if text.len() < single_line.len() {
                self.write_space()?;
            }
            return Ok(());
        }

        // Skip the GroupStart and GroupEnd markers.
        self.render_events_with_breaks(&group_events[1..group_events.len() - 1], trailing_width)
    }

    /// The width of the text that follows a group without a break opportunity in between, such as
    /// the comma after a list item. `outer_trailing_width` is what follows the enclosing events.
    fn trailing_width(&self, events: &[LayoutEvent], outer_trailing_width: usize) -> usize {
        let mut width = 0;
        for event in events {
            match event {
                LayoutEvent::Token(token) => {
                    let text = token.render(&self.config);
                    match text.split_once('\n') {
                        Some((first_line, _)) => return width + first_line.len(),
                        None => width += text.len(),
                    }
                }
                LayoutEvent::Space => width += 1,
                LayoutEvent::GroupStart { .. }
                | LayoutEvent::GroupEnd
                | LayoutEvent::IndentStart
                | LayoutEvent::IndentEnd
                | LayoutEvent::AlignStart
                | LayoutEvent::AlignEnd => {}
                LayoutEvent::Line(_) | LayoutEvent::Comment { .. } => return width,
            }
        }
        width + outer_trailing_width
    }

    /// The current line length, counting the indentation that will be written when the
    /// next text starts a line. `current_line_length` is reset to zero by a line break
    /// and only grows with the indent when text is actually written, so width checks
    /// made right after a break must account for the pending indentation themselves.
    fn effective_line_length(&self) -> usize {
        if self.at_line_start {
            self.indentation_width()
        } else {
            self.current_line_length
        }
    }

    /// Continuation lines start at the column of the innermost alignment, plus one indent for
    /// every level opened since. Without an alignment they start at the indent level.
    fn indentation_width(&self) -> usize {
        let indent = self.indent_level * self.config.indent_size;
        match self.alignments.last() {
            Some(alignment) => (alignment.column + indent)
                .saturating_sub(alignment.indent_level * self.config.indent_size),
            None => indent,
        }
    }

    /// The column is kept with the indent level it was taken at, not as an offset from that
    /// level's indent: the text on the line can start left of it, `select case` with the select
    /// list already one level in for instance.
    fn start_alignment(&mut self) {
        self.alignments.push(Alignment {
            column: self.effective_line_length(),
            indent_level: self.indent_level,
        });
    }

    fn render_events_with_breaks(
        &mut self,
        events: &[LayoutEvent],
        trailing_width: usize,
    ) -> Result<(), std::fmt::Error> {
        let mut i = 0;
        while i < events.len() {
            match &events[i] {
                LayoutEvent::Token(token) => {
                    // This path is only taken when the enclosing group broke, so the statement
                    // already spans several lines and the terminator can stand alone. A statement
                    // that fits is rendered by single_line_if_fits, which never reaches this code, and
                    // therefore keeps its semicolon attached.
                    if self.config.isolate_semicolon && matches!(token, TokenKind::SEMICOLON) {
                        self.write_line_break()?;
                    }

                    let text = token.render(&self.config);
                    self.write_text(&text)?;
                    i += 1;
                }
                LayoutEvent::Space => {
                    self.write_space()?;
                    i += 1;
                }
                LayoutEvent::Line(line_type) => {
                    match line_type {
                        LineType::Fill
                            if self.next_fill_item_fits(events, i + 1, 1, trailing_width) =>
                        {
                            self.write_space()?;
                        }
                        LineType::FillNoSpace
                            if self.next_fill_item_fits(events, i + 1, 0, trailing_width) => {}
                        _ => self.write_line_break()?,
                    }
                    i += 1;
                }
                LayoutEvent::Comment { text, .. } => {
                    self.write_text(text)?;
                    i += 1;
                }
                LayoutEvent::GroupStart { .. } => {
                    let group_end = self.find_group_end(events, i);
                    let group_trailing_width =
                        self.trailing_width(&events[group_end + 1..], trailing_width);
                    self.render_group(&events[i..=group_end], group_trailing_width)?;
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
                LayoutEvent::AlignStart => {
                    self.start_alignment();
                    i += 1;
                }
                LayoutEvent::AlignEnd => {
                    self.alignments.pop();
                    i += 1;
                }
            }
        }
        Ok(())
    }

    /// The group on one line, if it can be collapsed and fits on the current line followed by
    /// `trailing_width` columns that cannot move to the next line. Only the first line of a text
    /// spanning several lines, a function body for instance, sits on the current one.
    ///
    /// Measuring stops as soon as the line is full, so a large group that cannot fit is not
    /// rendered in full for every level it is nested in.
    fn single_line_if_fits(
        &self,
        group_events: &[LayoutEvent],
        trailing_width: usize,
    ) -> Option<String> {
        let available = self
            .config
            .max_line_length
            .saturating_sub(self.effective_line_length());
        let mut buffer = String::new();
        // Set once the text reaches a line break, after which the width no longer matters.
        let mut first_line_fits = false;

        for event in group_events {
            match event {
                LayoutEvent::Token(token) => buffer.push_str(&token.render(&self.config)),
                LayoutEvent::Space
                | LayoutEvent::Line(LineType::SoftOrSpace)
                | LayoutEvent::Line(LineType::Fill) => buffer.push(' '),
                // Collapsing would push code behind a `--` comment or flatten a multi-line block
                // comment into the surrounding expression.
                LayoutEvent::Line(LineType::Hard) => return None,
                LayoutEvent::Comment { text, line_comment } => {
                    if *line_comment || text.contains('\n') {
                        return None;
                    }
                    buffer.push_str(text);
                }
                LayoutEvent::Line(LineType::Soft | LineType::FillNoSpace)
                | LayoutEvent::GroupStart { .. }
                | LayoutEvent::GroupEnd
                | LayoutEvent::IndentStart
                | LayoutEvent::IndentEnd
                | LayoutEvent::AlignStart
                | LayoutEvent::AlignEnd => {}
            }

            if !first_line_fits {
                match buffer.split_once('\n') {
                    Some((first_line, _)) if first_line.len() <= available => {
                        first_line_fits = true;
                    }
                    Some(_) => return None,
                    None if buffer.len() + trailing_width > available => return None,
                    None => {}
                }
            }
        }

        Some(buffer)
    }

    fn handle_line(&mut self, line_type: &LineType) -> Result<(), std::fmt::Error> {
        match line_type {
            LineType::Hard => {
                self.write_line_break()?;
            }
            LineType::Soft | LineType::SoftOrSpace | LineType::Fill => {
                // For now, just treat as space outside groups
                self.write_space()?;
            }
            LineType::FillNoSpace => {}
        }
        Ok(())
    }

    /// Whether the next item of a fill list fits on the current line. The last item is followed by
    /// `trailing_width`, the text after the list that cannot move to the next line, such as the
    /// semicolon closing the statement.
    fn next_fill_item_fits(
        &self,
        events: &[LayoutEvent],
        start: usize,
        separator_width: usize,
        trailing_width: usize,
    ) -> bool {
        let Some((item_length, is_last)) = self.next_fill_item_length(events, start) else {
            return false;
        };
        let trailing_width = if is_last { trailing_width } else { 0 };

        self.current_line_length + separator_width + item_length + trailing_width
            <= self.config.max_line_length
    }

    /// The length of the fill item starting at `start`, and whether it runs to the end of
    /// `events`, which makes it the last item.
    fn next_fill_item_length(&self, events: &[LayoutEvent], start: usize) -> Option<(usize, bool)> {
        let mut length = 0;
        let mut depth = 0;

        for event in events.iter().skip(start) {
            match event {
                LayoutEvent::Token(token) => {
                    length += token.render(&self.config).len();
                }
                LayoutEvent::Space => length += 1,
                LayoutEvent::Line(LineType::Hard) => return None,
                LayoutEvent::Line(LineType::Soft) => {}
                LayoutEvent::Line(LineType::SoftOrSpace) => length += 1,
                LayoutEvent::Line(LineType::Fill | LineType::FillNoSpace) if depth == 0 => {
                    return Some((length, false));
                }
                LayoutEvent::Line(LineType::Fill) => length += 1,
                LayoutEvent::Line(LineType::FillNoSpace) => {}
                LayoutEvent::Comment { text, line_comment } => {
                    if *line_comment {
                        return None;
                    }
                    length += text.len();
                }
                LayoutEvent::GroupStart { .. } => depth += 1,
                LayoutEvent::GroupEnd if depth == 0 => return Some((length, false)),
                LayoutEvent::GroupEnd => depth -= 1,
                LayoutEvent::IndentStart
                | LayoutEvent::IndentEnd
                | LayoutEvent::AlignStart
                | LayoutEvent::AlignEnd => {}
            }
        }

        Some((length, true))
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
        } else if std::mem::take(&mut self.pending_space) {
            write!(self.writer, " ")?;
        }

        write!(self.writer, "{text}")?;
        if let Some((_, last_line)) = text.rsplit_once('\n') {
            self.current_line_length = last_line.len();
            self.at_line_start = last_line.is_empty();
        } else {
            self.current_line_length += text.len();
        }
        Ok(())
    }

    /// Two spaces in a row, from a space next to a line that stayed flat for instance, are
    /// written as one.
    fn write_space(&mut self) -> Result<(), std::fmt::Error> {
        if !self.at_line_start && !self.pending_space {
            self.pending_space = true;
            self.current_line_length += 1;
        }
        Ok(())
    }

    /// Ends the current line. A line that has no text yet is not ended again: two breaks in a
    /// row, around an empty list for instance, would otherwise print a blank line.
    fn write_line_break(&mut self) -> Result<(), std::fmt::Error> {
        self.pending_space = false;
        if self.at_line_start {
            return Ok(());
        }
        writeln!(self.writer)?;
        self.current_line_length = 0;
        self.at_line_start = true;
        Ok(())
    }

    fn write_indentation(&mut self) -> Result<(), std::fmt::Error> {
        let indent_str = match self.config.indent_style {
            IndentStyle::Spaces => {
                let spaces = " ".repeat(self.indentation_width());
                self.current_line_length += spaces.len();
                spaces
            }
            // One tab per indent level and spaces for the rest of an alignment. The columns only
            // line up when the editor's tab width equals `indent_size`.
            IndentStyle::Tabs => {
                let width = self.indentation_width();
                let tabs = match self.config.indent_size {
                    0 => self.indent_level,
                    size => self.indent_level.min(width / size),
                };
                let spaces = width - tabs * self.config.indent_size;
                self.current_line_length += width;
                "\t".repeat(tabs) + &" ".repeat(spaces)
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
    use crate::emitter::{EventEmitter, GroupKind};

    fn render_events(events: Vec<LayoutEvent>, config: RenderConfig) -> String {
        let mut output = String::new();
        let mut renderer = Renderer::new(&mut output, config);
        renderer.render(events).unwrap();
        output
    }

    #[test]
    fn test_keyword_case_upper() {
        let mut emitter = EventEmitter::new(crate::FormatConfig::default());
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
        let mut emitter = EventEmitter::new(crate::FormatConfig::default());
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
        let mut emitter = EventEmitter::new(crate::FormatConfig::default());
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
        let mut emitter = EventEmitter::new(crate::FormatConfig::default());
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
        let mut emitter = EventEmitter::new(crate::FormatConfig::default());
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

    #[test]
    fn a_block_comment_is_rendered_inline() {
        let mut emitter = EventEmitter::new(Default::default());
        emitter.token(TokenKind::SELECT_KW);
        emitter.space();
        emitter.comment("/* why */".to_string(), false);
        emitter.space();
        emitter.token(TokenKind::INT_NUMBER(1));

        let output = render_events(emitter.events, RenderConfig::default());
        assert_eq!(output, "select /* why */ 1");
    }

    #[test]
    fn a_line_comment_forces_the_group_to_break() {
        let mut emitter = EventEmitter::new(Default::default());
        emitter.group_start(crate::emitter::GroupKind::SelectStmt);
        emitter.token(TokenKind::SELECT_KW);
        emitter.space();
        emitter.comment("-- why".to_string(), true);
        emitter.line(crate::emitter::LineType::SoftOrSpace);
        emitter.token(TokenKind::INT_NUMBER(1));
        emitter.group_end();

        let output = render_events(emitter.events, RenderConfig::default());
        assert_eq!(output, "select -- why\n1");
    }

    #[test]
    fn a_broken_statement_gets_its_semicolon_on_its_own_line() {
        let mut emitter = EventEmitter::new(crate::FormatConfig::default());
        emitter.group_start(GroupKind::SelectStmt);
        emitter.token(TokenKind::SELECT_KW);
        emitter.line(crate::emitter::LineType::Hard);
        emitter.token(TokenKind::INT_NUMBER(1));
        emitter.token(TokenKind::SEMICOLON);
        emitter.group_end();

        let config = RenderConfig {
            isolate_semicolon: true,
            ..Default::default()
        };

        let output = render_events(emitter.events, config);
        assert_eq!(output, "select\n1\n;");
    }

    #[test]
    fn a_single_line_statement_keeps_its_semicolon_attached() {
        let mut emitter = EventEmitter::new(crate::FormatConfig::default());
        emitter.token(TokenKind::SELECT_KW);
        emitter.space();
        emitter.token(TokenKind::INT_NUMBER(1));
        emitter.token(TokenKind::SEMICOLON);

        let config = RenderConfig {
            isolate_semicolon: true,
            ..Default::default()
        };

        let output = render_events(emitter.events, config);
        assert_eq!(output, "select 1;");
    }
}
