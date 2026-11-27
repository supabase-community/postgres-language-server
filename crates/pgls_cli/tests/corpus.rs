//! The corpus check: runs the CLI over real-world SQL and compares a summary per source with its
//! snapshot, `tests/snapshots/corpus__<source>.snap`.
//!
//! The sources are listed in `tests/corpus.toml`, each pinned to a commit, and fetched into
//! `target/tmp/corpus` on the first run, which needs network access. The CLI runs `check` over each
//! source with every rule and without a database, and `format` without `--write`. The summary holds
//! the number of diagnostics per category, the files `format` would change and the crashes, so a
//! change to a rule, the parser or the formatter shows up as a changed snapshot.
//!
//! Next to the CLI, the test checks every file with the fuzzers' oracles, which live in their crates
//! behind the `testing` features, and counts where they fail: `format_statement` doesn't refuse a
//! statement for a failed round trip and its output formats to itself, the splitter's statements
//! of a file that parses parse to the file's statements, and the lexer's tokens are the scanner's.
//!
//! The run takes about a minute in release. In debug, `format` of GitLab's schema runs into the time
//! limit, so the test refuses to run there. It is ignored in the regular run; CI runs it with
//!
//! ```sh
//! cargo test -p pgls_cli --release --test corpus -- --ignored
//! ```
//!
//! Prefix it with `INSTA_UPDATE=always` to accept the changed summaries.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails on unexpected data; clippy exempts `#[test]` functions, not their helpers"
)]

