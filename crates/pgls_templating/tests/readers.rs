//! What the readers of the hosts make of the fixtures in `tests/data/<syntax>/`: one snapshot per
//! fixture with the text that the analysis reads under each line of the source, each
//! placeholder, and the diagnostics of the reader, and the invariants of a [`Preprocessed`] text
//! checked for every fixture. A new fixture needs no new code.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

use std::fmt::Write;

use dir_test::{Fixture, dir_test};
use insta::{assert_snapshot, with_settings};
use pgls_lexer::SyntaxKind;
use pgls_migrations::MigrationTool;
use pgls_templating::{Preprocessed, Template};
use pgls_text_size::TextRange;

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/data/go-printf/", glob: "*.sql")]
fn go_printf(fixture: Fixture<&str>) {
    ReaderFixture::check(&fixture, Syntax::GoPrintf);
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/data/rust-fmt/", glob: "*.sql")]
fn rust_fmt(fixture: Fixture<&str>) {
    ReaderFixture::check(&fixture, Syntax::RustFmt);
}

#[dir_test(dir: "$CARGO_MANIFEST_DIR/tests/data/envsub/", glob: "*.sql")]
fn envsub(fixture: Fixture<&str>) {
    ReaderFixture::check(&fixture, Syntax::Envsub);
}

/// The host of a folder of fixtures.
#[derive(Clone, Copy)]
enum Syntax {
    GoPrintf,
    RustFmt,
    /// goose's `ENVSUB`, read through the goose migration tool, so its fixtures have markers.
    Envsub,
}

impl Syntax {
    fn read(self, text: &str) -> Option<Preprocessed> {
        match self {
            Self::GoPrintf => Template::GoPrintf.read(text),
            Self::RustFmt => Template::RustFmt.read(text),
            Self::Envsub => MigrationTool::Goose.read(text).preprocessed,
        }
    }

    fn folder(self) -> &'static str {
        match self {
            Self::GoPrintf => "go-printf",
            Self::RustFmt => "rust-fmt",
            Self::Envsub => "envsub",
        }
    }
}

/// A fixture and what the reader of its host makes of it.
struct ReaderFixture<'a> {
    text: &'a str,
    preprocessed: Preprocessed,
}

impl<'a> ReaderFixture<'a> {
    /// Checks the invariants of the fixture and snapshots its summary.
    fn check(fixture: &Fixture<&'a str>, syntax: Syntax) {
        let text = fixture.content();
        let name = std::path::Path::new(fixture.path())
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        let preprocessed = syntax
            .read(text)
            .unwrap_or_else(|| panic!("{name}: the fixture has no directives"));
        let fixture = Self { text, preprocessed };
        fixture.assert_invariants();
        with_settings!({
            omit_expression => true,
            snapshot_path => format!("snapshots/{}", syntax.folder()),
            prepend_module_to_snapshot => false,
        }, {
            assert_snapshot!(name, fixture.summary());
        });
    }

    /// What every [`Preprocessed`] text has: the length and the line breaks of the file, the
    /// file as written outside of the placeholders, and placeholders in file order that don't
    /// overlap, with the unknown values among them.
    fn assert_invariants(&self) {
        let (text, analysed) = (self.text, self.preprocessed.text.as_str());
        assert_eq!(analysed.len(), text.len(), "the length changed");
        let breaks = |text: &str| {
            text.match_indices(['\n', '\r'])
                .map(|(offset, _)| offset)
                .collect::<Vec<_>>()
        };
        assert_eq!(breaks(analysed), breaks(text), "the line breaks moved");

        let placeholders = &self.preprocessed.placeholders;
        for pair in placeholders.windows(2) {
            assert!(
                pair[0].end() <= pair[1].start(),
                "placeholders out of order or overlapping: {pair:?}"
            );
        }
        let mut end = 0;
        for placeholder in placeholders
            .iter()
            .map(|range| std::ops::Range::<usize>::from(*range))
            .chain(std::iter::once(text.len()..text.len()))
        {
            assert_eq!(
                analysed[end..placeholder.start],
                text[end..placeholder.start],
                "the text outside of the placeholders changed"
            );
            end = placeholder.end;
        }
        for unknown in &self.preprocessed.unknown_values {
            assert!(
                placeholders.contains(unknown),
                "{unknown:?} isn't a placeholder"
            );
        }
    }

