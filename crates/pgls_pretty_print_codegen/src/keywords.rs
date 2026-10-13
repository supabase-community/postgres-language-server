// Adapted from Squawk's keyword generation helper.
use anyhow::{Context, Ok, Result};
use std::path;

fn parse_header() -> Result<Vec<(String, String)>> {
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
                [name, _value, category, _is_bare_label] => {
                    let name = name.trim().replace('\"', "");
                    keywords.push((name, category.trim().to_string()));
                }
                _ => anyhow::bail!("Problem reading kwlist.h row"),
            }
        }
    }

    Ok(keywords)
}

pub(crate) struct KeywordKinds {
    pub(crate) all_keywords: Vec<String>,
    /// Every keyword with its grammar category from kwlist.h, such as `RESERVED_KEYWORD`.
    pub(crate) categories: Vec<(String, String)>,
}

pub(crate) fn keyword_kinds() -> Result<KeywordKinds> {
    let mut categories = parse_header()?;
    categories.sort();
    let all_keywords = categories.iter().map(|(name, _)| name.clone()).collect();

    Ok(KeywordKinds {
        all_keywords,
        categories,
    })
}
