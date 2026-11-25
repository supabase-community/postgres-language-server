use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::utils::replace_section;

pub fn generate_env_variables(docs_dir: &Path) -> Result<()> {
    let file_path = docs_dir.join("reference/env_variables.md");

    let mut content = vec![];

    writeln!(content, "\n",)?;

    for variable in pgls_env::VARIABLES {
        writeln!(
            content,
            "### `{}`\n\n {}\n",
            variable.name(),
            variable.description()
        )?;
    }

    writeln!(content, "## Deprecated\n")?;
    writeln!(
        content,
        "The names from before the rename to Postgres Language Server still work. Use the `PGLS_*` names above instead.\n"
    )?;
    for variable in pgls_env::VARIABLES {
        if let Some(alias) = variable.deprecated_alias() {
            writeln!(
                content,
                "### `{alias}`\n\n Deprecated, use `{}` instead.\n",
                variable.name()
            )?;
        }
    }

    let data = fs::read_to_string(&file_path)?;

    let content = String::from_utf8(content)?;
    let new_data = replace_section(&data, "ENV_VARS", &content)?;

    fs::write(file_path, new_data)?;

    Ok(())
}
