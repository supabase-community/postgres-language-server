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
        let mut directives: Vec<Expansion> = match self {
            Self::None => return None,
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
        Self::name_arguments(&mut directives);
        (!directives.is_empty())
            .then(|| Preprocessed::substitute_reading(text, &directives, Self::is_ambiguous_after))
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

        /// The placeholders that can't be read where they are, with the text they follow.
        fn ambiguous(template: Template, text: &'a str) -> Vec<(&'a str, &'a str)> {
            template
                .read(text)
                .map(|preprocessed| preprocessed.ambiguous)
                .unwrap_or_default()
                .iter()
                .map(|ambiguity| (&text[ambiguity.placeholder], &text[ambiguity.after]))
                .collect()
        }

        fn new(text: &str, placeholders: &[&'a str], unknown_values: &[&'a str]) -> Option<Self> {
            Some(Self {
                text: text.to_string(),
                placeholders: placeholders.to_vec(),
                unknown_values: unknown_values.to_vec(),
            })
        }
    }

    #[test]
    fn go_printf_reads_every_directive_form() {
        let text =
            "select %v, %+d, %-8s, %#x, % d, %05.2f, %.3g, %*d, %[1]s, %[3]*.[2]*[1]f, %T from t;";
        let read = Read::of(Template::GoPrintf, text).unwrap();
        assert_eq!(
            read.placeholders,
            [
                "%v",
                "%+d",
                "%-8s",
                "%#x",
                "% d",
                "%05.2f",
                "%.3g",
                "%*d",
                "%[1]s",
                "%[3]*.[2]*[1]f",
                "%T"
            ]
        );
        assert_eq!(
            read.text,
            "select $a, $b_, $c__, $d_, $e_, $f____, $g__, $h_, $a___, $a____________, $b from t;"
        );
    }

    #[test]
    fn go_printf_placeholders_are_values_or_identifiers_by_context() {
        assert_eq!(
            Read::of(
                Template::GoPrintf,
                "select * from %s.%s where id = %d and name = %q;"
            ),
            Read::new(
                "select * from $a.$b where id = $c and name = $d;",
                &["%s", "%s", "%d", "%q"],
                &[]
            )
        );
        // Composite names, also with Unicode, and quoted identifiers are one parameter.
        assert_eq!(
            Read::of(
                Template::GoPrintf,
                "create table app_%s (id bigint);\nselect * from 漢é_%s_x, \"%s\", %s%s;"
            ),
            Read::new(
                "create table $app_a (id bigint);\nselect * from $_b_x_____, $c__, $de_;",
                &["app_%s", "漢é_%s_x", "\"%s\"", "%s%s"],
                &[]
            )
        );
    }

    #[test]
    fn go_printf_escapes_and_the_modulo_operator() {
        // Go writes `%` for `%%`, and the `% 2,` of `7 % 2, 8` is no directive.
        assert_eq!(
            Read::of(
                Template::GoPrintf,
                "select 7 %% 2, '100%%', 7 % 2, 8, a %d;"
            ),
            Read::new(
                "select 7 %  2, '100%%', 7 % 2, 8, a $a;",
                &["%%", "%%", "%d"],
                &["%%"]
            )
        );
        for text in [
            "select 7 % 2;",
            "select 7 %;",
            "select 7 % 2.5;",
            "select '%z', '%[x]d', '%[1d', '%Y', '%';",
        ] {
            assert_eq!(Read::of(Template::GoPrintf, text), None, "{text}");
        }
    }

    #[test]
    fn go_printf_keeps_directives_in_strings_and_comments() {
        let text = "select '%s', E'%d', $$ select %v $$; -- %q\n/* %x */";
        assert_eq!(
            Read::of(Template::GoPrintf, text),
            Read::new(text, &["%s", "%d", "%v", "%q", "%x"], &["%s", "%d", "%v"])
        );
    }

    #[test]
    fn rust_fmt_reads_every_directive_form() {
        let text = "select {}, {0}, {name}, {name:>8}, { }, {value:>width$.precision$}, {:?} from tenant_{schema}, 漢é_{t};";
        assert_eq!(
            Read::of(Template::RustFmt, text),
            Read::new(
                "select $a, $a_, $b____, $b_______, $c_, $d________________________, $e__ from $tenant_f______, $_g______;",
                &[
                    "{}",
                    "{0}",
                    "{name}",
                    "{name:>8}",
                    "{ }",
                    "{value:>width$.precision$}",
                    "{:?}",
                    "tenant_{schema}",
                    "漢é_{t}"
                ],
                &[]
            )
        );
    }

    #[test]
    fn rust_fmt_escapes_and_braces_that_start_no_directive() {
        // `{{` and `}}` write `{` and `}`. `{1,2}` and `{"a"` are no directives.
        assert_eq!(
            Read::of(
                Template::RustFmt,
                "select '{{\"a\": {v}}}'::jsonb, '{1,2}'::int[], '{\"a\": 1}';"
            ),
            Read::new(
                "select '{{\"a\": {v}}}'::jsonb, '{1,2}'::int[], '{\"a\": 1}';",
                &["{{", "{v}", "}}"],
                &["{{", "{v}", "}}"]
            )
        );
        for text in [
            "select '{1,2}';",
            "select 1 }",
            "select {a b};",
            "select {a",
        ] {
            assert_eq!(Read::of(Template::RustFmt, text), None, "{text}");
        }
    }

    #[test]
    fn rust_fmt_keeps_directives_in_strings_and_comments() {
        let text = "select '{}', $body$ select {x} $body$; -- {0}\n";
        assert_eq!(
            Read::of(Template::RustFmt, text),
            Read::new(text, &["{}", "{x}", "{0}"], &["{}", "{x}"])
        );
    }

    #[test]
    fn directives_after_a_name_or_a_value_are_ambiguous() {
        for (text, ambiguous) in [
            ("select a %s from t;", ("%s", "a")),
            ("select \"a\" %s from t;", ("%s", "\"a\"")),
            ("select 1 %s;", ("%s", "1")),
            ("select 'a' %s;", ("%s", "'a'")),
            ("select f(x) %s;", ("%s", ")")),
            ("select a[1] %s;", ("%s", "]")),
            ("select $1 %s;", ("%s", "$1")),
            ("select %s %s from t;", ("%s", "%s")),
            ("select * from t app_%s;", ("app_%s", "t")),
        ] {
            assert_eq!(
                Read::ambiguous(Template::GoPrintf, text),
                [ambiguous],
                "{text}"
            );
        }
        assert_eq!(
            Read::ambiguous(Template::RustFmt, "select * from t {alias};"),
            [("{alias}", "t")]
        );
        assert_eq!(
            Read::ambiguous(Template::RustFmt, "select * from t {}{}, t {0}_x;"),
            [("{}{}", "t"), ("{0}_x", "t")]
        );
        for text in [
            "select a as %s, %s.%s, app_%s, tenant_x from t where id = %d and x in (%s);",
            "select * from %s join %s on %s.id = %d limit %d offset %d;",
            "select 7 %% %d, a::%s, '%s' as x, \"%s\" from t; -- a %s",
            "select a \"100%%\" from t;",
        ] {
            assert_eq!(
                Read::ambiguous(Template::GoPrintf, text),
                Vec::<(&str, &str)>::new(),
                "{text}"
            );
        }
        assert_eq!(
            Read::ambiguous(
                Template::RustFmt,
                "select * from {schema}.{table} as {alias} where tenant_{schema}.id = {};"
            ),
            Vec::<(&str, &str)>::new()
        );
    }

    /// The placeholders of the directives of `text`, and the shared names among them, as their
    /// text in the file.
    fn shared_names(template: Template, text: &str) -> Vec<(&str, &str)> {
        template
            .read(text)
            .map(|preprocessed| preprocessed.shared_names)
            .unwrap_or_default()
            .iter()
            .map(|shared| (&text[shared.placeholder], &text[shared.earlier]))
            .collect()
    }

    #[test]
    fn placeholders_are_named_by_the_argument_they_take() {
        // Distinct implicit arguments get distinct placeholders, an index takes its argument,
        // and the directives after it continue from there. `%*d` takes the width from the
        // third argument and the value from the fourth.
        assert_eq!(
            Read::of(
                Template::GoPrintf,
                "select %v, %v, %[1]s, %s, %[1]d, %*d, %d from t;"
            )
            .map(|read| read.text),
            Some("select $a, $b, $a___, $b, $a___, $c_, $d from t;".to_string())
        );
        // `{}` takes the next positional argument, `{0}` the first, and `{name}` the same
        // argument every time. A precision `.*` takes an argument before the value.
        assert_eq!(
            Read::of(
                Template::RustFmt,
                "select {}, {}, {0}, {name}, {name:>4}, {:.*}, {};"
            )
            .map(|read| read.text),
            Some("select $a, $b, $a_, $c____, $c_______, $d___, $e;".to_string())
        );
        assert_eq!(
            shared_names(Template::GoPrintf, "select %v, %v, %[1]v;"),
            Vec::<(&str, &str)>::new()
        );
    }

    #[test]
    fn more_arguments_than_names_are_not_merged_silently() {
        // The 54th distinct argument gets the name of the first.
        let text = format!("select {};", vec!["%v"; 54].join(", "));
        let read = Read::of(Template::GoPrintf, &text).unwrap();
        assert!(read.text.starts_with("select $a, $b,"), "{}", read.text);
        assert!(read.text.ends_with(", $_, $a;"), "{}", read.text);
        assert_eq!(shared_names(Template::GoPrintf, &text), [("%v", "%v")]);
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
