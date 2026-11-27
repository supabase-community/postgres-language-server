//! SQL files that a host fills in before Postgres sees them: format strings of a host language,
//! like Go's `fmt.Sprintf` and Rust's `format!`, and migrations with goose's `ENVSUB`.
//!
//! A reader of the host finds its [`Expansion`]s, and [`Preprocessed::substitute`] turns them into
//! the text that the analysis reads instead of the file as written. The readers of the template
//! syntaxes are [`Template::read`]; goose's `ENVSUB` is read by `pgls_migrations`, with the
//! migration tool of the file.

mod go;
mod rust;
mod substitute;
mod tagged;

use std::collections::HashMap;
use std::str::FromStr;

use pgls_lexer::SyntaxKind;
use pgls_text_size::TextRange;
use serde::{Deserialize, Serialize};

pub use tagged::Tagged;

// The value of `templating.syntax`: the doc comments are the documentation of the setting.
/// The format strings of a host language that the SQL files are.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum Template {
    /// The files are SQL as written (default).
    #[default]
    None,
    /// Go's `fmt.Sprintf`: `%s`, `%d`, `%[1]v`, `%5.2f`, and `%%` for `%`.
    GoPrintf,
    /// Rust's `format!`: `{}`, `{0}`, `{name}`, `{name:>8}`, and `{{` and `}}` for `{` and `}`.
    RustFmt,
}

impl FromStr for Template {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "go-printf" => Ok(Self::GoPrintf),
            "rust-fmt" => Ok(Self::RustFmt),
            _ => Err(
                "Value not supported for the template syntax. Use 'none', 'go-printf' or 'rust-fmt'.",
            ),
        }
    }
}

impl Template {
    /// The most directives of a file that get placeholders. The ones after them stay as written.
    pub const EXPANSIONS_MAX: usize = 100_000;

    /// The text that the analysis reads for `text`, with placeholders for the directives of the
    /// template, or `None` if it has none.
    ///
    /// Cost: one pass over the text, and [`Preprocessed::substitute`] if it has directives.
    pub fn read(self, text: &str) -> Option<Preprocessed> {
        let mut directives = self.directives(text);
        Self::name_arguments(&mut directives);
        (!directives.is_empty())
            .then(|| Preprocessed::substitute_reading(text, &directives, Self::is_ambiguous_after))
    }

    /// The directives of `text`, in text order, at most [`Template::EXPANSIONS_MAX`], without
    /// the names of their placeholders.
    fn directives(self, text: &str) -> Vec<Expansion<'_>> {
        let mut directives: Vec<Expansion> = match self {
            Self::None => return Vec::new(),
            Self::GoPrintf => go::Directives::new(text)
                .take(Self::EXPANSIONS_MAX + 1)
                .collect(),
            Self::RustFmt => rust::Directives::new(text)
                .take(Self::EXPANSIONS_MAX + 1)
                .collect(),
        };
        if directives.len() > Self::EXPANSIONS_MAX {
            tracing::warn!(
                "the file has more than {} template directives, the ones after them stay as written",
                Self::EXPANSIONS_MAX
            );
            directives.truncate(Self::EXPANSIONS_MAX);
        }
        directives
    }

    /// `text` as the host would write it, with a sample value for each directive, see
    /// [`Expansion::render`].
    #[cfg(feature = "testing")]
    pub fn render(self, text: &str) -> String {
        Expansion::render(text, &self.directives(text))
    }

    /// The characters that name the placeholder of an argument, see [`Template::name_arguments`].
    const NAMES: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";

    /// Names the placeholders of `directives` by the argument they take, so that the parameter
    /// conversion reads two directives as one parameter only if they take the same argument.
    ///
    /// A directive like `%s` only has room for `$` and one character, so the arguments get the
    /// characters of [`Template::NAMES`] in the order they first appear, and the 54th distinct
    /// argument gets the name of the first again. Where two arguments with the same name meet in
    /// one statement, [`Preprocessed::shared_names`] says so, and the statement isn't
    /// typechecked.
    fn name_arguments(directives: &mut [Expansion]) {
        let mut names: HashMap<Argument, &'static str> = HashMap::new();
        for directive in directives {
            let Value::Unknown { name, argument } = &mut directive.value else {
                continue;
            };
            let count = names.len();
            *name = names.entry(*argument).or_insert_with(|| {
                let index = count % Self::NAMES.len();
                Self::NAMES.get(index..index + 1).unwrap_or("e")
            });
        }
    }

    /// Whether a directive directly after a token of `kind` with `text` can't be read: after a
    /// name, a value, a `)`, a `]` or another directive, like `select a %s from t` or
    /// `from t {alias}`, it stands for an alias without `AS` or for a part of a clause, which a
    /// placeholder can't be. After a keyword, like `as %s`, or punctuation, like `%s.%s`,
    /// `= %d` or `in (%s)`, it is a name or a value.
    fn is_ambiguous_after(kind: SyntaxKind, text: &str) -> bool {
        match kind {
            // Keywords have kinds of their own, so these are names, quoted or not.
            SyntaxKind::IDENT
            | SyntaxKind::INT_NUMBER
            | SyntaxKind::FLOAT_NUMBER
            | SyntaxKind::STRING
            | SyntaxKind::ESC_STRING
            | SyntaxKind::BYTE_STRING
            | SyntaxKind::BIT_STRING
            | SyntaxKind::DOLLAR_QUOTED_STRING
            | SyntaxKind::POSITIONAL_PARAM
            | SyntaxKind::NAMED_PARAM
            | SyntaxKind::R_PAREN
            | SyntaxKind::R_BRACK => true,
            // Characters that SQL doesn't know, like the `{` of a Rust directive in Go mode.
            SyntaxKind::ERROR => text.chars().any(|c| c.is_alphanumeric() || c == '_'),
            _ => false,
        }
    }
}