use pgls_line_index::LineIndex;
use pgls_pretty_print::{FormatConfig, FormatError, FormatStatementParams, format_statement};
use pgls_text_size::TextSize;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io::{Read, Seek, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

/// Every rule, so a change to any of them shows.
const CONFIGURATION: &str = r#"{ "linter": { "rules": { "all": true } } }"#;

/// How long one run of the CLI over a source may take. The slowest, `format` of GitLab's schema,
/// takes about 50 s in release. A run that takes longer is recorded as a crash and stopped, rather
/// than holding up CI.
const RUN_DURATION_MAX: Duration = Duration::from_secs(300);

/// How often a running CLI is checked for its exit.
const RUN_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// The CLI skips larger files. GitLab's schema is about 3 MiB, above the default limit.
const FILE_BYTES_MAX: u64 = 16 * 1024 * 1024;

/// How many of the files `format` would change the summary lists, so it stays small. The count
/// covers the rest.
const REFORMATTED_LISTED_MAX: usize = 20;

/// How many of the places where an oracle fails the summary lists. The count covers the rest.
const FINDINGS_LISTED_MAX: usize = 5;

/// The first line the CLI's panic hook writes to stderr.
const PANIC_LINE: &str = "Encountered an unexpected error";

#[derive(Deserialize)]
struct Corpus {
    source: Vec<Source>,
}

#[derive(Deserialize)]
struct Source {
    name: String,
    repo: String,
    /// A full commit SHA.
    rev: String,
    /// The directories and files the CLI runs over.
    paths: Vec<String>,
}

/// The CLI's JSON report, the parts the summary needs.
#[derive(Deserialize)]
struct Report {
    summary: ReportSummary,
    diagnostics: Vec<ReportDiagnostic>,
}

#[derive(Deserialize)]
struct ReportSummary {
    changed: usize,
    unchanged: usize,
    skipped: usize,
}

#[derive(Deserialize)]
struct ReportDiagnostic {
    category: Option<String>,
    location: Option<ReportLocation>,
}

#[derive(Deserialize)]
struct ReportLocation {
    path: String,
}

/// The snapshot of one source. Counts and paths only: messages and lines would make every
/// unrelated change to a message or a fixture show up.
#[derive(Default)]
struct Summary {
    files: usize,
    skipped: usize,
    /// Diagnostics per category of `check`.
    check: BTreeMap<String, usize>,
    /// Diagnostics per category of `format`.
    format: BTreeMap<String, usize>,
    /// The files `format` would change.
    reformatted: BTreeSet<String>,
    /// Runs that exited abnormally, panicked or reported an `internalError/*`.
    crashes: BTreeSet<String>,
    oracles: Oracles,
}

/// Where the fuzzers' oracles fail, as `path:line`, in the order of the files.
#[derive(Default)]
struct Oracles {
    /// Statements that `format_statement` refuses because its output doesn't parse or parses to a
    /// different AST, or whose output doesn't format to itself.
    format: Vec<String>,
    /// Files that parse, but whose splitter's statements don't parse to the file's statements. At
    /// the first statement that differs.
    split: Vec<String>,
    /// Files where the lexer's tokens differ from the scanner's. At the first token that differs.
    lex: Vec<String>,
}

#[test]
#[ignore = "fetches the sources and needs a release build; CI runs it"]
fn corpus() {
    if cfg!(debug_assertions) {
        panic!(
            "the corpus check runs in release: \
             cargo test -p pgls_cli --release --test corpus -- --ignored"
        );
    }
    let (corpus, cache) = Corpus::load();
    let config = cache.join("postgres-language-server.jsonc");
    fs::write(&config, CONFIGURATION).unwrap();

    let summaries: Vec<_> = corpus
        .source
        .iter()
        .map(|source| {
            let dir = source.checkout(&cache);
            (source, Summary::of(source, &dir, &config))
        })
        .collect();

    // Compare all of them, so that every changed summary is shown and written.
    let changed: Vec<_> = summaries
        .iter()
        .filter(|(source, summary)| {
            catch_unwind(|| insta::assert_snapshot!(source.name.as_str(), summary.to_string()))
                .is_err()
        })
        .map(|(source, _)| source.name.as_str())
        .collect();
    assert!(changed.is_empty(), "the summaries of {changed:?} changed");
}

/// Only fetches the sources, which the nightly fuzz runs use as seeds. Unlike `corpus`, it runs in
/// debug.
#[test]
#[ignore = "fetches the sources; the nightly fuzz runs use them"]
fn corpus_sources() {
    let (corpus, cache) = Corpus::load();
    for source in &corpus.source {
        source.checkout(&cache);
    }
}

impl Corpus {
    /// The sources of `tests/corpus.toml`, and the directory they are checked out into.
    fn load() -> (Self, PathBuf) {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus.toml");
        let corpus = toml::from_str(&fs::read_to_string(manifest).unwrap()).unwrap();
        let cache = Path::new(env!("CARGO_TARGET_TMPDIR")).join("corpus");
        fs::create_dir_all(&cache).unwrap();
        (corpus, cache)
    }
}

impl Source {
    /// Checks out the `.sql` and `.pg` files under `paths` at `rev` into the cache, unless an
    /// earlier run did, and returns the directory.
    fn checkout(&self, cache: &Path) -> PathBuf {
        assert!(
            self.rev.len() == 40 && self.rev.bytes().all(|b| b.is_ascii_hexdigit()),
            "{}: `rev` must be a full commit SHA",
            self.name
        );
        // `corpus` and `corpus_sources` run in parallel and check out the same sources.
        static CHECKOUT: Mutex<()> = Mutex::new(());
        let _checkout = CHECKOUT.lock().unwrap_or_else(PoisonError::into_inner);
        let dir = cache.join(format!("{}-{}", self.name, self.rev));
        if dir.exists() {
            return dir;
        }
        // Checked out next to it and renamed when complete, so an interrupted fetch is redone.
        let partial = cache.join(format!("{}-{}.partial", self.name, self.rev));
        if partial.exists() {
            fs::remove_dir_all(&partial).unwrap();
        }
        fs::create_dir_all(&partial).unwrap();
        let started = Instant::now();

        self.git(&partial, &["init", "--quiet"], "");
        // The commit and its trees, without the file contents.
        self.git(
            &partial,
            &[
                "fetch",
                "--quiet",
                "--depth=1",
                "--filter=blob:none",
                &self.repo,
                &self.rev,
            ],
            "",
        );
        // The contents of the SQL files in one fetch. Checking them out without it fetches each
        // file separately, which takes minutes.
        let mut ls_tree = vec![
            "ls-tree",
            "-r",
            "-z",
            "--format=%(objectname) %(path)",
            &self.rev,
            "--",
        ];
        ls_tree.extend(self.paths.iter().map(String::as_str));
        let listing = self.git(&partial, &ls_tree, "");
        let (blobs, files): (Vec<_>, Vec<_>) = listing
            .split('\0')
            .filter_map(|entry| entry.split_once(' '))
            .filter(|(_, path)| path.ends_with(".sql") || path.ends_with(".pg"))
            .unzip();
        assert!(
            !files.is_empty(),
            "{}: no SQL files in {:?}",
            self.name,
            self.paths
        );
        self.git(
            &partial,
            &[
                "fetch",
                "--quiet",
                "--filter=blob:none",
                "--stdin",
                &self.repo,
            ],
            &blobs.join("\n"),
        );
        self.git(
            &partial,
            &[
                "checkout",
                "--quiet",
                &self.rev,
                "--pathspec-from-file=-",
                "--pathspec-file-nul",
            ],
            &files.join("\0"),
        );

        fs::rename(&partial, &dir).unwrap();
        eprintln!(
            "{}: fetched {} files in {:.1?}",
            self.name,
            files.len(),
            started.elapsed()
        );
        dir
    }

    /// Runs `git` in `dir` and returns its output. Fails with git's message, e.g. when offline.
    fn git(&self, dir: &Path, args: &[&str], stdin: &str) -> String {
        let mut child = Command::new("git")
            .current_dir(dir)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("running git");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}: `git {}` failed ({}); the corpus check needs network access to fetch {}:\n{}",
            self.name,
            args.join(" "),
            output.status,
            self.repo,
            String::from_utf8_lossy(&output.stderr),
        );
        String::from_utf8(output.stdout).unwrap()
    }
}

