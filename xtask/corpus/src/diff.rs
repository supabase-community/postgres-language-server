//! The differences between two runs, as Markdown for a job summary: counts, rules, paths and
//! lines, never the source text.

use anyhow::Result;
use std::cmp::Ordering;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use crate::run::{Crash, Finding, Results, SOURCES_DIR};

/// How many entries each list of the report shows. A change that touches a rule shows up on
/// thousands of statements; the first hundred show what it is, and the per-rule table the scale.
const LISTED_MAX: usize = 100;

pub struct Diff {
    base: Results,
    head: Results,
    check: Changes<Finding>,
    format: Changes<Finding>,
    /// Findings that kept their place and rule but changed their message.
    check_messages: usize,
    format_messages: usize,
    crashes: Changes<Crash>,
    /// Files whose formatted output differs, with the first line that differs.
    reformatted: Vec<(String, usize)>,
    /// Files only one of the runs has: the runs used different manifests.
    unmatched: Vec<String>,
}

/// What a sorted list gained and lost, counting duplicates.
struct Changes<T> {
    added: Vec<T>,
    removed: Vec<T>,
}

impl<T: Clone> Changes<T> {
    /// `base` and `head` must be sorted by `compare`.
    fn between(base: &[T], head: &[T], compare: impl Fn(&T, &T) -> Ordering) -> Self {
        let mut changes = Self {
            added: Vec::new(),
            removed: Vec::new(),
        };
        let (mut b, mut h) = (0, 0);
        while b < base.len() || h < head.len() {
            let order = match (base.get(b), head.get(h)) {
                (Some(old), Some(new)) => compare(old, new),
                (Some(_), None) => Ordering::Less,
                (None, _) => Ordering::Greater,
            };
            match order {
                Ordering::Less => {
                    changes.removed.extend(base.get(b).cloned());
                    b += 1;
                }
                Ordering::Greater => {
                    changes.added.extend(head.get(h).cloned());
                    h += 1;
                }
                Ordering::Equal => {
                    b += 1;
                    h += 1;
                }
            }
        }
        changes
    }

    fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }
}

impl Finding {
    /// Where and what, without the message.
    fn place(&self) -> (&str, usize, usize, &str, &str) {
        (
            &self.path,
            self.line,
            self.column,
            &self.category,
            &self.severity,
        )
    }

    fn cmp_place(&self, other: &Self) -> Ordering {
        self.place().cmp(&other.place())
    }
}

impl Diff {
    pub fn between(base_dir: &Path, head_dir: &Path) -> Result<Self> {
        let base = Results::load(base_dir)?;
        let head = Results::load(head_dir)?;

        let check = Changes::between(&base.check, &head.check, Finding::cmp_place);
        let check_messages = Changes::between(&base.check, &head.check, Ord::cmp)
            .added
            .len()
            .saturating_sub(check.added.len());
        let format = Changes::between(&base.format, &head.format, Finding::cmp_place);
        let format_messages = Changes::between(&base.format, &head.format, Ord::cmp)
            .added
            .len()
            .saturating_sub(format.added.len());
        let crashes = Changes::between(&base.crashes, &head.crashes, |old, new| {
            (&old.command, &old.path).cmp(&(&new.command, &new.path))
        });

        let mut reformatted = Vec::new();
        let mut unmatched = Vec::new();
        for source in &head.sources {
            let Some(old) = base.sources.iter().find(|old| old.name == source.name) else {
                unmatched.extend(source.files.iter().map(|f| format!("{}/{f}", source.name)));
                continue;
            };
            let files = Changes::between(&old.files, &source.files, Ord::cmp);
            unmatched.extend(
                files
                    .added
                    .iter()
                    .chain(&files.removed)
                    .map(|f| format!("{}/{f}", source.name)),
            );
            for file in &source.files {
                if files.added.binary_search(file).is_ok() {
                    continue;
                }
                let path = PathBuf::from(SOURCES_DIR).join(&source.name).join(file);
                let old_text = std::fs::read(base_dir.join(&path))?;
                let new_text = std::fs::read(head_dir.join(&path))?;
                if old_text == new_text {
                    continue;
                }
                let mut old_lines = old_text.split(|&b| b == b'\n');
                let mut new_lines = new_text.split(|&b| b == b'\n');
                let mut line = 1;
                while old_lines.next() == new_lines.next() {
                    line += 1;
                }
                reformatted.push((format!("{}/{file}", source.name), line));
            }
        }
        for source in &base.sources {
            if !head.sources.iter().any(|new| new.name == source.name) {
                unmatched.extend(source.files.iter().map(|f| format!("{}/{f}", source.name)));
            }
        }

        Ok(Self {
            base,
            head,
            check,
            format,
            check_messages,
            format_messages,
            crashes,
            reformatted,
            unmatched,
        })
    }

