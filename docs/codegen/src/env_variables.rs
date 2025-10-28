use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::utils::replace_section;

pub fn generate_env_variables(docs_dir: &Path) -> Result<()> {
    let file_path = docs_dir.join("reference/env_variables.md");

    let mut content = vec![];

    let env = pgls_env::pgls_env();

    writeln!(content, "\n",)?;

    writeln!(
        content,
        "### `{}`\n\n {}\n",
        env.pgls_log_path.name(),
        env.pgls_log_path.description()
    )?;
    writeln!(
        content,
        "### `{}`\n\n {}\n",
        env.pgls_log_level.name(),
        env.pgls_log_level.description()
    )?;
    writeln!(
        content,
        "### `{}`\n\n {}\n",
        env.pgls_log_prefix.name(),
        env.pgls_log_prefix.description()
    )?;
    writeln!(
        content,
        "### `{}`\n\n {}\n",
        env.pgls_config_path.name(),
        env.pgls_config_path.description()
    )?;

    writeln!(
        content,
        "### `{}`\n\n {}\n",
        env.pgt_log_path.name(),
        env.pgt_log_path.description()
    )?;
    writeln!(
        content,
        "### `{}`\n\n {}\n",
        env.pgt_log_level.name(),
        env.pgt_log_level.description()
    )?;
    writeln!(
        content,
        "### `{}`\n\n {}\n",
        env.pgt_log_prefix.name(),
        env.pgt_log_prefix.description()
    )?;
    writeln!(
        content,
        "### `{}`\n\n {}\n",
        env.pgt_config_path.name(),
        env.pgt_config_path.description()
    )?;

    let data = fs::read_to_string(&file_path)?;

    let conent_str = String::from_utf8(content)?;
    let new_data = replace_section(&data, "ENV_VARS", &conent_str);

    fs::write(file_path, new_data)?;

    Ok(())
}
