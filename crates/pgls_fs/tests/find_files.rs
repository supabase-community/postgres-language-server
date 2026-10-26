use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use pgls_fs::{ConfigName, FileSystem, FindFiles, OsFileSystem, SkipDirectory};

/// Creates `files` below a new temporary directory.
fn tree(files: &[&str]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for file in files {
        let path = root.path().join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "{}").unwrap();
    }
    root
}

/// The files the search finds, relative to `root`, and whether it stopped at a limit.
fn find(root: &Path, search: FindFiles) -> (Vec<String>, bool) {
    let found = OsFileSystem::new(root.to_path_buf()).find_files(root, &search);
    let mut files: Vec<_> = found
        .files
        .iter()
        .map(|file| {
            file.strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    files.sort();
    (files, found.truncated)
}

const CONFIG: &[&str] = &["postgres-language-server.jsonc"];

#[test]
fn skips_hidden_vendored_and_ignored_folders() {
    let root = tree(&[
        "postgres-language-server.jsonc",
        "db1/postgres-language-server.jsonc",
        ".hidden/postgres-language-server.jsonc",
        "node_modules/pkg/postgres-language-server.jsonc",
        "target/postgres-language-server.jsonc",
        "build/postgres-language-server.jsonc",
        "db1/other.jsonc",
    ]);
    std::fs::write(root.path().join(".gitignore"), "build/\n").unwrap();

    let (files, truncated) = find(root.path(), FindFiles::new(CONFIG));
    assert_eq!(
        files,
        [
            "db1/postgres-language-server.jsonc",
            "postgres-language-server.jsonc"
        ]
    );
    assert!(!truncated);
}

/// A skipped directory is not searched, unless a directory between the root and it has one of the
/// files: the file there decides what applies below it.
#[test]
fn skips_directories_up_to_the_first_file() {
    let root = tree(&[
        "legacy/app/postgres-language-server.jsonc",
        "db1/postgres-language-server.jsonc",
        "db1/legacy/postgres-language-server.jsonc",
    ]);
    let skip_directory: SkipDirectory = Arc::new(|directory: &Path| directory.ends_with("legacy"));

    let (files, _) = find(
        root.path(),
        FindFiles {
            skip_directory: Some(skip_directory),
            ..FindFiles::new(CONFIG)
        },
    );
    assert_eq!(
        files,
        [
            "db1/legacy/postgres-language-server.jsonc",
            "db1/postgres-language-server.jsonc"
        ]
    );
}

#[test]
fn stops_at_the_limits() {
    let root = tree(&[
        "a/postgres-language-server.jsonc",
        "a/b/postgres-language-server.jsonc",
        "a/b/c/postgres-language-server.jsonc",
    ]);

    let (files, truncated) = find(
        root.path(),
        FindFiles {
            max_depth: 2,
            ..FindFiles::new(CONFIG)
        },
    );
    assert_eq!(
        files,
        [
            "a/b/postgres-language-server.jsonc",
            "a/postgres-language-server.jsonc"
        ]
    );
    assert!(truncated);

    let (files, truncated) = find(
        root.path(),
        FindFiles {
            max_entries: 2,
            ..FindFiles::new(CONFIG)
        },
    );
    assert!(files.len() < 3, "{files:?}");
    assert!(truncated);
}

/// Measures the search that nested configuration discovery runs on every configuration load. Run
/// it with
/// `PGLS_FIND_FILES_ROOT=/path/to/repo cargo test -p pgls_fs --release --test find_files -- --ignored --nocapture`.
/// With `PGLS_FIND_FILES_SKIP=<name>`, it skips the directories with that name, like a root
/// configuration that ignores them.
#[test]
#[ignore]
fn measure_find_files() {
    let root = PathBuf::from(std::env::var("PGLS_FIND_FILES_ROOT").expect("PGLS_FIND_FILES_ROOT"));
    let fs = OsFileSystem::new(root.clone());
    let names = ConfigName::file_names();
    let runs: u32 = std::env::var("PGLS_FIND_FILES_RUNS")
        .ok()
        .and_then(|runs| runs.parse().ok())
        .unwrap_or(5);
    let skip_directory = std::env::var("PGLS_FIND_FILES_SKIP")
        .ok()
        .map(|name| Arc::new(move |directory: &Path| directory.ends_with(&name)) as SkipDirectory);
    let search = FindFiles {
        skip_directory,
        ..FindFiles::new(&names)
    };
    // Warm the page cache once, then measure.
    let _ = fs.find_files(&root, &search);
    let mut found = Default::default();
    let start = Instant::now();
    for _ in 0..runs {
        found = fs.find_files(&root, &search);
    }
    let found: pgls_fs::FoundFiles = found;
    println!(
        "find_files({}) = {:?} per run, found {} files, truncated: {}",
        root.display(),
        start.elapsed() / runs,
        found.files.len(),
        found.truncated,
    );
}
