use std::env;
use std::fmt::Write;
use std::path::PathBuf;

/// Writes `Keyword::ALL` from `kwlist.h` in the libpg_query submodule that `pgls_query` builds, so
/// the keywords always match the parser.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let kwlist_path = manifest_dir
        .join("../pgls_query/vendor/libpg_query/src/postgres/include/parser/kwlist.h")
        .canonicalize()
        .map_err(|error| {
            format!("kwlist.h not found ({error}). Run: git submodule update --init --recursive")
        })?;
    println!("cargo:rerun-if-changed={}", kwlist_path.display());

    // A row is `PG_KEYWORD("abort", ABORT_P, UNRESERVED_KEYWORD, BARE_LABEL)`.
    let mut keywords = Vec::new();
    for line in std::fs::read_to_string(&kwlist_path)?.lines() {
        let Some(row) = line.strip_prefix("PG_KEYWORD(") else {
            continue;
        };
        let fields: Vec<&str> = row.split([',', ')']).map(str::trim).collect();
        let [name, _token, category, _label, ..] = fields[..] else {
            return Err(format!("unexpected kwlist.h row: {line}").into());
        };
        let category = match category {
            "UNRESERVED_KEYWORD" => "Unreserved",
            "COL_NAME_KEYWORD" => "ColName",
            "TYPE_FUNC_NAME_KEYWORD" => "TypeFuncName",
            "RESERVED_KEYWORD" => "Reserved",
            other => return Err(format!("unknown keyword category {other}").into()),
        };
        keywords.push((name.trim_matches('"').to_string(), category));
    }
    // `KeywordCategory::of` searches the list by bytes; Postgres keeps it sorted by `strcmp`.
    keywords.sort();

    let mut out = String::from(
        "impl Keyword {\n    /// Every keyword of `kwlist.h`, sorted by name.\n    pub const ALL: &[Keyword] = &[\n",
    );
    for (name, category) in keywords {
        writeln!(
            out,
            "        Keyword {{ name: {name:?}, category: KeywordCategory::{category} }},"
        )?;
    }
    out.push_str("    ];\n}\n");
    std::fs::write(PathBuf::from(env::var("OUT_DIR")?).join("keywords.rs"), out)?;

    Ok(())
}
