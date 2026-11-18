//! Postgres' own regression SQL (`src/test/regress/sql`) for every supported major version,
//! together with the verdict Postgres gave each statement.
//!
//! The data lives in `data/<major>/`:
//!
//! - `SOURCE`: the pinned upstream tag, e.g. `REL_17_11`
//! - `sql/*.sql`: the upstream regression files, verbatim
//! - `verdicts/*.txt`: one `start..end verdict` line per statement, where `start..end` is the byte
//!   range of the statement Postgres was sent, in the preprocessed source (see [`preprocess`])
//! - `catalog.json.gz`: the catalog of a fresh database on that version, as gzipped
//!   `pgls_catalog::Snapshot` JSON
//!
//! Everything is recorded by `just record-regress <major> [tag]`; reading it needs neither the
//! network nor a database, nor the statement splitter: a fix of the splitter doesn't change what
//! was recorded, and `tests/splitter.rs` shows where the splitter now differs from the recorded
//! statements. `just record-regress <major> --catalog-only` re-records only the catalog, e.g.
//! after the snapshot queries change.

use std::{
    fmt, fs,
    io::Read,
    path::{Path, PathBuf},
};

use pgls_text_size::{TextRange, TextSize};

/// What Postgres did with a statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Accepted,
    /// Rejected with the given SQLSTATE, or `CLIENT_ERROR` if the error didn't come from the server.
    Rejected(String),
    /// Not run, or no reliable answer: transaction control, `COPY` to or from the client,
    /// statements cancelled by `statement_timeout`, and everything after a statement that hung.
    Skipped,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::Accepted => f.write_str("accepted"),
            Verdict::Rejected(state) => write!(f, "rejected {state}"),
            Verdict::Skipped => f.write_str("skipped"),
        }
    }
}

impl Verdict {
    fn parse(s: &str) -> Option<Self> {
        match s.split_once(' ') {
            None if s == "accepted" => Some(Verdict::Accepted),
            None if s == "skipped" => Some(Verdict::Skipped),
            Some(("rejected", state)) if !state.is_empty() => {
                Some(Verdict::Rejected(state.to_owned()))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Statement {
    /// The statement's range in [`File::source`], without surrounding whitespace.
    pub range: TextRange,
    pub verdict: Verdict,
}

/// The statement's line in a verdict file.
impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}..{} {}",
            u32::from(self.range.start()),
            u32::from(self.range.end()),
            self.verdict
        )
    }
}

