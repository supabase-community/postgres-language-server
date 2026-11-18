//! One run: `check` and `format` of one CLI binary over the corpus, normalised so that two runs
//! compare.

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::manifest::Corpus;

/// How long one invocation of the CLI may take. The largest source, Postgres' regression suite,
/// takes about 10 s in a release build; a debug build is about ten times slower.
const INVOKE_TIMEOUT: Duration = Duration::from_secs(600);

/// How often a running invocation is checked for its exit and its memory.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// The memory one invocation may take. CI runners have 16 GiB; above this, the kernel's OOM killer
/// would end the run without saying why. `check` of GitLab's `structure.sql` took more than 20 GiB
/// in 0.28.2. Only enforced on Linux, where `/proc` has the size.
const RSS_BYTES_MAX: u64 = 8 * 1024 * 1024 * 1024;

/// The size limit passed to the CLI. GitLab's `structure.sql` is about 3 MiB; the CLI's default
/// limit would skip it.
const FILE_BYTES_MAX: u64 = 16 * 1024 * 1024;

/// How much of a crash's message is kept: the line with the panic message, not the backtrace.
const REASON_BYTES_MAX: usize = 300;

/// The configuration the CLI runs with: every rule, so a change to any of them shows.
const CONFIGURATION: &str = r#"{ "linter": { "rules": { "all": true } } }"#;
const CONFIGURATION_FILE: &str = "postgres-language-server.jsonc";

/// The results file of a run, next to the `sources/` directory with the formatted files.
const RESULTS_FILE: &str = "results.json";
pub const SOURCES_DIR: &str = "sources";

#[derive(Serialize, Deserialize)]
pub struct Results {
    pub sources: Vec<SourceFiles>,
    /// The diagnostics of `check`, sorted.
    pub check: Vec<Finding>,
    /// The diagnostics of `format`, sorted.
    pub format: Vec<Finding>,
    /// Invocations that didn't produce a report, and internal errors reported as diagnostics.
    pub crashes: Vec<Crash>,
}

#[derive(Serialize, Deserialize)]
pub struct SourceFiles {
    pub name: String,
    pub licence: String,
    /// The pinned commit of a git source.
    pub rev: Option<String>,
    /// Relative to the source's directory, sorted.
    pub files: Vec<String>,
}

/// A diagnostic, without its end and with its path relative to `sources/`.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub category: String,
    pub severity: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Crash {
    /// The CLI command: `check` or `format`.
    pub command: String,
    /// The file, relative to `sources/`.
    pub path: String,
    pub reason: String,
}

/// The CLI's JSON report (`--reporter=json`), the parts the check needs.
#[derive(Deserialize)]
struct Report {
    diagnostics: Vec<ReportDiagnostic>,
}

#[derive(Deserialize)]
struct ReportDiagnostic {
    severity: String,
    message: String,
    category: Option<String>,
    location: Option<ReportLocation>,
}

#[derive(Deserialize)]
struct ReportLocation {
    path: String,
    start: ReportPosition,
}

#[derive(Deserialize)]
struct ReportPosition {
    line: usize,
    column: usize,
}

/// What one invocation produced.
enum Outcome {
    Report(Vec<Finding>),
    Crash(String),
}

/// Runs the CLI binary in the output directory.
struct Cli<'a> {
    binary: &'a Path,
    out: &'a Path,
}

impl Results {
    /// Copies the corpus into `out/sources/`, runs `check` and then `format --write` over each
    /// source, and writes `out/results.json`. Afterwards `out/sources/` holds the formatted files.
    pub fn collect(corpus: &Corpus, cache: &Path, binary: &Path, out: &Path) -> Result<Self> {
        let binary = binary
            .canonicalize()
            .with_context(|| format!("finding the binary {}", binary.display()))?;
        std::fs::create_dir_all(out).with_context(|| format!("creating {}", out.display()))?;
        let out = out.canonicalize()?;
        ensure!(
            std::fs::read_dir(&out)?.next().is_none(),
            "{} is not empty",
            out.display()
        );
        std::fs::write(out.join(CONFIGURATION_FILE), CONFIGURATION)?;
        let cli = Cli {
            binary: &binary,
            out: &out,
        };

        let mut results = Self {
            sources: Vec::with_capacity(corpus.sources.len()),
            check: Vec::new(),
            format: Vec::new(),
            crashes: Vec::new(),
        };
        for source in &corpus.sources {
            let dir = corpus.dir(source, cache);
            let files = source.files(&dir)?;
            eprintln!("{}: {} files", source.name, files.len());
            let target = PathBuf::from(SOURCES_DIR).join(&source.name);
            let copy = || -> Result<()> {
                for file in &files {
                    let to = out.join(&target).join(file);
                    if let Some(parent) = to.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::copy(dir.join(file), &to)
                        .with_context(|| format!("copying {}/{file}", source.name))?;
                }
                Ok(())
            };
            copy()?;

            for command in ["check", "format"] {
                let findings = match cli.invoke(command, &[target.as_path()])? {
                    Outcome::Report(findings) => findings,
                    Outcome::Crash(reason) if files.len() == 1 => {
                        results.crashes.push(Crash::new(
                            command,
                            format!("{}/{}", source.name, files.join("")),
                            reason,
                        ));
                        Vec::new()
                    }
                    Outcome::Crash(reason) => {
                        // Find the files that crash. `format` may have written some files before
                        // it crashed, so start again from the originals.
                        eprintln!(
                            "{}: `{command}` crashed ({reason}); running it per file",
                            source.name
                        );
                        if command == "format" {
                            copy()?;
                        }
                        let mut findings = Vec::new();
                        for file in &files {
                            match cli.invoke(command, &[target.join(file).as_path()])? {
                                Outcome::Report(found) => findings.extend(found),
                                Outcome::Crash(reason) => results.crashes.push(Crash::new(
                                    command,
                                    format!("{}/{file}", source.name),
                                    reason,
                                )),
                            }
                        }
                        findings
                    }
                };
                let (internal, findings): (Vec<_>, Vec<_>) = findings
                    .into_iter()
                    .partition(|f| f.category.starts_with("internalError"));
                results.crashes.extend(internal.into_iter().map(|finding| {
                    let reason = format!("{}: {}", finding.category, finding.message);
                    Crash::new(command, finding.path, reason)
                }));
                match command {
                    "check" => results.check.extend(findings),
                    _ => results.format.extend(findings),
                }
            }

            results.sources.push(SourceFiles {
                name: source.name.clone(),
                licence: source.licence.clone(),
                rev: source.origin.rev().map(str::to_string),
                files,
            });
        }
        results.check.sort_unstable();
        results.format.sort_unstable();
        results.crashes.sort_unstable();

        let json = serde_json::to_string(&results)?;
        std::fs::write(out.join(RESULTS_FILE), json)?;
        Ok(results)
    }

