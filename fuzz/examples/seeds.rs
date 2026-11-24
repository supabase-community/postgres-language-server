//! Writes the seeds of the fuzz targets to `fuzz/seeds/`, which `just fuzz` runs first:
//!
//! - `inputs/`: every statement of Postgres' regression suite of every recorded version, and runs
//!   of consecutive statements with the text between them, for the splitter
//! - `keywords.dict`: a libFuzzer dictionary of Postgres' keywords and the tokens the lexer and
//!   the splitter treat specially
//!
//! Nothing of it is committed: the regression SQL is in the repository already.

use std::{collections::HashSet, fs, path::Path};

use pgls_text_size::TextRange;

/// The longest run of consecutive statements written as one seed, in bytes. Below the targets'
/// input limit.
const RUN_LEN_MAX: usize = 2000;

/// Tokens of the dictionary besides the keywords, in libFuzzer's syntax, which escapes `\` and `"`
/// and writes other bytes as `\xNN`.
const TOKENS: &[&str] = &[
    ";",
    "$$",
    "$f$",
    "E'",
    "U&\\\"",
    "U&'",
    "B'",
    "X'",
    "'",
    "\\\"",
    "::",
    "--",
    "/*",
    "*/",
    "\\x0A",
    "\\x0D\\x0A",
    "\\x0D",
    "\\x0A\\x0A",
    "begin atomic",
    "create schema",
    "$1",
    ":name",
    "@name",
    "\\\\",
];

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("seeds");
    let inputs = dir.join("inputs");
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

    let kwlist = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../crates/pgls_query/vendor/libpg_query/src/postgres/include/parser/kwlist.h");
    let kwlist = fs::read_to_string(&kwlist)
        .unwrap_or_else(|e| panic!("read {}: {e}; run `just setup` first", kwlist.display()));
    let mut dictionary = String::new();
    // Lines like `PG_KEYWORD("abort", ABORT_P, UNRESERVED_KEYWORD, BARE_LABEL)`.
    for keyword in kwlist.lines().filter_map(|line| {
        line.strip_prefix("PG_KEYWORD(\"")?
            .split_once('"')
            .map(|(keyword, _)| keyword)
    }) {
        dictionary.push_str(&format!("\"{keyword}\"\n"));
    }
    for token in TOKENS {
        dictionary.push_str(&format!("\"{token}\"\n"));
    }
    fs::write(dir.join("keywords.dict"), dictionary).expect("write the dictionary");

    println!("{} seeds in {}", seeds.len(), inputs.display());
}