impl Summary {
    /// Runs `check` and `format` over the source checked out in `dir`.
    fn of(source: &Source, dir: &Path, config: &Path) -> Self {
        let mut summary = Self::default();
        for command in ["check", "format"] {
            summary.run(command, source, dir, config);
        }
        let started = Instant::now();
        summary.check_oracles(source, dir);
        eprintln!(
            "{}: the oracles took {:.1?}",
            source.name,
            started.elapsed()
        );
        summary
    }

    /// Runs one command of the CLI and adds what it reported.
    fn run(&mut self, command: &str, source: &Source, dir: &Path, config: &Path) {
        let mut stdout = tempfile::tempfile().unwrap();
        let mut stderr = tempfile::tempfile().unwrap();
        let started = Instant::now();
        let mut child = Command::new(env!("CARGO_BIN_EXE_postgres-language-server"))
            .current_dir(dir)
            .arg(command)
            .arg("--config-path")
            .arg(config)
            .args([
                "--reporter=json",
                "--max-diagnostics=none",
                "--colors=off",
                "--disable-db",
            ])
            .arg(format!("--files-max-size={FILE_BYTES_MAX}"))
            .args(&source.paths)
            .stdin(Stdio::null())
            // Files, not pipes: a pipe that nobody reads while waiting fills up and blocks the CLI.
            .stdout(stdout.try_clone().unwrap())
            .stderr(stderr.try_clone().unwrap())
            .spawn()
            .expect("running the CLI");
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if started.elapsed() > RUN_DURATION_MAX {
                child.kill().unwrap();
                child.wait().unwrap();
                eprintln!(
                    "{}: `{command}` stopped after {RUN_DURATION_MAX:?}",
                    source.name
                );
                self.crashes.insert(format!(
                    "{command}: timed out after {} s",
                    RUN_DURATION_MAX.as_secs()
                ));
                return;
            }
            std::thread::sleep(RUN_POLL_INTERVAL);
        };
        eprintln!(
            "{}: `{command}` took {:.1?}",
            source.name,
            started.elapsed()
        );

        let [stdout, stderr] = [&mut stdout, &mut stderr].map(|file| {
            let mut text = String::new();
            file.rewind().unwrap();
            file.read_to_string(&mut text).unwrap();
            text
        });
        let crashes = self.crashes.len();
        if stderr.contains(PANIC_LINE) {
            self.crashes.insert(format!("{command}: panicked"));
        }
        // Exit code 1 means the report has errors.
        if !matches!(status.code(), Some(0 | 1)) {
            self.crashes.insert(format!("{command}: {status}"));
        }
        let report = stdout
            .lines()
            .filter(|line| line.starts_with('{'))
            .find_map(|line| serde_json::from_str::<Report>(line).ok());
        if report.is_none() {
            self.crashes.insert(format!("{command}: no report"));
        }
        if self.crashes.len() > crashes {
            // The message stays out of the snapshot, but CI's log should have it.
            eprintln!("{}: `{command}` crashed:\n{stderr}", source.name);
        }
        let Some(report) = report else {
            return;
        };

