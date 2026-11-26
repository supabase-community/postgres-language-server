//! Writes the seeds of the fuzz targets to `fuzz/seeds/inputs/`, which `just fuzz` runs first:
//! every statement of Postgres' regression suite of every recorded version, and runs of
//! consecutive statements with the text between them, for the splitter. The `.sql` and `.pg` files
//! under the directories given as arguments, e.g. the corpus test's sources in
//! `target/tmp/corpus`, are cut into statements by the splitter and added the same way; a
//! directory that doesn't exist is skipped. Nothing of it is committed: the regression SQL is in
//! the repository already.

use std::{collections::HashSet, fs, path::Path};

use pgls_text_size::TextRange;

/// The longest run of consecutive statements written as one seed, in bytes. Below the targets'
/// input limit.
const RUN_LEN_MAX: usize = 2000;

/// The most files read from the directories given as arguments.
const FILES_MAX: usize = 20_000;

fn main() {
    let inputs = Path::new(env!("CARGO_MANIFEST_DIR")).join("seeds/inputs");
    if inputs.exists() {
        fs::remove_dir_all(&inputs).expect("remove the old seeds");
    }
    fs::create_dir_all(&inputs).expect("create the seed directory");

    let mut seeds = Seeds::default();
    for version in pgls_postgres_regress::versions() {
        for file in version.files() {
            seeds.add(&file.source, file.statements.iter().map(|s| s.range));
        }
    }
    let mut dirs = std::env::args().skip(1).map(Into::into).collect::<Vec<_>>();
    let mut files = 0;
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for path in entries.filter_map(|entry| Some(entry.ok()?.path())) {
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if !path.extension().is_some_and(|e| e == "sql" || e == "pg") {
                continue;
            }
            // Files that aren't UTF-8 aren't seeds.
            let Ok(sql) = fs::read_to_string(&path) else {
                continue;
            };
            if files == FILES_MAX {
                eprintln!("seeds: read the first {FILES_MAX} files, skipping the rest");
                dirs.clear();
                break;
            }
            files += 1;
            seeds.add(
                &sql,
                pgls_statement_splitter::split(&sql).ranges.into_iter(),
            );
        }
    }
    let mut seeds = seeds.0.into_iter().collect::<Vec<_>>();
    seeds.sort();
    for (index, seed) in seeds.iter().enumerate() {
        fs::write(inputs.join(format!("{index:06}.sql")), seed).expect("write a seed");
    }

    println!(
        "{} seeds ({files} extra files) in {}",
        seeds.len(),
        inputs.display()
    );
}

/// The distinct seeds.
#[derive(Default)]
struct Seeds(HashSet<String>);

impl Seeds {
    /// Adds each statement of `source` and the runs of consecutive statements up to
    /// `RUN_LEN_MAX`.
    fn add(&mut self, source: &str, statements: impl Iterator<Item = TextRange>) {
        let mut run: Option<TextRange> = None;
        for statement in statements {
            self.0.insert(source[statement].to_owned());
            let longer = run.map_or(statement, |run| run.cover(statement));
            if usize::from(longer.len()) <= RUN_LEN_MAX {
                run = Some(longer);
                continue;
            }
            if let Some(run) = run {
                self.0.insert(source[run].to_owned());
            }
            run = Some(statement);
        }
        if let Some(run) = run {
            self.0.insert(source[run].to_owned());
        }
    }
}