impl Statement {
    fn parse(line: &str) -> Option<Self> {
        let (range, verdict) = line.split_once(' ')?;
        let (start, end) = range.split_once("..")?;
        let (start, end) = (start.parse::<u32>().ok()?, end.parse::<u32>().ok()?);
        Some(Statement {
            range: (start <= end).then(|| TextRange::new(start.into(), end.into()))?,
            verdict: Verdict::parse(verdict)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct File {
    /// File name without the `.sql` extension, e.g. `join`.
    pub name: String,
    /// The preprocessed source; see [`preprocess`].
    pub source: String,
    pub statements: Vec<Statement>,
}

impl File {
    pub fn sql(&self, statement: &Statement) -> &str {
        &self.source[statement.range]
    }

    /// The 1-based line and column where the statement starts, which key its verdict.
    pub fn line_col(&self, statement: &Statement) -> (usize, usize) {
        self.line_col_at(statement.range.start())
    }

    /// The 1-based line and column of an offset in [`File::source`].
    pub fn line_col_at(&self, offset: TextSize) -> (usize, usize) {
        // One offset, one position.
        line_cols(&self.source, std::iter::once(offset))
            .into_iter()
            .next()
            .unwrap_or((1, 1))
    }
}

#[derive(Debug, Clone)]
pub struct Version {
    pub major: u32,
    /// The upstream tag the files come from, e.g. `REL_17_11`.
    pub tag: String,
    dir: PathBuf,
}

/// The name of the catalog file in `data/<major>/`.
pub const CATALOG_FILE: &str = "catalog.json.gz";

impl Version {
    /// The catalog of a fresh database on this version, as `pgls_catalog::Snapshot` JSON.
    pub fn catalog_json(&self) -> String {
        let path = self.dir.join(CATALOG_FILE);
        let file = fs::File::open(&path).unwrap_or_else(|e| {
            panic!(
                "read {}: {e}; run `just record-regress {} --catalog-only`",
                path.display(),
                self.major
            )
        });
        let mut json = String::new();
        flate2::read::GzDecoder::new(file)
            .read_to_string(&mut json)
            .unwrap_or_else(|e| panic!("decompress {}: {e}", path.display()));
        json
    }

    /// Loads every file of this version, sorted by name.
    ///
    /// Panics if a verdict file is missing or doesn't fit its SQL file.
    pub fn files(&self) -> Vec<File> {
        let sql_dir = self.dir.join("sql");
        let mut names = fs::read_dir(&sql_dir)
            .unwrap_or_else(|e| panic!("read {}: {e}", sql_dir.display()))
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|x| x == "sql"))
            .filter_map(|path| Some(path.file_stem()?.to_string_lossy().into_owned()))
            .collect::<Vec<_>>();
        names.sort();
        self.check_no_orphan_verdicts(&names);
        names.into_iter().map(|name| self.load(name)).collect()
    }

    fn load(&self, name: String) -> File {
        let path = self.dir.join("sql").join(format!("{name}.sql"));
        let raw =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let source = preprocess(&raw);
        let verdicts_path = self.verdicts_path(&name);
        let Ok(verdicts) = fs::read_to_string(&verdicts_path) else {
            self.out_of_date(&name, "no verdict file")
        };
        let mut statements = Vec::<Statement>::new();
        for entry in verdicts.lines() {
            let Some(statement) = Statement::parse(entry) else {
                self.out_of_date(&name, &format!("malformed verdict `{entry}`"))
            };
            let after_previous = statements
                .last()
                .is_none_or(|previous| previous.range.end() <= statement.range.start());
            let fits = source
                .get(statement.range.start().into()..statement.range.end().into())
                .is_some_and(|sql| !sql.is_empty() && sql.trim() == sql);
            if !after_previous || !fits {
                self.out_of_date(
                    &name,
                    &format!("verdict `{entry}` doesn't fit the statements of the SQL file"),
                );
            }
            statements.push(statement);
        }
        File {
            name,
            source,
            statements,
        }
    }

    fn check_no_orphan_verdicts(&self, names: &[String]) {
        let Ok(entries) = fs::read_dir(self.dir.join("verdicts")) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_stem().map(|stem| stem.to_string_lossy()) else {
                continue;
            };
            if !names.iter().any(|x| *x == name) {
                self.out_of_date(&name, "verdict file without SQL file");
            }
        }
    }

    fn verdicts_path(&self, name: &str) -> PathBuf {
        self.dir.join("verdicts").join(format!("{name}.txt"))
    }

    fn out_of_date(&self, name: &str, detail: &str) -> ! {
        panic!(
            "{}/{name}: {detail}; fixtures out of date, run `just record-regress {}`",
            self.major, self.major
        )
    }
}

/// The directory that holds one subdirectory per major version.
pub fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

/// Every recorded version, oldest first.
pub fn versions() -> Vec<Version> {
    let root = data_dir();
    let mut versions = fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("read {}: {e}", root.display()))
        .flatten()
        .filter_map(|entry| {
            let dir = entry.path();
            let major = dir.file_name()?.to_str()?.parse().ok()?;
            Some(version_in(dir, major))
        })
        .collect::<Vec<_>>();
    versions.sort_by_key(|x| x.major);
    versions
}

/// The recorded version for `major`, if there is one.
pub fn version(major: u32) -> Option<Version> {
    let dir = data_dir().join(major.to_string());
    dir.is_dir().then(|| version_in(dir, major))
}

fn version_in(dir: PathBuf, major: u32) -> Version {
    let source = dir.join("SOURCE");
    let tag = fs::read_to_string(&source)
        .unwrap_or_else(|e| panic!("read {}: {e}", source.display()))
        .trim()
        .to_owned();
    Version { major, tag, dir }
}