        self.files = self
            .files
            .max(report.summary.changed + report.summary.unchanged);
        self.skipped = self.skipped.max(report.summary.skipped);
        for diagnostic in report.diagnostics {
            let category = diagnostic.category.unwrap_or_default();
            let path = diagnostic
                .location
                .map(|location| location.path.trim_start_matches("./").to_string())
                .unwrap_or_default();
            if category.starts_with("internalError") {
                self.crashes
                    .insert(format!("{command}: {category} in {path}"));
                continue;
            }
            if command == "format" && category == "format" {
                self.reformatted.insert(path);
            }
            let counts = match command {
                "check" => &mut self.check,
                _ => &mut self.format,
            };
            *counts.entry(category).or_default() += 1;
        }
    }
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "files: {} ({} skipped)", self.files, self.skipped)?;
        for (command, counts) in [("check", &self.check), ("format", &self.format)] {
            writeln!(f, "{command}:")?;
            for (category, count) in counts {
                writeln!(f, "  {category}: {count}")?;
            }
        }
        Summary::write_listed(
            f,
            "reformatted files",
            &self.reformatted,
            REFORMATTED_LISTED_MAX,
        )?;
        writeln!(f, "crashes: {}", self.crashes.len())?;
        for crash in &self.crashes {
            writeln!(f, "  {crash}")?;
        }
        for (oracle, findings) in [
            ("format round-trip", &self.oracles.format),
            ("splitter vs parser", &self.oracles.split),
            ("lexer vs scanner", &self.oracles.lex),
        ] {
            Summary::write_listed(f, oracle, findings, FINDINGS_LISTED_MAX)?;
        }
        Ok(())
    }
}

impl Summary {
    /// Writes the count of `items` and the first `listed_max` of them.
    fn write_listed<'a>(
        f: &mut fmt::Formatter<'_>,
        title: &str,
        items: impl IntoIterator<Item = &'a String, IntoIter: ExactSizeIterator>,
        listed_max: usize,
    ) -> fmt::Result {
        let items = items.into_iter();
        let count = items.len();
        writeln!(f, "{title}: {count}")?;
        for item in items.take(listed_max) {
            writeln!(f, "  {item}")?;
        }
        if count > listed_max {
            writeln!(f, "  and {} more", count - listed_max)?;
        }
        Ok(())
    }
}

impl Summary {
    /// Checks the SQL files of the source checked out in `dir` with the oracles. An oracle that
    /// panics fails.
    fn check_oracles(&mut self, source: &Source, dir: &Path) {
        let listing = source.git(dir, &["ls-files", "-z"], "");
        let mut paths: Vec<_> = listing
            .split('\0')
            .filter(|path| !path.is_empty())
            .collect();
        paths.sort_unstable();
        let config = FormatConfig::default();
        for path in paths {
            let file = dir.join(path);
            if fs::metadata(&file).unwrap().len() > FILE_BYTES_MAX {
                continue;
            }
            // The CLI reports a file that isn't UTF-8; there is nothing to check.
            let Ok(sql) = fs::read_to_string(&file) else {
                continue;
            };
            let lines = LineIndex::new(&sql);
            let at = |offset: TextSize| {
                let line = lines
                    .line_col(offset)
                    .map_or(0, |line_col| line_col.line + 1);
                format!("{path}:{line}")
            };
            let split = pgls_statement_splitter::split(&sql);
            for &range in &split.ranges {
                let statement = &sql[range];
                // Text that doesn't parse as one statement, and refusals other than a failed round
                // trip, pass.
                let round_trips = catch_unwind(|| {
                    let Some(ast) = pgls_query::parse(statement)
                        .ok()
                        .and_then(|parsed| parsed.into_root())
                    else {
                        return true;
                    };
                    match format_statement(FormatStatementParams::new(&ast, statement, &config)) {
                        Ok(result) => result.formats_to_itself(&config),
                        Err(
                            FormatError::ParseError { .. } | FormatError::BetaUnsupported { .. },
                        ) => false,
                        Err(_) => true,
                    }
                });
                if !round_trips.unwrap_or(false) {
                    self.oracles.format.push(at(range.start()));
                }
            }
            let split = catch_unwind(AssertUnwindSafe(|| split.parser_difference(&sql)));
            if let Some(offset) = split.unwrap_or(Some(TextSize::from(0))) {
                self.oracles.split.push(at(offset));
            }
            let lex = catch_unwind(|| pgls_lexer::lex(&sql).scanner_difference());
            if let Some(offset) = lex.unwrap_or(Some(TextSize::from(0))) {
                self.oracles.lex.push(at(offset));
            }
        }
    }
}
