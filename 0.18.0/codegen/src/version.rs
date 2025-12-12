use pgls_env::VERSION;
use std::{fs, path::Path};

use regex::Regex;

pub fn replace_version(docs_dir: &Path) -> anyhow::Result<()> {
    let index_path = docs_dir.join("getting_started.md");

    let data = fs::read_to_string(&index_path)?;

    let version_pattern = Regex::new(r"\$\{PGLS_VERSION\}").unwrap();
    let new_data = version_pattern.replace_all(&data, VERSION);

    fs::write(&index_path, new_data.as_ref())?;

    Ok(())
}