    pub fn markdown(&self, base_label: &str, head_label: &str) -> String {
        let mut out = String::new();
        let _ = self.write_markdown(&mut out, base_label, head_label);
        out
    }

    fn write_markdown(
        &self,
        out: &mut String,
        base_label: &str,
        head_label: &str,
    ) -> std::fmt::Result {
        writeln!(out, "## Ecosystem check: `{base_label}` → `{head_label}`\n")?;
        let unchanged = self.check.is_empty()
            && self.format.is_empty()
            && self.check_messages == 0
            && self.format_messages == 0
            && self.crashes.is_empty()
            && self.reformatted.is_empty()
            && self.unmatched.is_empty();
        let files: usize = self.head.sources.iter().map(|s| s.files.len()).sum();
        if unchanged {
            writeln!(
                out,
                "No changes on {files} files of {} sources.\n",
                self.head.sources.len()
            )?;
        }

        writeln!(
            out,
            "| Source | Licence | Commit | Files | `check` findings | `format` findings | Reformatted files | Crashes |"
        )?;
        writeln!(out, "|---|---|---|--:|--:|--:|--:|--:|")?;
        for source in &self.head.sources {
            let prefix = format!("{}/", source.name);
            let in_source = |path: &str| path.starts_with(&prefix);
            let count =
                |findings: &[Finding]| findings.iter().filter(|f| in_source(&f.path)).count();
            let crashes = |crashes: &[Crash]| crashes.iter().filter(|c| in_source(&c.path)).count();
            writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                source.name,
                source.licence,
                source
                    .rev
                    .as_deref()
                    .map(|rev| format!("`{}`", rev.get(..7).unwrap_or(rev)))
                    .unwrap_or_else(|| "in repo".to_string()),
                source.files.len(),
                Self::before_after(count(&self.base.check), count(&self.head.check)),
                Self::before_after(count(&self.base.format), count(&self.head.format)),
                self.reformatted
                    .iter()
                    .filter(|(path, _)| in_source(path))
                    .count(),
                Self::before_after(crashes(&self.base.crashes), crashes(&self.head.crashes)),
            )?;
        }
        writeln!(out)?;

        if !self.unmatched.is_empty() {
            writeln!(
                out,
                "The runs used different corpus files; {} files are only in one of them.\n",
                self.unmatched.len()
            )?;
        }

        if !self.crashes.is_empty() {
            writeln!(out, "### Crashes\n")?;
            Self::write_list(out, "New", &self.crashes.added, |crash| {
                format!("`{}` on `{}`: {}", crash.command, crash.path, crash.reason)
            })?;
            Self::write_list(out, "Fixed", &self.crashes.removed, |crash| {
                format!("`{}` on `{}`: {}", crash.command, crash.path, crash.reason)
            })?;
        }

