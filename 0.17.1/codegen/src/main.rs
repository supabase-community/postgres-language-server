use std::env;
use std::path::{Path, PathBuf};

use docs_codegen::cli_doc::generate_cli_doc;
use docs_codegen::default_configuration::generate_default_configuration;
use docs_codegen::env_variables::generate_env_variables;
use docs_codegen::rules_docs::generate_rules_docs;
use docs_codegen::rules_index::generate_rules_index;
use docs_codegen::rules_sources::generate_rule_sources;
use docs_codegen::schema::generate_schema;

fn docs_root() -> PathBuf {
    let dir =
        env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned());
    Path::new(&dir).parent().unwrap().to_path_buf()
}

fn main() -> anyhow::Result<()> {
    let docs_root = docs_root();

    generate_default_configuration(&docs_root)?;
    generate_env_variables(&docs_root)?;
    generate_cli_doc(&docs_root)?;
    generate_rules_docs(&docs_root)?;
    generate_rules_index(&docs_root)?;
    generate_rule_sources(&docs_root)?;
    generate_schema(&docs_root)?;

    Ok(())
}
