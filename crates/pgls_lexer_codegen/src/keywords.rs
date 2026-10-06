// Adapted from Squawk's keyword generation helper.
use anyhow::{Context, Ok, Result};
use std::path;

pub(crate) struct Keyword {
    pub(crate) name: String,
    /// The category in kwlist.h, e.g. `UNRESERVED_KEYWORD`.
    pub(crate) category: String,
}

fn parse_header() -> Result<Vec<Keyword>> {
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
                    keywords.push(Keyword {
                        name: name.trim().replace('\"', ""),
                        category: category.trim().to_string(),
                    });
                }
                _ => anyhow::bail!("Problem reading kwlist.h row"),
            }
        }
    }

    Ok(keywords)
}

pub(crate) struct KeywordKinds {
    pub(crate) all_keywords: Vec<Keyword>,
}

pub(crate) fn keyword_kinds() -> Result<KeywordKinds> {
    let mut all_keywords = parse_header()?;
    all_keywords.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(KeywordKinds { all_keywords })
}
