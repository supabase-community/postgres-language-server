//! `corpus.toml`: the sources, their pinned commits and licences, and fetching them into the cache.

use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

/// The licences a source may have: permissive ones, so that a repro minimised from a source can
/// become a test of this repository. SPDX ids.
const LICENCES_PERMISSIVE: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "PostgreSQL",
];

/// The SQL files of one source. The largest source has about 300; a pattern that matches far more
/// than that is a mistake in the manifest, and the run would take hours.
const FILES_MAX: usize = 5_000;

/// How deep a source's directories may nest. Real repositories stay under 15.
const DEPTH_MAX: usize = 32;

/// Written next to a checkout once it is complete, so an interrupted fetch is redone.
const COMPLETE_SUFFIX: &str = ".complete";

/// The parsed manifest.
pub struct Corpus {
    /// The manifest file, which `bump` rewrites.
    path: PathBuf,
    /// The root of the repository, which `path` sources are relative to.
    root: PathBuf,
    pub sources: Vec<Source>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    #[serde(rename = "source")]
    sources: Vec<Source>,
}

#[derive(Deserialize)]
pub struct Source {
    /// A path segment: the source's directory in the cache and the prefix of its files in reports.
    pub name: String,
    /// The SPDX id of the licence of the included files.
    pub licence: String,
    #[serde(flatten)]
    pub origin: Origin,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Origin {
    /// A repository, downloaded at a pinned commit.
    Git {
        git: String,
        /// The branch `bump` moves the pin to.
        branch: String,
        /// The pinned commit, a full SHA.
        rev: String,
        /// The files to download, as gitignore patterns (`git sparse-checkout --no-cone`).
        include: Vec<String>,
    },
    /// A directory of this repository.
    Path { path: PathBuf },
}

impl Origin {
    /// The pinned commit of a git source.
    pub fn rev(&self) -> Option<&str> {
        match self {
            Origin::Git { rev, .. } => Some(rev),
            Origin::Path { .. } => None,
        }
    }
}

impl Corpus {
    pub fn load(path: &Path, root: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading the manifest {}", path.display()))?;
        let manifest: Manifest =
            toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        for (i, source) in manifest.sources.iter().enumerate() {
            source.validate()?;
            ensure!(
                manifest.sources[..i].iter().all(|s| s.name != source.name),
                "two sources are named {:?}",
                source.name
            );
        }
        Ok(Self {
            path: path.to_path_buf(),
            root: root.to_path_buf(),
            sources: manifest.sources,
        })
    }

    /// Downloads every git source that isn't in the cache yet, and removes the other commits of
    /// each source from the cache.
    pub fn fetch(&self, cache: &Path) -> Result<()> {
        for source in &self.sources {
            source.fetch(cache)?;
        }
        Ok(())
    }

    /// The directory with a source's files: its checkout in the cache, or its directory in this
    /// repository.
    pub fn dir(&self, source: &Source, cache: &Path) -> PathBuf {
        match &source.origin {
            Origin::Git { rev, .. } => cache.join(&source.name).join(rev),
            Origin::Path { path } => self.root.join(path),
        }
    }

