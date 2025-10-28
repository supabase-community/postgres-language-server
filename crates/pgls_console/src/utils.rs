use termcolor::NoColor;

use crate::fmt::{Display, Formatter, Termcolor};
use crate::{Markup, markup};
use std::io;

/// Adapter type providing a std::fmt::Display implementation for any type that
/// implements pgls_console::fmt::Display.
pub struct StdDisplay<T: Display>(pub T);

impl<T> std::fmt::Display for StdDisplay<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer: Vec<u8> = Vec::new();
        let mut termcolor = Termcolor(NoColor::new(&mut buffer));
        let mut formatter = Formatter::new(&mut termcolor);

        self.0.fmt(&mut formatter).map_err(|_| std::fmt::Error)?;

        let content = String::from_utf8(buffer).map_err(|_| std::fmt::Error)?;

        f.write_str(content.as_str())
    }
}

/// It displays a type that implements [std::fmt::Display]
pub struct DebugDisplay<T>(pub T);

impl<T> Display for DebugDisplay<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> io::Result<()> {
        write!(f, "{:?}", self.0)
    }
}

/// It displays a `Option<T>`, where `T` implements [std::fmt::Display]
pub struct DebugDisplayOption<T>(pub Option<T>);

impl<T> Display for DebugDisplayOption<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut Formatter) -> io::Result<()> {
        use crate as pgls_console;

        match &self.0 {
            Some(value) => {
                markup!({ DebugDisplay(value) }).fmt(fmt)?;
            }
            _ => {
                markup!(<Dim>"unset"</Dim>).fmt(fmt)?;
            }
        }
        Ok(())
    }
}

/// A horizontal line with the given print width
pub struct HorizontalLine {
    width: usize,
}

impl HorizontalLine {
    pub fn new(width: usize) -> Self {
        Self { width }
    }
}

impl Display for HorizontalLine {
    fn fmt(&self, fmt: &mut Formatter) -> io::Result<()> {
        fmt.write_str(&"\u{2501}".repeat(self.width))
    }
}

// It prints `\n`
pub struct Softline;

pub const SOFT_LINE: Softline = Softline;

impl Display for Softline {
    fn fmt(&self, fmt: &mut Formatter) -> io::Result<()> {
        fmt.write_str("\n")
    }
}

// It prints `\n\n`
pub struct Hardline;

pub const HARD_LINE: Hardline = Hardline;

impl Display for Hardline {
    fn fmt(&self, fmt: &mut Formatter) -> io::Result<()> {
        fmt.write_str("\n\n")
    }
}

/// It prints N whitespaces, where N is the `width` provided by [Padding::new]
pub struct Padding {
    width: usize,
}

impl Padding {
    pub fn new(width: usize) -> Self {
        Self { width }
    }
}

impl Display for Padding {
    fn fmt(&self, fmt: &mut Formatter) -> io::Result<()> {
        for _ in 0..self.width {
            fmt.write_str(" ")?;
        }
        Ok(())
    }
}

/// It writes a pair of key-value, with the given padding
pub struct KeyValuePair<'a>(pub &'a str, pub Markup<'a>);

impl Display for KeyValuePair<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> io::Result<()> {
        let KeyValuePair(key, value) = self;
        write!(fmt, "  {key}:")?;

        let padding_width = 30usize.saturating_sub(key.len() + 1);

        for _ in 0..padding_width {
            fmt.write_str(" ")?;
        }

        value.fmt(fmt)?;

        fmt.write_str("\n")
    }
}
