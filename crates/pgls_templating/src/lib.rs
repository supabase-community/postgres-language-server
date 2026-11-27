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

use std::str::FromStr;

use pgls_text_size::TextRange;
use serde::{Deserialize, Serialize};

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
        (!directives.is_empty()).then(|| Preprocessed::substitute(text, &directives))
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
    /// A value that only the host knows, like an argument or an environment variable. Its name,
    /// like the name of the variable, names the placeholder.
    Unknown(&'a str),
    /// This text, like `%` for Go's `%%`. It isn't longer than the expansion.
    Known(&'static str),
}

impl<'a> Value<'a> {
    /// What names the placeholder of an identifier with this value.
    fn name(self) -> &'a str {
        match self {
            Self::Unknown(name) => name,
            Self::Known(text) => text,
        }
    }
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
}

impl Preprocessed {
    /// Whether `range` overlaps a placeholder. An empty range touches it if it's inside it.
    pub fn touches_placeholder(&self, range: TextRange) -> bool {
        self.placeholders
            .iter()
            .any(|placeholder| Self::overlaps(*placeholder, range))
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
            "select $v, $d_, $s__, $x_, $d_, $f____, $g__, $d_, $s___, $f____________, $T from t;"
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
                "select * from $s.$s where id = $d and name = $q;",
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
                "create table $app_s (id bigint);\nselect * from $_s_x_____, $s__, $ss_;",
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
                "select 7 %  2, '100%%', 7 % 2, 8, a $d;",
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
                "select $e, $e0, $name_, $name____, $e_, $value____________________, $e__ from $tenant_schema_, $_t______;",
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
    fn templates_read_only_their_own_syntax() {
        let text = "select %s, {name};";
        assert_eq!(Read::of(Template::None, text), None);
        assert_eq!(
            Read::of(Template::GoPrintf, text).map(|read| read.text),
            Some("select $s, {name};".to_string())
        );
        assert_eq!(
            Read::of(Template::RustFmt, text).map(|read| read.text),
            Some("select %s, $name_;".to_string())
        );
    }
}