    /// Moves the pin of every git source to the head of its branch, keeping the rest of the
    /// manifest as written.
    pub fn bump(&self) -> Result<()> {
        let mut text = std::fs::read_to_string(&self.path)
            .with_context(|| format!("reading the manifest {}", self.path.display()))?;
        for source in &self.sources {
            let Origin::Git {
                git, branch, rev, ..
            } = &source.origin
            else {
                continue;
            };
            let refname = format!("refs/heads/{branch}");
            let listing = Git::run(None, &["ls-remote", git, &refname])?;
            let Some(head) = listing.split_whitespace().next() else {
                bail!("{}: {git} has no branch {branch}", source.name);
            };
            ensure!(
                Source::is_full_sha(head),
                "{}: `git ls-remote` printed {listing:?}",
                source.name
            );
            if head == rev {
                eprintln!("{}: up to date", source.name);
                continue;
            }
            // A full SHA appears once in the manifest, as the pin of its source.
            text = text.replace(&format!("\"{rev}\""), &format!("\"{head}\""));
            eprintln!("{}: {rev} -> {head}", source.name);
        }
        std::fs::write(&self.path, text)
            .with_context(|| format!("writing the manifest {}", self.path.display()))
    }
}

impl Source {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.name.is_empty()
                && self.name.bytes().all(|b| b.is_ascii_lowercase()
                    || b.is_ascii_digit()
                    || b == b'-'
                    || b == b'_'),
            "source name {:?}: use lowercase letters, digits, `-` and `_`",
            self.name
        );
        ensure!(
            LICENCES_PERMISSIVE.contains(&self.licence.as_str()),
            "{}: licence {:?} is not one of {LICENCES_PERMISSIVE:?}. Copyleft SQL can't become a \
             test of this repository without rewriting the repro first.",
            self.name,
            self.licence
        );
        match &self.origin {
            Origin::Git { rev, include, .. } => {
                ensure!(
                    Self::is_full_sha(rev),
                    "{}: `rev` must be a full commit SHA, not {rev:?}",
                    self.name
                );
                ensure!(!include.is_empty(), "{}: `include` is empty", self.name);
            }
            Origin::Path { path } => {
                ensure!(
                    path.components().all(|c| matches!(c, Component::Normal(_))),
                    "{}: `path` must be relative to the repository root, without `..`",
                    self.name
                );
            }
        }
        Ok(())
    }

    fn is_full_sha(rev: &str) -> bool {
        rev.len() == 40 && rev.bytes().all(|b| b.is_ascii_hexdigit())
    }

    /// Downloads only the included files of the pinned commit: a shallow fetch without blobs,
    /// then a sparse checkout, which fetches the blobs it needs. The `.git` directory is removed
    /// afterwards; the checkout is plain files.
    fn fetch(&self, cache: &Path) -> Result<()> {
        let Origin::Git {
            git, rev, include, ..
        } = &self.origin
        else {
            return Ok(());
        };
        let source_dir = cache.join(&self.name);
        let dir = source_dir.join(rev);
        let complete = source_dir.join(format!("{rev}{COMPLETE_SUFFIX}"));

        if complete.exists() {
            eprintln!("{}: cached at {}", self.name, dir.display());
        } else {
            eprintln!("{}: fetching {git} at {rev}", self.name);
            if dir.exists() {
                std::fs::remove_dir_all(&dir)
                    .with_context(|| format!("removing the partial checkout {}", dir.display()))?;
            }
            std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;

            let run = |args: &[&str]| Git::run(Some(&dir), args);
            run(&["init", "--quiet"])?;
            run(&["remote", "add", "origin", git])?;
            let mut sparse = vec!["sparse-checkout", "set", "--no-cone"];
            sparse.extend(include.iter().map(String::as_str));
            run(&sparse)?;
            run(&[
                "fetch",
                "--quiet",
                "--no-tags",
                "--depth=1",
                "--filter=blob:none",
                "origin",
                rev,
            ])?;
            run(&["checkout", "--quiet", "--detach", "FETCH_HEAD"])?;
            let head = run(&["rev-parse", "HEAD"])?;
            ensure!(
                head.trim() == rev,
                "{}: fetched {} instead of the pinned {rev}",
                self.name,
                head.trim()
            );
            std::fs::remove_dir_all(dir.join(".git"))
                .with_context(|| format!("removing {}/.git", dir.display()))?;
            std::fs::write(&complete, "")
                .with_context(|| format!("writing {}", complete.display()))?;
        }

        // Other commits of this source, from before a bump.
        for entry in std::fs::read_dir(&source_dir)
            .with_context(|| format!("listing {}", source_dir.display()))?
        {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let stale = name.strip_suffix(COMPLETE_SUFFIX).unwrap_or(&name) != rev;
            if !stale {
                continue;
            }
            let path = entry.path();
            eprintln!("{}: removing the stale {}", self.name, path.display());
            if entry.file_type()?.is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }
        }
        Ok(())
    }

    /// The `.sql` files under `dir`, relative to it with `/` separators, sorted.
    pub fn files(&self, dir: &Path) -> Result<Vec<String>> {
        ensure!(
            dir.is_dir(),
            "{}: {} doesn't exist. Run `fetch` first.",
            self.name,
            dir.display()
        );
        let mut files = Vec::new();
        let mut pending = vec![(dir.to_path_buf(), 0usize)];
        while let Some((current, depth)) = pending.pop() {
            for entry in std::fs::read_dir(&current)
                .with_context(|| format!("listing {}", current.display()))?
            {
                let entry = entry?;
                let file_type = entry.file_type()?;
                let path = entry.path();
                if file_type.is_dir() {
                    if depth + 1 > DEPTH_MAX {
                        eprintln!(
                            "{}: skipping {}, deeper than {DEPTH_MAX} directories",
                            self.name,
                            path.display()
                        );
                        continue;
                    }
                    pending.push((path, depth + 1));
                } else if file_type.is_file()
                    && path.extension().is_some_and(|extension| extension == "sql")
                {
                    if files.len() == FILES_MAX {
                        bail!(
                            "{}: more than {FILES_MAX} files; narrow its `include`",
                            self.name
                        );
                    }
                    let relative = path.strip_prefix(dir).unwrap_or(&path);
                    let relative = relative
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy())
                        .collect::<Vec<_>>()
                        .join("/");
                    files.push(relative);
                }
            }
        }
        files.sort_unstable();
        Ok(files)
    }
}

