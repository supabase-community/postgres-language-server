//! Checks the statement boundaries of the splitter on every file of Postgres' regression suite of
//! every supported version: against the statements the fixtures were recorded with, and against
//! libpg_query's parser. The differences are snapshotted per version, so a new one fails the test
//! and a fixed one shows up as a diff.
//!
//! The fixtures key each verdict by the range of the statement Postgres was sent, so a fix of the
//! splitter shows up in the first check until the fixtures are re-recorded with
//! `just record-regress <major>`; the verdicts stay valid in the meantime.
//!
//! The parser can't take a whole file, since the files hold statements Postgres rejects. So the
//! second check starts at each statement of the splitter and looks for the statement psql sends
//! from there: psql ends a statement at a `;` outside of parentheses and `BEGIN ... END`, so it's
//! the shortest text up to a `;` that the parser reads as one statement. If that isn't one
//! statement of the splitter, the splitter cut it in pieces, ran past its end, or both.
//!
//! Where no text up to a `;` parses, e.g. at a statement with a syntax error, the check moves on
//! to the next statement of the splitter. Such a statement is tried with the statements after it
//! too, since the splitter may have cut at a `;` inside it, e.g. in a `BEGIN ATOMIC` body.

use pgls_postgres_regress::{File, Version};
use pgls_query::protobuf::Token;
use pgls_text_size::{TextRange, TextSize};

/// How many statements of the splitter after a statement are scanned for the end of the
/// statement. A wrong cut makes a handful of fragments; the limit bounds the scan of a long run of
/// statements without a `;`, and of the statements after one that doesn't parse.
const SCANNED_STATEMENTS_MAX: usize = 16;

/// How many `;` the check tries as the end of a statement. A `BEGIN ATOMIC` body holds one per
/// statement.
const TRIED_SEMICOLONS_MAX: usize = 64;

/// How many characters of a statement the report shows.
const REPORTED_CHARS_MAX: usize = 80;

#[test]
fn splitter_agrees_with_recordings() {
    for version in pgls_postgres_regress::versions() {
        let mut statements = 0;
        let mut entries = Vec::new();
        for file in version.files() {
            statements += file.statements.len();
            for difference in Difference::all(&file) {
                entries.push(difference.entry(&file));
            }
        }
        let mut report = format!(
            "{}: {statements} recorded statements, {} differences\n",
            version.tag,
            entries.len()
        );
        for entry in entries {
            report.push('\n');
            report.push_str(&entry);
        }
        insta::assert_snapshot!(format!("splitter_vs_recorded_{}", version.major), report);
    }
}

/// Overlapping statements of the recording and the splitter that differ. Either side is empty
/// where the other has a statement that overlaps none of its own.
struct Difference {
    recorded: Vec<TextRange>,
    splitter: Vec<TextRange>,
}

impl Difference {
    fn all(file: &File) -> Vec<Difference> {
        let recorded = file
            .statements
            .iter()
            .map(|statement| statement.range)
            .collect::<Vec<_>>();
        let splitter = pgls_postgres_regress::split(&file.source);
        let (mut r, mut s) = (0, 0);
        let mut differences = Vec::new();
        loop {
            let (next_recorded, next_splitter) = (recorded.get(r), splitter.get(s));
            if next_recorded.is_some() && next_recorded == next_splitter {
                r += 1;
                s += 1;
                continue;
            }
            // The statement that starts first opens the difference, and every statement that
            // overlaps it joins. Statements aren't empty, so every round takes at least the first
            // one, and the loop ends after both lists.
            let Some(mut end) = [next_recorded, next_splitter]
                .into_iter()
                .flatten()
                .min_by_key(|range| range.start())
                .map(|range| range.end())
            else {
                break;
            };
            let mut difference = Difference {
                recorded: Vec::new(),
                splitter: Vec::new(),
            };
            loop {
                let mut grew = false;
                for (ranges, index, taken) in [
                    (&recorded, &mut r, &mut difference.recorded),
                    (&splitter, &mut s, &mut difference.splitter),
                ] {
                    while let Some(&range) = ranges.get(*index).filter(|x| x.start() < end) {
                        end = end.max(range.end());
                        taken.push(range);
                        *index += 1;
                        grew = true;
                    }
                }
                if !grew {
                    break;
                }
            }
            differences.push(difference);
        }
        differences
    }

    fn entry(&self, file: &File) -> String {
        let start = self
            .recorded
            .iter()
            .chain(&self.splitter)
            .map(|range| range.start())
            .min()
            .unwrap_or_default();
        let mut entry = format!("{}.sql:{}:\n", file.name, location(file, start));
        for (side, ranges) in [("recorded", &self.recorded), ("splitter", &self.splitter)] {
            for &range in ranges {
                entry.push_str(&format!(
                    "  {side} {}-{} {}\n",
                    location(file, range.start()),
                    location(file, range.end()),
                    excerpt(&file.source[range])
                ));
            }
        }
        entry
    }
}

#[test]
fn splitter_agrees_with_parser() {
    for version in pgls_postgres_regress::versions() {
        insta::assert_snapshot!(
            format!("splitter_vs_parser_{}", version.major),
            report(&version)
        );
    }
}

/// The statement psql sends from the start of a statement of the splitter, where that differs
/// from the splitter's.
struct Mismatch {
    /// The parser's statement, with its `;`.
    parser: TextRange,
    /// The statements of the splitter that overlap it.
    splitter: Vec<TextRange>,
}

#[derive(Default)]
struct Counts {
    statements: usize,
    /// Statements where no text up to a `;` parses.
    unchecked: usize,
    /// Statements where a limit stopped the search for the end.
    limits_hit: usize,
}

