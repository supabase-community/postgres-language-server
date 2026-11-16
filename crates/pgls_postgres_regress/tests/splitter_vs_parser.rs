//! Checks the statement boundaries of the splitter against libpg_query's parser, on every file of
//! Postgres' regression suite of every supported version.
//!
//! The parser can't take a whole file, since the files hold statements Postgres rejects. So the
//! check starts at each statement of the splitter and looks for the statement psql sends from
//! there: psql ends a statement at a `;` outside of parentheses and `BEGIN ... END`, so it's the
//! shortest text up to a `;` that the parser reads as one statement. If that isn't one statement
//! of the splitter, the splitter cut it in pieces, ran past its end, or both.
//!
//! Where no text up to a `;` parses, e.g. at a statement with a syntax error, the check moves on
//! to the next statement of the splitter.
//!
//! The mismatches are snapshotted per version, so a new one fails the test and a fixed one shows
//! up as a diff.

use pgls_postgres_regress::{File, Version};
use pgls_query::protobuf::Token;
use pgls_text_size::{TextRange, TextSize};

/// How many statements of the splitter after a statement without a `;` are scanned for the end of
/// the statement. A wrong cut makes a handful of fragments; the limit bounds the scan of a long
/// run of statements without a `;`.
const SCANNED_STATEMENTS_MAX: usize = 16;

/// How many `;` the check tries as the end of a statement. A `BEGIN ATOMIC` body holds one per
/// statement.
const TRIED_SEMICOLONS_MAX: usize = 64;

/// How many characters of a statement the report shows.
const REPORTED_CHARS_MAX: usize = 80;

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
    let ranges = file
        .statements
        .iter()
        .map(|statement| statement.range)
        .collect::<Vec<_>>();
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
fn parser_statement(file: &File, ranges: &[TextRange], counts: &mut Counts) -> Option<TextRange> {
    let first = ranges.first()?;
    // The statements of the splitter up to the first one that ends with a `;`.
    let scanned = ranges
        .iter()
        .position(|range| file.source[*range].ends_with(';'))
        .unwrap_or(ranges.len() - 1);
    if scanned > SCANNED_STATEMENTS_MAX {
        counts.limits_hit += 1;
    }
    let last = ranges.get(scanned.min(SCANNED_STATEMENTS_MAX))?;
    let window = TextRange::new(first.start(), last.end());
    let text = &file.source[window];
    let tokens = pgls_query::scan(text).ok()?.tokens;
    let mut ends = tokens
        .iter()
        .filter(|token| token.token == Token::Ascii59 as i32)
        .filter_map(|token| u32::try_from(token.end).ok())
        .map(|end| window.start() + TextSize::from(end))
        .collect::<Vec<_>>();
    if ends.len() > TRIED_SEMICOLONS_MAX {
        counts.limits_hit += 1;
        ends.truncate(TRIED_SEMICOLONS_MAX);
    }
    if scanned + 1 == ranges.len() && !text.ends_with(';') {
        ends.push(window.end());
    }
    ends.into_iter()
        .map(|end| TextRange::new(window.start(), end))
        .find(|&candidate| {
            pgls_query::split_with_parser(&file.source[candidate])
                .is_ok_and(|statements| statements.len() == 1)
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