/// The `git` executable.
struct Git;

impl Git {
    /// Runs git in `dir` (else in the current directory) and returns its stdout.
    fn run(dir: Option<&Path>, args: &[&str]) -> Result<String> {
        let mut command = Command::new("git");
        if let Some(dir) = dir {
            command.current_dir(dir);
        }
        let output = command
            .args(args)
            .output()
            .with_context(|| format!("running git {}", args.join(" ")))?;
        ensure!(
            output.status.success(),
            "git {} failed with {}: {}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(text: &str) -> Result<Corpus> {
        let dir = std::env::temp_dir().join(format!("xtask_corpus_{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{:x}.toml", text.len()));
        std::fs::write(&path, text)?;
        let corpus = Corpus::load(&path, &dir);
        std::fs::remove_file(&path)?;
        corpus
    }

    #[test]
    fn the_manifest_is_valid() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let corpus = Corpus::load(&root.join("corpus.toml"), root).unwrap();
        assert!(!corpus.sources.is_empty());
    }

    #[test]
    fn rejects_copyleft_licences() {
        let error = load(
            r#"
            [[source]]
            name = "calcom"
            licence = "AGPL-3.0"
            git = "https://github.com/calcom/cal.com.git"
            branch = "main"
            rev = "0123456789abcdef0123456789abcdef01234567"
            include = ["/packages/prisma/migrations/**/*.sql"]
            "#,
        )
        .err()
        .unwrap();
        assert!(error.to_string().contains("AGPL-3.0"), "{error}");
    }

    #[test]
    fn rejects_unpinned_sources() {
        let error = load(
            r#"
            [[source]]
            name = "sqlfluff"
            licence = "MIT"
            git = "https://github.com/sqlfluff/sqlfluff.git"
            branch = "main"
            rev = "main"
            include = ["/test/*.sql"]
            "#,
        )
        .err()
        .unwrap();
        assert!(error.to_string().contains("full commit SHA"), "{error}");
    }

    #[test]
    fn rejects_paths_outside_the_repository() {
        let error = load(
            r#"
            [[source]]
            name = "outside"
            licence = "MIT"
            path = "../elsewhere"
            "#,
        )
        .err()
        .unwrap();
        assert!(error.to_string().contains("without `..`"), "{error}");
    }
}
