use pgls_cli::pg_l_s_command;
use std::{fs, path::Path};

use crate::utils;

pub fn generate_cli_doc(docs_dir: &Path) -> anyhow::Result<()> {
    let file_path = docs_dir.join("reference/cli.md");

    let content = fs::read_to_string(&file_path)?;

    let new_content = utils::replace_section(
        &content,
        "CLI_REF",
        &pg_l_s_command().render_markdown("postgres-language-server"),
    );

    fs::write(file_path, &new_content)?;

    Ok(())
}
