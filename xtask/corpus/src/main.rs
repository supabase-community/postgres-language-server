//! The ecosystem check: runs two builds of the CLI over real-world SQL and reports what changed.
//!
//! The third-party SQL stays out of the repository. `corpus.toml` pins each source to a commit and
//! names its licence, `fetch` downloads the sources into a cache, `run` runs `check` and `format`
//! of one binary over them, and `diff` compares two runs as Markdown with paths, rules and lines,
//! never source text.

mod diff;
mod manifest;
mod run;

use anyhow::{Context, Result};
use bpaf::Bpaf;
use std::path::{Path, PathBuf};

use crate::diff::Diff;
use crate::manifest::Corpus;
use crate::run::Results;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
enum Task {
    /// Downloads every source of the manifest at its pinned commit into the cache
    #[bpaf(command)]
    Fetch {
        #[bpaf(external(location))]
        location: Location,
    },
    /// Runs `check` and `format` of a CLI binary over the fetched corpus, and writes the results
    /// and the formatted files into a new directory
    #[bpaf(command)]
    Run {
        #[bpaf(external(location))]
        location: Location,
        /// The `postgres-language-server` binary to run
        #[bpaf(long("binary"), argument("PATH"))]
        binary: PathBuf,
        /// The directory to write into. It must not exist or be empty
        #[bpaf(long("out"), argument("DIR"))]
        out: PathBuf,
    },
    /// Prints the differences between two runs as Markdown
    #[bpaf(command)]
    Diff {
        /// The output directory of the baseline run
        #[bpaf(long("base"), argument("DIR"))]
        base: PathBuf,
        /// The output directory of the run to compare
        #[bpaf(long("head"), argument("DIR"))]
        head: PathBuf,
        /// What to call the baseline in the report, e.g. a commit or a release
        #[bpaf(long("base-label"), argument("NAME"), fallback("base".to_string()))]
        base_label: String,
        /// What to call the compared run in the report
        #[bpaf(long("head-label"), argument("NAME"), fallback("head".to_string()))]
        head_label: String,
    },
    /// Moves the pin of every git source to the head of its branch
    #[bpaf(command)]
    Bump {
        #[bpaf(external(location))]
        location: Location,
    },
}

#[derive(Debug, Clone, Bpaf)]
struct Location {
    /// The manifest of the sources. Default: `xtask/corpus/corpus.toml`
    #[bpaf(long("manifest"), argument("PATH"), optional)]
    manifest: Option<PathBuf>,
    /// Where the sources are downloaded to. Default: `$PGLS_CORPUS_CACHE`, else
    /// `$XDG_CACHE_HOME/pgls-corpus`, else `~/.cache/pgls-corpus`
    #[bpaf(long("cache"), env("PGLS_CORPUS_CACHE"), argument("DIR"), optional)]
    cache: Option<PathBuf>,
}

impl Location {
    /// The root of the repository, which `path` sources are relative to.
    fn project_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap_or(Path::new("."))
    }

    fn corpus(&self) -> Result<Corpus> {
        let root = Self::project_root();
        let manifest = self
            .manifest
            .clone()
            .unwrap_or_else(|| root.join("xtask/corpus/corpus.toml"));
        Corpus::load(&manifest, root)
    }

    fn cache(&self) -> Result<PathBuf> {
        if let Some(cache) = &self.cache {
            return Ok(cache.clone());
        }
        if let Some(xdg) = std::env::var_os("XDG_CACHE_HOME").filter(|dir| !dir.is_empty()) {
            return Ok(PathBuf::from(xdg).join("pgls-corpus"));
        }
        let home = std::env::var_os("HOME")
            .context("neither --cache, $PGLS_CORPUS_CACHE, $XDG_CACHE_HOME nor $HOME is set")?;
        Ok(PathBuf::from(home).join(".cache/pgls-corpus"))
    }
}

fn main() -> Result<()> {
    match task().run() {
        Task::Fetch { location } => location.corpus()?.fetch(&location.cache()?),
        Task::Run {
            location,
            binary,
            out,
        } => {
            let results = Results::collect(&location.corpus()?, &location.cache()?, &binary, &out)?;
            eprintln!(
                "{} files: {} check findings, {} format findings, {} crashes",
                results.sources.iter().map(|s| s.files.len()).sum::<usize>(),
                results.check.len(),
                results.format.len(),
                results.crashes.len(),
            );
            Ok(())
        }
        Task::Diff {
            base,
            head,
            base_label,
            head_label,
        } => {
            let diff = Diff::between(&base, &head)?;
            print!("{}", diff.markdown(&base_label, &head_label));
            Ok(())
        }
        Task::Bump { location } => location.corpus()?.bump(),
    }
}
