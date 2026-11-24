/// A line break of the LSP: `\n`, `\r\n` or a lone `\r`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineBreak {
    /// `\n`
    Lf,
    /// `\r\n`
    CrLf,
    /// A lone `\r`
    Cr,
}

impl LineBreak {
    /// The characters a line break starts with.
    pub(crate) const START_CHARS: [char; 2] = ['\n', '\r'];

    /// The line break at the start of `text`, `None` if `text` doesn't start with one.
    pub fn at_start_of(text: &str) -> Option<Self> {
        if text.starts_with("\r\n") {
            Some(Self::CrLf)
        } else if text.starts_with('\r') {
            Some(Self::Cr)
        } else if text.starts_with('\n') {
            Some(Self::Lf)
        } else {
            None
        }
    }

    /// The first line break of `text`, e.g. to write new lines in the style of a file. `None` if
    /// `text` has none.
    pub fn first_in(text: &str) -> Option<Self> {
        let start = text.find(Self::START_CHARS)?;
        Self::at_start_of(text.get(start..)?)
    }

    /// Whether `text` contains a line break.
    pub fn is_in(text: &str) -> bool {
        text.contains(Self::START_CHARS)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
            Self::Cr => "\r",
        }
    }
}
