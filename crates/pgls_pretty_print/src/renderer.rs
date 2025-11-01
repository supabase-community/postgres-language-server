use crate::emitter::{LayoutEvent, LineType};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub enum IndentStyle {
    Spaces,
    Tabs,
}

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub max_line_length: usize,
    pub indent_size: usize,
    pub indent_style: IndentStyle,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            max_line_length: 80,
            indent_size: 2,
            indent_style: IndentStyle::Spaces,
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
                    let token_text = token.render();
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
                    assert!(false, "Unmatched group end");
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

    fn render_events_with_breaks(&mut self, events: &[LayoutEvent]) -> Result<(), std::fmt::Error> {
        let mut i = 0;
        while i < events.len() {
            match &events[i] {
                LayoutEvent::Token(token) => {
                    let text = token.render();
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
                    let inner_events = &events[i + 1..group_end]; // skip GroupStart/GroupEnd
                    self.render_events_with_breaks(inner_events)?;
                    i = group_end + 1;
                }
                LayoutEvent::GroupEnd => {
                    assert!(false, "Unmatched group end");
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
                    let text = token.render();
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
        for i in start..events.len() {
            match &events[i] {
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

        write!(self.writer, "{}", text)?;
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
        write!(self.writer, "{}", indent_str)?;
        Ok(())
    }
}