    /// The source with the analysed text under each line, the placeholders, and the
    /// diagnostics of the reader.
    fn summary(&self) -> String {
        let (text, analysed) = (self.text, self.preprocessed.text.as_str());
        let mut summary = String::new();
        for (number, (line, analysed_line)) in (1..).zip(
            text.split_inclusive('\n')
                .zip(analysed.split_inclusive('\n')),
        ) {
            writeln!(summary, "{number:>3} | {}", line.trim_end()).unwrap();
            if analysed_line != line {
                writeln!(summary, "    | {}", analysed_line.trim_end()).unwrap();
            }
        }

        let kinds = self.kinds();
        summary.push_str("\nplaceholders:\n");
        for (placeholder, kind) in self.preprocessed.placeholders.iter().zip(kinds) {
            writeln!(
                summary,
                "  {:>9} `{}` -> `{}` {kind}",
                format!("{placeholder:?}"),
                &text[*placeholder],
                &analysed[*placeholder],
            )
            .unwrap();
        }

        let ambiguous = &self.preprocessed.ambiguous;
        let shared = &self.preprocessed.shared_names;
        if !ambiguous.is_empty() || !shared.is_empty() {
            summary.push_str("\ndiagnostics:\n");
        }
        for ambiguity in ambiguous {
            writeln!(
                summary,
                "  {:?} `{}`: {}",
                ambiguity.placeholder,
                &text[ambiguity.placeholder],
                ambiguity.message(text)
            )
            .unwrap();
        }
        for shared in shared {
            writeln!(
                summary,
                "  {:?} `{}` has the placeholder of {:?} `{}`, another value: the statement \
                 with both isn't typechecked",
                shared.placeholder,
                &text[shared.placeholder],
                shared.earlier,
                &text[shared.earlier],
            )
            .unwrap();
        }
        summary
    }

    /// What each placeholder is to the analysis: a value or an identifier, as the parameter
    /// conversion of its statement reads it, an unknown value in a string, as written in a
    /// comment, or what the host writes, like `%` for `%%`.
    fn kinds(&self) -> Vec<String> {
        let analysed = self.preprocessed.text.as_str();
        let lexed = pgls_lexer::lex(analysed);
        let tokens: Vec<(TextRange, SyntaxKind)> = lexed
            .tokens()
            .enumerate()
            .map(|(index, kind)| (lexed.range(index), kind))
            .collect();
        let statements = pgls_statement_splitter::split(analysed).ranges;
        self.preprocessed
            .placeholders
            .iter()
            .map(|placeholder| {
                let token = tokens
                    .iter()
                    .find(|(range, _)| range.contains(placeholder.start()))
                    .map(|(_, kind)| *kind);
                match token {
                    _ if self.preprocessed.unknown_values.contains(placeholder) => {
                        "unknown value".to_string()
                    }
                    Some(SyntaxKind::COMMENT) => "in a comment".to_string(),
                    Some(SyntaxKind::NAMED_PARAM) => {
                        let statement = statements
                            .iter()
                            .find(|statement| statement.contains_range(*placeholder))
                            .expect("a placeholder in a statement");
                        let converted =
                            pgls_lexer::convert_to_positional_params(&analysed[*statement]);
                        let relative = *placeholder - statement.start();
                        if converted[relative].starts_with('$') {
                            "value".to_string()
                        } else {
                            "identifier".to_string()
                        }
                    }
                    _ if analysed[*placeholder] == self.text[*placeholder] => {
                        "as written".to_string()
                    }
                    _ => format!("written as `{}`", analysed[*placeholder].trim_end()),
                }
            })
            .collect()
    }
}
