// from https://github.com/sbdchd/squawk/blob/ac9f90c3b2be8d2c46fd5454eb48975afd268dbe/crates/xtask/src/keywords.rs
use anyhow::{Context, Ok, Result};
use std::path;

fn parse_header() -> Result<Vec<String>> {
    // use the environment variable set by the build script to locate the kwlist.h file
    let kwlist_file = path::PathBuf::from(env!("PG_QUERY_KWLIST_PATH"));
    let data = std::fs::read_to_string(kwlist_file).context("Failed to read kwlist.h")?;

    let mut keywords = Vec::new();

    for line in data.lines() {
        if line.starts_with("PG_KEYWORD") {
            let line = line
                .split(&['(', ')'])
                .nth(1)
                .context("Invalid kwlist.h structure")?;

            let row_items: Vec<&str> = line.split(',').collect();

            match row_items[..] {
                [name, _value, _category, _is_bare_label] => {
                    let name = name.trim().replace('\"', "");
                    keywords.push(name);
                }
                _ => anyhow::bail!("Problem reading kwlist.h row"),
            }
        }
    }

    Ok(keywords)
}

pub(crate) struct KeywordKinds {
    pub(crate) all_keywords: Vec<String>,
}

pub(crate) fn keyword_kinds() -> Result<KeywordKinds> {
    let mut all_keywords = parse_header()?;
    all_keywords.sort();

    Ok(KeywordKinds { all_keywords })
}