/// Removes what psql handles itself, so the rest can be sent to the server statement by statement:
/// meta-command lines (`\set`, `\if`, ...) and the data lines of `COPY ... FROM STDIN`.
///
/// Removed lines are blanked rather than dropped, so line numbers stay those of the upstream file.
pub fn preprocess(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut in_copy = false;
    for line in source.lines() {
        if in_copy {
            if line.trim() == "\\." {
                in_copy = false;
            }
        } else if !line.trim_start().starts_with('\\') {
            output.push_str(line);
            let upper = line.to_ascii_uppercase();
            if upper.contains("COPY")
                && (upper.contains("FROM STDIN") || upper.contains("FROM STDOUT"))
            {
                in_copy = true;
            }
        }
        output.push('\n');
    }
    output
}

/// Splits a preprocessed source into its statements, trimmed and without empty ones, with today's
/// splitter. The recorder sends these statements to Postgres; loading reads the recorded ranges
/// instead.
pub fn split(source: &str) -> Vec<TextRange> {
    pgls_statement_splitter::split(source)
        .ranges
        .into_iter()
        .filter_map(|range| {
            let text = &source[range];
            let trimmed = text.trim_start();
            let start = range.start() + TextSize::of(&text[..text.len() - trimmed.len()]);
            let len = TextSize::of(trimmed.trim_end());
            (len > 0.into()).then(|| TextRange::at(start, len))
        })
        .collect()
}

/// The 1-based line and column (in characters) of each offset, which must be ascending.
fn line_cols(source: &str, offsets: impl Iterator<Item = TextSize>) -> Vec<(usize, usize)> {
    let (mut line, mut line_start, mut done) = (1, 0, 0);
    offsets
        .map(|offset| {
            let offset = usize::from(offset);
            for (i, _) in source[done..offset].match_indices('\n') {
                line += 1;
                line_start = done + i + 1;
            }
            done = offset;
            (line, source[line_start..offset].chars().count() + 1)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocess_keeps_line_numbers() {
        let source = "select 1;\n\\set x 1\ncopy t from stdin;\n1\t2\n\\.\nselect 2;\n";
        let cleaned = preprocess(source);
        assert_eq!(cleaned, "select 1;\n\ncopy t from stdin;\n\n\nselect 2;\n");
        let ranges = split(&cleaned);
        let starts = line_cols(&cleaned, ranges.iter().map(|range| range.start()));
        assert_eq!(starts, [(1, 1), (3, 1), (6, 1)]);
    }

    #[test]
    fn verdicts_round_trip() {
        for verdict in [
            Verdict::Accepted,
            Verdict::Skipped,
            Verdict::Rejected("42P01".into()),
        ] {
            assert_eq!(Verdict::parse(&verdict.to_string()), Some(verdict));
        }
        for line in ["0..9 accepted", "12..40 rejected 42P01", "40..40 skipped"] {
            let statement = Statement::parse(line).expect("a verdict line");
            assert_eq!(statement.to_string(), line);
        }
        for line in ["9..0 accepted", "0..9", "0-9 accepted", "0..9 rejected"] {
            assert!(Statement::parse(line).is_none(), "{line}");
        }
    }

    #[test]
    fn line_cols_count_characters() {
        let source = "ä; select 1;\nselect 2; select 3;";
        let offsets = [0, 4, 14, 24].map(TextSize::from);
        assert_eq!(
            line_cols(source, offsets.into_iter()),
            [(1, 1), (1, 4), (2, 1), (2, 11)]
        );
    }

    #[test]
    fn every_version_loads() {
        let versions = versions();
        assert!(!versions.is_empty(), "no recorded versions");
        for version in versions {
            let files = version.files();
            assert!(!files.is_empty(), "{} has no files", version.major);
            assert!(
                files
                    .iter()
                    .flat_map(|file| &file.statements)
                    .any(|statement| statement.verdict == Verdict::Accepted),
                "{} has no accepted statement",
                version.major
            );
        }
    }
}
