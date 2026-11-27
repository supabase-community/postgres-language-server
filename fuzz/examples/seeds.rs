//! Writes the seeds of the fuzz targets to `fuzz/seeds/inputs/`, which `just fuzz` runs first:
//! every statement of Postgres' regression suite of every recorded version, and runs of
//! consecutive statements with the text between them, for the splitter. Nothing of it is
//! committed: the regression SQL is in the repository already.

use std::{collections::HashSet, fs, path::Path};

use pgls_text_size::TextRange;

/// The longest run of consecutive statements written as one seed, in bytes. Below the targets'
/// input limit.
const RUN_LEN_MAX: usize = 2000;

fn main() {
    let inputs = Path::new(env!("CARGO_MANIFEST_DIR")).join("seeds/inputs");
    if inputs.exists() {
        fs::remove_dir_all(&inputs).expect("remove the old seeds");
    }
    fs::create_dir_all(&inputs).expect("create the seed directory");

    let mut seen = HashSet::new();
    for version in pgls_postgres_regress::versions() {
        for file in version.files() {
            let mut run: Option<TextRange> = None;
            for statement in &file.statements {
                seen.insert(file.sql(statement).to_owned());
                let longer = run.map_or(statement.range, |run| run.cover(statement.range));
                if usize::from(longer.len()) <= RUN_LEN_MAX {
                    run = Some(longer);
                    continue;
                }
                if let Some(run) = run {
                    seen.insert(file.source[run].to_owned());
                }
                run = Some(statement.range);
            }
            if let Some(run) = run {
                seen.insert(file.source[run].to_owned());
            }
        }
    }
    let mut seeds = seen.into_iter().collect::<Vec<_>>();
    seeds.sort();
    for (index, seed) in seeds.iter().enumerate() {
        fs::write(inputs.join(format!("{index:06}.sql")), seed).expect("write a seed");
    }

    println!("{} seeds in {}", seeds.len(), inputs.display());
}
