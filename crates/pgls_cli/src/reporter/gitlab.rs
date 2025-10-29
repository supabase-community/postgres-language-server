use crate::diagnostics::CliDiagnostic;
use crate::reporter::{Report, ReportConfig, ReportWriter};
use path_absolutize::Absolutize;
use pgls_console::fmt::{Display, Formatter};
use pgls_console::{Console, ConsoleExt, markup};
use pgls_diagnostics::display::SourceFile;
use pgls_diagnostics::{Error, PrintDescription, Resource, Severity};
use serde::Serialize;
use std::sync::RwLock;
use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
};

pub(crate) struct GitLabReportWriter;

impl ReportWriter for GitLabReportWriter {
    fn write(
        &mut self,
        console: &mut dyn Console,
        _command_name: &str,
        report: &Report,
        config: &ReportConfig,
    ) -> Result<(), CliDiagnostic> {
        let repository_root = report
            .traversal
            .as_ref()
            .and_then(|traversal| traversal.workspace_root.clone());

        let hasher = RwLock::default();
        let diagnostics = GitLabDiagnostics {
            report,
            config,
            hasher: &hasher,
            repository_root: repository_root.as_deref(),
        };
        console.log(markup!({ diagnostics }));
        Ok(())
    }
}

#[derive(Default)]
struct GitLabHasher(HashSet<u64>);

impl GitLabHasher {
    fn rehash_until_unique(&mut self, fingerprint: u64) -> u64 {
        let mut current = fingerprint;
        while self.0.contains(&current) {
            let mut hasher = DefaultHasher::new();
            current.hash(&mut hasher);
            current = hasher.finish();
        }

        self.0.insert(current);
        current
    }
}

struct GitLabDiagnostics<'a> {
    report: &'a Report,
    config: &'a ReportConfig,
    hasher: &'a RwLock<GitLabHasher>,
    repository_root: Option<&'a Path>,
}

impl<'a> GitLabDiagnostics<'a> {
    fn attempt_to_relativize(&self, subject: &str) -> Option<PathBuf> {
        let Ok(resolved) = Path::new(subject).absolutize() else {
            return None;
        };

        let Ok(relativized) = resolved.strip_prefix(self.repository_root?) else {
            return None;
        };

        Some(relativized.to_path_buf())
    }

    fn compute_initial_fingerprint(&self, diagnostic: &Error, path: &str) -> u64 {
        let location = diagnostic.location();
        let code = match location.span {
            Some(span) => match location.source_code {
                Some(source_code) => &source_code.text[span],
                None => "",
            },
            None => "",
        };

        let check_name = diagnostic
            .category()
            .map(|category| category.name())
            .unwrap_or_default();

        calculate_hash(&Fingerprint {
            check_name,
            path,
            code,
        })
    }
}

impl Display for GitLabDiagnostics<'_> {
    fn fmt(&self, fmt: &mut Formatter) -> std::io::Result<()> {
        let mut hasher = self.hasher.write().unwrap();
        let gitlab_diagnostics: Vec<_> = self
            .report
            .diagnostics
            .iter()
            .filter(|d| d.severity() >= self.config.diagnostic_level)
            .filter(|d| {
                if self.config.verbose {
                    d.tags().is_verbose()
                } else {
                    true
                }
            })
            .filter_map(|pgls_diagnostic| {
                let absolute_path = match pgls_diagnostic.location().resource {
                    Some(Resource::File(file)) => Some(file),
                    _ => None,
                }
                .unwrap_or_default();
                let path_buf = self.attempt_to_relativize(absolute_path);
                let path = match path_buf {
                    Some(buf) => buf.to_str().unwrap_or(absolute_path).to_owned(),
                    None => absolute_path.to_owned(),
                };

                let initial_fingerprint = self.compute_initial_fingerprint(pgls_diagnostic, &path);
                let fingerprint = hasher.rehash_until_unique(initial_fingerprint);

                GitLabDiagnostic::try_from_diagnostic(
                    pgls_diagnostic,
                    path.to_string(),
                    fingerprint,
                )
            })
            .collect();
        let serialized = serde_json::to_string_pretty(&gitlab_diagnostics)?;
        fmt.write_str(serialized.as_str())?;
        Ok(())
    }
}

/// An entry in the GitLab Code Quality report.
/// See https://docs.gitlab.com/ee/ci/testing/code_quality.html#implement-a-custom-tool
#[derive(Serialize)]
pub struct GitLabDiagnostic<'a> {
    /// A description of the code quality violation.
    description: String,
    /// A unique name representing the static analysis check that emitted this issue.
    check_name: &'a str,
    /// A unique fingerprint to identify the code quality violation. For example, an MD5 hash.
    fingerprint: String,
    /// A severity string (can be info, minor, major, critical, or blocker).
    severity: &'a str,
    /// The location where the code quality violation occurred.
    location: Location,
}

impl<'a> GitLabDiagnostic<'a> {
    pub fn try_from_diagnostic(
        diagnostic: &'a Error,
        path: String,
        fingerprint: u64,
    ) -> Option<Self> {
        let location = diagnostic.location();
        let span = location.span?;
        let source_code = location.source_code?;
        let description = PrintDescription(diagnostic).to_string();
        let begin = match SourceFile::new(source_code).location(span.start()) {
            Ok(start) => start.line_number.get(),
            Err(_) => return None,
        };
        let check_name = diagnostic
            .category()
            .map(|category| category.name())
            .unwrap_or_default();

        Some(GitLabDiagnostic {
            severity: match diagnostic.severity() {
                Severity::Hint => "info",
                Severity::Information => "minor",
                Severity::Warning => "major",
                Severity::Error => "critical",
                Severity::Fatal => "blocker",
            },
            description,
            check_name,
            fingerprint: fingerprint.to_string(),
            location: Location {
                path,
                lines: GitLabLines { begin },
            },
        })
    }
}

#[derive(Serialize)]
pub struct Location {
    path: String,
    lines: GitLabLines,
}

#[derive(Serialize)]
pub struct GitLabLines {
    begin: usize,
}

#[derive(Hash, Serialize)]
pub struct Fingerprint<'a> {
    code: &'a str,
    check_name: &'a str,
    path: &'a str,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}
