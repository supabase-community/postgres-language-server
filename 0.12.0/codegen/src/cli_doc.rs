use pgt_cli::pgt_command;
use std::{fs, path::Path};

use crate::utils;

pub fn generate_cli_doc(docs_dir: &Path) -> anyhow::Result<()> {
    let file_path = docs_dir.join("cli_reference.md");

    let content = fs::read_to_string(&file_path)?;

    let new_content = utils::replace_section(
        &content,
        "CLI_REF",
        &pgt_command().render_markdown("postgrestools"),
    );

    fs::write(file_path, &new_content)?;

    Ok(())
}