fn report(version: &Version) -> String {
    let mut entries = Vec::new();
    let mut counts = Counts::default();
    for file in version.files() {
        for mismatch in mismatches(&file, &mut counts) {
            entries.push(entry(&file, &mismatch));
        }
    }
    let mut report = format!(
        "{}: {} statements, {} not checked because no text up to a `;` parses, {} mismatches\n",
        version.tag,
        counts.statements,
        counts.unchecked,
        entries.len()
    );
    if counts.limits_hit > 0 {
        report.push_str(&format!(
            "{} statements hit a limit: more than {SCANNED_STATEMENTS_MAX} statements without a \
             `;` or {TRIED_SEMICOLONS_MAX} `;` tried\n",
            counts.limits_hit
        ));
    }
    for entry in entries {
        report.push('\n');
        report.push_str(&entry);
    }
    report
}

fn mismatches(file: &File, counts: &mut Counts) -> Vec<Mismatch> {
    let ranges = pgls_postgres_regress::split(&file.source);
    counts.statements += ranges.len();
    let mut mismatches = Vec::new();
    let mut index = 0;
    while let Some(&range) = ranges.get(index) {
        let Some(parser) = parser_statement(file, &ranges[index..], counts) else {
            counts.unchecked += 1;
            index += 1;
            continue;
        };
        let overlapping = ranges[index..]
            .iter()
            .take_while(|range| range.start() < parser.end())
            .copied()
            .collect::<Vec<_>>();
        index += overlapping.len().max(1);
        if overlapping != [TextRange::new(range.start(), parser.end())] {
            mismatches.push(Mismatch {
                parser,
                splitter: overlapping,
            });
        }
    }
    mismatches
}

/// The statement psql sends from the start of `ranges[0]`: the shortest text up to a `;` that the
/// parser reads as one statement. At the end of the file, the text up to the end counts too.
///
/// The `;` are first looked for up to the first statement of the splitter that ends with one. If
/// none of them ends a statement, the splitter may have cut at a `;` inside the statement, e.g. in
/// a `BEGIN ATOMIC` body, so the `;` of the statements after it are tried too.
fn parser_statement(file: &File, ranges: &[TextRange], counts: &mut Counts) -> Option<TextRange> {
    let first = ranges.first()?;
    let last = ranges.len() - 1;
    let with_semicolon = ranges
        .iter()
        .position(|range| file.source[*range].ends_with(';'))
        .unwrap_or(last);
    if with_semicolon > SCANNED_STATEMENTS_MAX {
        counts.limits_hit += 1;
    }
    let mut tried = first.start();
    for scanned in [with_semicolon, last].map(|x| x.min(SCANNED_STATEMENTS_MAX)) {
        let window = TextRange::new(first.start(), ranges.get(scanned)?.end());
        if window.end() <= tried {
            continue;
        }
        let found = statement_end(file, window, tried, scanned == last, counts);
        if found.is_some() {
            return found;
        }
        tried = window.end();
    }
    None
}

/// The shortest text from the start of `window` up to a `;` after `tried` that the parser reads
/// as one statement starting at the start of `window`. With `at_end`, the text up to the end of `window` counts too.
fn statement_end(
    file: &File,
    window: TextRange,
    tried: TextSize,
    at_end: bool,
    counts: &mut Counts,
) -> Option<TextRange> {
    let text = &file.source[window];
    let tokens = pgls_query::scan(text).ok()?.tokens;
    let mut ends = tokens
        .iter()
        .filter(|token| token.token == Token::Ascii59 as i32)
        .filter_map(|token| u32::try_from(token.end).ok())
        .map(|end| window.start() + TextSize::from(end))
        .filter(|&end| end > tried)
        .collect::<Vec<_>>();
    if ends.len() > TRIED_SEMICOLONS_MAX {
        counts.limits_hit += 1;
        ends.truncate(TRIED_SEMICOLONS_MAX);
    }
    if at_end && !text.ends_with(';') {
        ends.push(window.end());
    }
    ends.into_iter()
        .map(|end| TextRange::new(window.start(), end))
        .find(|&candidate| {
            // The statement starts where the one of the splitter does, rather than after an
            // empty statement, e.g. a lone `;`.
            let text = &file.source[candidate];
            pgls_query::split_with_parser(text).is_ok_and(|statements| {
                matches!(statements.as_slice(), [statement] if statement.as_ptr() == text.as_ptr())
            })
        })
}

fn entry(file: &File, mismatch: &Mismatch) -> String {
    let mut entry = format!(
        "{}.sql:{}: the parser reads {}-{}, the splitter {}\n",
        file.name,
        location(file, mismatch.parser.start()),
        location(file, mismatch.parser.start()),
        location(file, mismatch.parser.end()),
        mismatch
            .splitter
            .iter()
            .map(|range| format!(
                "{}-{}",
                location(file, range.start()),
                location(file, range.end())
            ))
            .collect::<Vec<_>>()
            .join(", ")
    );
    for &range in &mismatch.splitter {
        entry.push_str(&format!(
            "  {:<8} {}\n",
            location(file, range.start()),
            excerpt(&file.source[range])
        ));
    }
    entry
}

fn location(file: &File, offset: TextSize) -> String {
    let (line, col) = file.line_col_at(offset);
    format!("{line}:{col}")
}

/// The statement on one line, shortened to [`REPORTED_CHARS_MAX`] characters.
fn excerpt(sql: &str) -> String {
    let line = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.chars().count() <= REPORTED_CHARS_MAX {
        line
    } else {
        format!(
            "{}…",
            line.chars().take(REPORTED_CHARS_MAX).collect::<String>()
        )
    }
}