/// A part of the file that the host replaces before Postgres sees it, like `%s` or `${SCHEMA}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Expansion<'a> {
    pub range: TextRange,
    pub value: Value<'a>,
}

#[cfg(feature = "testing")]
impl Expansion<'_> {
    /// `text` as the host would write it with sample values for `expansions`, which are in file
    /// order: a distinct identifier per positional argument, like `a0`, or a distinct integer
    /// for Go's numeric verbs, like `1000`; `v_NAME` for a named argument or variable; the text
    /// of a known value, like `%` for `%%`. Two texts that render to the same tree are the same
    /// program to the host, which tests of the formatter check.
    pub fn render(text: &str, expansions: &[Expansion]) -> String {
        let mut rendered = String::with_capacity(text.len());
        let mut end = 0;
        for expansion in expansions {
            let range = std::ops::Range::<usize>::from(expansion.range);
            let (Some(before), Some(written)) =
                (text.get(end..range.start), text.get(range.clone()))
            else {
                continue;
            };
            rendered.push_str(before);
            match expansion.value {
                Value::Known(text) => rendered.push_str(text),
                Value::Unknown {
                    argument: Argument::Index(index),
                    ..
                } if written.starts_with('%')
                    && written.ends_with(|c| "bcdoOxXUeEfFgG".contains(c)) =>
                {
                    rendered.push_str(&(1000 + index).to_string());
                }
                Value::Unknown {
                    argument: Argument::Index(index),
                    ..
                } => rendered.push_str(&format!("a{index}")),
                Value::Unknown {
                    argument: Argument::Named(name),
                    ..
                } => rendered.push_str(&format!("v_{name}")),
            }
            end = range.end;
        }
        rendered.push_str(text.get(end..).unwrap_or_default());
        rendered
    }
}

/// What the host writes in place of an [`Expansion`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value<'a> {
    /// A value that only the host knows: `argument`, like an argument of the format string or an
    /// environment variable. Two expansions of the same argument have the same value. `name`
    /// names the placeholder.
    Unknown {
        name: &'a str,
        argument: Argument<'a>,
    },
    /// This text, like `%` for Go's `%%`. It isn't longer than the expansion.
    Known(&'static str),
}

impl<'a> Value<'a> {
    /// A value of the variable `name`, which also names its placeholder, like an environment
    /// variable of goose's `ENVSUB`.
    pub fn variable(name: &'a str) -> Self {
        Self::Unknown {
            name,
            argument: Argument::Named(name),
        }
    }

    /// What names the placeholder of an identifier with this value.
    fn name(self) -> &'a str {
        match self {
            Self::Unknown { name, .. } => name,
            Self::Known(text) => text,
        }
    }
}

/// The argument that an [`Expansion`] takes from the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Argument<'a> {
    /// By position, from 0, like Go's `%s` or `%[2]d` or Rust's `{}` or `{0}`.
    Index(usize),
    /// By name, like Rust's `{name}` or an environment variable.
    Named(&'a str),
}

/// The text that the analysis reads instead of the file as written, with placeholders where the
/// host replaces parts of it, like goose's `ENVSUB` or Go's `fmt.Sprintf`. It has the length of
/// the file, so that the offsets of diagnostics, fixes and editor positions are the same in both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preprocessed {
    pub text: String,
    /// The ranges that the host replaces when it runs the file, in file order. The text holds a
    /// placeholder there, or the file as written where a placeholder wouldn't parse, like in a
    /// string literal.
    pub placeholders: Vec<TextRange>,
    /// The placeholders that stay as written in a value, like in a string literal, in file
    /// order. The text holds what is written there, not the value that the host puts there.
    pub unknown_values: Vec<TextRange>,
    /// The placeholders that can't be read where they are, in file order. Only templates have
    /// them, see [`Template::read`].
    pub ambiguous: Vec<Ambiguity>,
    /// The placeholders with the text of an earlier one, but another value, in file order. A
    /// statement with both of them reads them as one parameter, so it isn't typechecked.
    pub shared_names: Vec<SharedName>,
}

/// A placeholder with the text of an earlier placeholder of another value, like the 54th
/// distinct argument of a template, which has the name of the first. See
/// [`Template::name_arguments`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedName {
    pub placeholder: TextRange,
    /// The last placeholder before it with its text.
    pub earlier: TextRange,
}