        for (command, changes, messages, base, head) in [
            (
                "check",
                &self.check,
                self.check_messages,
                &self.base.check,
                &self.head.check,
            ),
            (
                "format",
                &self.format,
                self.format_messages,
                &self.base.format,
                &self.head.format,
            ),
        ] {
            if changes.is_empty() && messages == 0 {
                continue;
            }
            writeln!(out, "### `{command}` findings\n")?;
            writeln!(out, "| Rule | `{base_label}` | `{head_label}` |")?;
            writeln!(out, "|---|--:|--:|")?;
            for (rule, old, new) in Self::rule_counts(base, head) {
                if old != new {
                    writeln!(out, "| `{rule}` | {old} | {new} |")?;
                }
            }
            writeln!(out)?;
            let place = |finding: &Finding| {
                format!(
                    "`{}:{}:{}` `{}` ({})",
                    finding.path, finding.line, finding.column, finding.category, finding.severity
                )
            };
            Self::write_list(out, "Added", &changes.added, place)?;
            Self::write_list(out, "Removed", &changes.removed, place)?;
            if messages > 0 {
                writeln!(
                    out,
                    "{messages} findings kept their place and rule but changed their message.\n"
                )?;
            }
        }

        if !self.reformatted.is_empty() {
            writeln!(out, "### Formatted output\n")?;
            Self::write_list(out, "Changed", &self.reformatted, |(path, line)| {
                format!("`{path}` from line {line}")
            })?;
        }
        Ok(())
    }

    fn before_after(base: usize, head: usize) -> String {
        if base == head {
            base.to_string()
        } else {
            format!("{base} → **{head}**")
        }
    }

    fn write_list<T>(
        out: &mut String,
        title: &str,
        items: &[T],
        line: impl Fn(&T) -> String,
    ) -> std::fmt::Result {
        if items.is_empty() {
            return Ok(());
        }
        writeln!(out, "**{title} ({}):**\n", items.len())?;
        for item in items.iter().take(LISTED_MAX) {
            writeln!(out, "- {}", line(item))?;
        }
        if items.len() > LISTED_MAX {
            writeln!(out, "- … and {} more", items.len() - LISTED_MAX)?;
        }
        writeln!(out)
    }

    /// The number of findings per rule on each side, sorted by rule.
    fn rule_counts<'a>(base: &'a [Finding], head: &'a [Finding]) -> Vec<(&'a str, usize, usize)> {
        let mut rules: Vec<(&str, bool)> = base
            .iter()
            .map(|f| (f.category.as_str(), false))
            .chain(head.iter().map(|f| (f.category.as_str(), true)))
            .collect();
        rules.sort_unstable();
        let mut counts: Vec<(&str, usize, usize)> = Vec::new();
        for (rule, is_head) in rules {
            match counts.last_mut() {
                Some(last) if last.0 == rule => {}
                _ => counts.push((rule, 0, 0)),
            }
            if let Some(last) = counts.last_mut() {
                if is_head {
                    last.2 += 1;
                } else {
                    last.1 += 1;
                }
            }
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(path: &str, line: usize, category: &str, message: &str) -> Finding {
        Finding {
            path: path.to_string(),
            line,
            column: 1,
            category: category.to_string(),
            severity: "warning".to_string(),
            message: message.to_string(),
        }
    }

    #[test]
    fn changes_count_duplicates() {
        let base = [1, 2, 2, 3];
        let head = [2, 3, 3, 4];
        let changes = Changes::between(&base, &head, Ord::cmp);
        assert_eq!(changes.added, [3, 4]);
        assert_eq!(changes.removed, [1, 2]);
    }

    #[test]
    fn a_new_message_is_not_a_new_finding() {
        let base = [finding("a.sql", 1, "lint/x", "old")];
        let head = [finding("a.sql", 1, "lint/x", "new")];
        assert!(Changes::between(&base, &head, Finding::cmp_place).is_empty());
        assert_eq!(Changes::between(&base, &head, Ord::cmp).added.len(), 1);
    }

    #[test]
    fn rule_counts_merge_both_sides() {
        let base = [
            finding("a.sql", 1, "lint/x", ""),
            finding("a.sql", 2, "lint/y", ""),
        ];
        let head = [
            finding("a.sql", 1, "lint/x", ""),
            finding("a.sql", 3, "lint/x", ""),
            finding("a.sql", 4, "lint/z", ""),
        ];
        assert_eq!(
            Diff::rule_counts(&base, &head),
            [("lint/x", 1, 2), ("lint/y", 1, 0), ("lint/z", 0, 1)]
        );
    }
}