    pub fn load(out: &Path) -> Result<Self> {
        let path = out.join(RESULTS_FILE);
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    }
}

impl Cli<'_> {
    /// Runs `command` over `targets` (relative to the output directory) with the JSON reporter.
    /// Exit codes 0 and 1 (diagnostics with errors) are reports; anything else, a timeout or a
    /// report that doesn't parse is a crash.
    fn invoke(&self, command: &str, targets: &[&Path]) -> Result<Outcome> {
        let stdout_path = self.out.join("stdout.json");
        let stderr_path = self.out.join("stderr.txt");
        let mut cli = Command::new(self.binary);
        cli.current_dir(self.out)
            .arg(command)
            .args([
                "--config-path",
                CONFIGURATION_FILE,
                "--reporter=json",
                "--max-diagnostics=none",
                "--colors=off",
                "--disable-db",
            ])
            .arg(format!("--files-max-size={FILE_BYTES_MAX}"));
        if command == "format" {
            cli.arg("--write");
        }
        let mut child = cli
            .args(targets)
            .stdin(Stdio::null())
            // Files, not pipes: a pipe that nobody reads while waiting fills up and blocks the CLI.
            .stdout(File::create(&stdout_path)?)
            .stderr(File::create(&stderr_path)?)
            .spawn()
            .with_context(|| format!("running {}", self.binary.display()))?;

        let deadline = Instant::now() + INVOKE_TIMEOUT;
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                child.kill()?;
                child.wait()?;
                return Ok(Outcome::Crash(format!(
                    "timed out after {} s",
                    INVOKE_TIMEOUT.as_secs()
                )));
            }
            if Self::rss_bytes(child.id()).is_some_and(|rss| rss > RSS_BYTES_MAX) {
                child.kill()?;
                child.wait()?;
                return Ok(Outcome::Crash(format!(
                    "stopped above {} GiB of memory",
                    RSS_BYTES_MAX >> 30
                )));
            }
            std::thread::sleep(POLL_INTERVAL);
        };

        let stdout = std::fs::read_to_string(&stdout_path).unwrap_or_default();
        let report = stdout
            .lines()
            .filter(|line| line.starts_with('{'))
            .find_map(|line| serde_json::from_str::<Report>(line).ok());
        let report = match (status.code(), report) {
            (Some(0 | 1), Some(report)) => report,
            (code, _) => {
                let stderr = std::fs::read_to_string(&stderr_path).unwrap_or_default();
                let message = stderr
                    .lines()
                    .find_map(|line| line.trim().strip_prefix("Message:"))
                    .map(str::trim)
                    .unwrap_or_default();
                let mut reason = match code {
                    Some(0 | 1) => "the report didn't parse".to_string(),
                    _ => format!("{status}"),
                };
                if !message.is_empty() {
                    reason.push_str(": ");
                    reason.push_str(message);
                }
                return Ok(Outcome::Crash(reason));
            }
        };

        let findings = report
            .diagnostics
            .into_iter()
            .map(|diagnostic| {
                let (path, line, column) = match diagnostic.location {
                    Some(location) => {
                        let path = location.path.replace('\\', "/");
                        let path = path.trim_start_matches("./");
                        let path = path
                            .strip_prefix(SOURCES_DIR)
                            .and_then(|p| p.strip_prefix('/'))
                            .unwrap_or(path);
                        (path.to_string(), location.start.line, location.start.column)
                    }
                    None => (String::new(), 0, 0),
                };
                Finding {
                    path,
                    line,
                    column,
                    category: diagnostic.category.unwrap_or_default(),
                    severity: diagnostic.severity,
                    message: diagnostic.message,
                }
            })
            .collect();
        Ok(Outcome::Report(findings))
    }
}

impl Cli<'_> {
    /// The resident memory of a process, from `/proc/<pid>/status`. `None` off Linux.
    fn rss_bytes(pid: u32) -> Option<u64> {
        let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
        let kib = status
            .lines()
            .find_map(|line| line.strip_prefix("VmRSS:"))?
            .trim()
            .strip_suffix("kB")?
            .trim()
            .parse::<u64>()
            .ok()?;
        Some(kib * 1024)
    }
}

impl Crash {
    /// Keeps the first [`REASON_BYTES_MAX`] bytes of the reason.
    fn new(command: &str, path: String, mut reason: String) -> Self {
        let mut end = reason.len().min(REASON_BYTES_MAX);
        while !reason.is_char_boundary(end) {
            end -= 1;
        }
        reason.truncate(end);
        Self {
            command: command.to_string(),
            path,
            reason,
        }
    }
}