/// A placeholder that can't be read where it is, because it directly follows a name or a value,
/// like the `%s` of `select a %s from t`: it stands for an alias without `AS` or for a part of a
/// clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ambiguity {
    pub placeholder: TextRange,
    /// The token it follows.
    pub after: TextRange,
}

impl Ambiguity {
    /// What the diagnostic of the placeholder says, given the text of the file.
    pub fn message(&self, text: &str) -> String {
        format!(
            "A template placeholder directly after `{}` can't be read: write `AS` before an alias; a placeholder can stand for a name or a value, not a SQL clause.",
            text.get(std::ops::Range::<usize>::from(self.after))
                .unwrap_or_default()
        )
    }
}

impl Preprocessed {
    /// Whether `range` overlaps a placeholder. An empty range touches it if it's inside it.
    pub fn touches_placeholder(&self, range: TextRange) -> bool {
        self.placeholders
            .iter()
            .any(|placeholder| Self::overlaps(*placeholder, range))
    }

    /// Whether a statement at `range` has two placeholders of different values with the same
    /// text, which the parameter conversion reads as one, see [`Preprocessed::shared_names`].
    pub fn shares_names_in(&self, range: TextRange) -> bool {
        let first = self
            .shared_names
            .partition_point(|shared| shared.placeholder.start() < range.start());
        self.shared_names
            .get(first..)
            .unwrap_or_default()
            .iter()
            .take_while(|shared| shared.placeholder.end() <= range.end())
            .any(|shared| shared.earlier.start() >= range.start())
    }

    /// Whether `range` overlaps a placeholder that stays as written in a value.
    pub fn touches_unknown_value(&self, range: TextRange) -> bool {
        self.unknown_values
            .iter()
            .any(|placeholder| Self::overlaps(*placeholder, range))
    }

    /// Whether `range` overlaps `placeholder`. An empty range overlaps it if it's inside it.
    fn overlaps(placeholder: TextRange, range: TextRange) -> bool {
        if range.is_empty() {
            placeholder.start() < range.start() && range.start() < placeholder.end()
        } else {
            placeholder.start() < range.end() && range.start() < placeholder.end()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What a template reads in a text.
    #[derive(Debug, PartialEq, Eq)]
    struct Read<'a> {
        /// The text that is analysed.
        text: String,
        placeholders: Vec<&'a str>,
        unknown_values: Vec<&'a str>,
    }

    impl<'a> Read<'a> {
        fn of(template: Template, text: &'a str) -> Option<Self> {
            let preprocessed = template.read(text)?;
            assert_eq!(preprocessed.text.len(), text.len());
            let written = |ranges: &[TextRange]| ranges.iter().map(|range| &text[*range]).collect();
            Some(Self {
                placeholders: written(&preprocessed.placeholders),
                unknown_values: written(&preprocessed.unknown_values),
                text: preprocessed.text,
            })
        }

        /// The shared names among the placeholders of `text`, as their text in the file.
        fn shared_names(template: Template, text: &str) -> Vec<(&str, &str)> {
            template
                .read(text)
                .map(|preprocessed| preprocessed.shared_names)
                .unwrap_or_default()
                .iter()
                .map(|shared| (&text[shared.placeholder], &text[shared.earlier]))
                .collect()
        }
    }

    #[test]
    fn more_arguments_than_names_are_not_merged_silently() {
        // The 54th distinct argument gets the name of the first.
        let text = format!("select {};", vec!["%v"; 54].join(", "));
        let read = Read::of(Template::GoPrintf, &text).unwrap();
        assert!(read.text.starts_with("select $a, $b,"), "{}", read.text);
        assert!(read.text.ends_with(", $_, $a;"), "{}", read.text);
        assert_eq!(
            Read::shared_names(Template::GoPrintf, &text),
            [("%v", "%v")]
        );
        let preprocessed = Template::GoPrintf.read(&text).unwrap();
        let whole = TextRange::up_to(pgls_text_size::TextSize::of(text.as_str()));
        assert!(preprocessed.shares_names_in(whole));
        // In different statements, they are different parameters anyway.
        let split = text.replacen(", ", "; select ", 53);
        let preprocessed = Template::GoPrintf.read(&split).unwrap();
        let last = split.rfind("select").unwrap();
        let last = TextRange::new(
            pgls_text_size::TextSize::try_from(last).unwrap(),
            pgls_text_size::TextSize::of(split.as_str()),
        );
        assert!(!preprocessed.shares_names_in(last));
    }

    #[test]
    fn templates_read_only_their_own_syntax() {
        let text = "select %s, {name};";
        assert_eq!(Read::of(Template::None, text), None);
        assert_eq!(
            Read::of(Template::GoPrintf, text).map(|read| read.text),
            Some("select $a, {name};".to_string())
        );
        assert_eq!(
            Read::of(Template::RustFmt, text).map(|read| read.text),
            Some("select %s, $a____;".to_string())
        );
    }
}
