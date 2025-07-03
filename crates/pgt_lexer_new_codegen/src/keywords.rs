// from https://github.com/sbdchd/squawk/blob/ac9f90c3b2be8d2c46fd5454eb48975afd268dbe/crates/xtask/src/keywords.rs
use anyhow::{Context, Ok, Result};
use enum_iterator::{Sequence, all};
use std::{
    collections::{HashMap, HashSet},
    path,
};

struct KeywordMeta {
    pub(crate) category: KeywordCategory,
    pub(crate) label: KeywordLabel,
}

enum KeywordLabel {
    As,
    Bare,
}

/// related:
///   - [postgres/src/backend/utils/adt/misc.c](https://github.com/postgres/postgres/blob/08691ea958c2646b6aadefff878539eb0b860bb0/src/backend/utils/adt/misc.c#L452-L467/)
///   - [postgres docs: sql keywords appendix](https://www.postgresql.org/docs/17/sql-keywords-appendix.html)
///
/// The header file isn't enough though because `json_scalar` can be a function
/// name, but `between` cannot be
///
/// The Postgres parser special cases certain calls like `json_scalar`:
/// <https://github.com/postgres/postgres/blob/028b4b21df26fee67b3ce75c6f14fcfd3c7cf2ee/src/backend/parser/gram.y#L15684C8-L16145>
///
/// | Category     | Column | Table | Function | Type |
/// |--------------|--------|-------|----------|------|
/// | Unreserved   | Y      | Y     | Y        | Y    |
/// | Reserved     | N      | N     | N        | N    |
/// | ColName      | Y      | Y     | N        | Y    |
/// | TypeFuncName | N      | N     | Y        | Y    |
///
#[derive(Clone, Copy)]
enum KeywordCategory {
    Unreserved,
    Reserved,
    ColName,
    TypeFuncName,
}

#[derive(Sequence, PartialEq)]
enum KWType {
    ColumnTable,
    Type,
}

impl std::fmt::Display for KWType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            KWType::ColumnTable => "COLUMN_OR_TABLE_KEYWORDS",
            KWType::Type => "TYPE_KEYWORDS",
        })
    }
}

fn keyword_allowed(cat: KeywordCategory, kw_type: KWType) -> bool {
    match cat {
        KeywordCategory::Unreserved => match kw_type {
            KWType::ColumnTable => true,
            KWType::Type => true,
        },
        KeywordCategory::Reserved => match kw_type {
            KWType::ColumnTable => false,
            KWType::Type => false,
        },
        KeywordCategory::ColName => match kw_type {
            KWType::ColumnTable => true,
            KWType::Type => true,
        },
        KeywordCategory::TypeFuncName => match kw_type {
            KWType::ColumnTable => false,
            KWType::Type => true,
        },
    }
}

fn parse_header() -> Result<HashMap<String, KeywordMeta>> {
    // use the environment variable set by the build script to locate the kwlist.h file
    let kwlist_file = path::PathBuf::from(env!("PG_QUERY_KWLIST_PATH"));
    let data = std::fs::read_to_string(kwlist_file).context("Failed to read kwlist.h")?;

    let mut keywords = HashMap::new();

    for line in data.lines() {
        if line.starts_with("PG_KEYWORD") {
            let line = line
                .split(&['(', ')'])
                .nth(1)
                .context("Invalid kwlist.h structure")?;

            let row_items: Vec<&str> = line.split(',').collect();

            match row_items[..] {
                [name, _value, category, is_bare_label] => {
                    let label = match is_bare_label.trim() {
                        "AS_LABEL" => KeywordLabel::As,
                        "BARE_LABEL" => KeywordLabel::Bare,
                        unexpected => anyhow::bail!("Unexpected label: {}", unexpected),
                    };

                    let category = match category.trim() {
                        "UNRESERVED_KEYWORD" => KeywordCategory::Unreserved,
                        "RESERVED_KEYWORD" => KeywordCategory::Reserved,
                        "COL_NAME_KEYWORD" => KeywordCategory::ColName,
                        "TYPE_FUNC_NAME_KEYWORD" => KeywordCategory::TypeFuncName,
                        unexpected => anyhow::bail!("Unexpected category: {}", unexpected),
                    };

                    let meta = KeywordMeta { category, label };
                    let name = name.trim().replace('\"', "");
                    keywords.insert(name, meta);
                }
                _ => anyhow::bail!("Problem reading kwlist.h row"),
            }
        }
    }

    Ok(keywords)
}

pub(crate) struct KeywordKinds {
    pub(crate) all_keywords: Vec<String>,
    pub(crate) bare_label_keywords: Vec<String>,
    pub(crate) unreserved_keywords: Vec<String>,
    pub(crate) reserved_keywords: Vec<String>,
    pub(crate) col_table_keywords: Vec<String>,
    pub(crate) type_keywords: Vec<String>,
}

pub(crate) fn keyword_kinds() -> Result<KeywordKinds> {
    let keywords = parse_header()?;
    let mut bare_label_keywords = keywords
        .iter()
        .filter(|(_key, value)| match value.label {
            KeywordLabel::As => false,
            KeywordLabel::Bare => true,
        })
        .map(|(key, _value)| key.to_owned())
        .collect::<Vec<String>>();
    bare_label_keywords.sort();

    let mut unreserved_keywords = keywords
        .iter()
        .filter(|(_key, value)| matches!(value.category, KeywordCategory::Unreserved))
        .map(|(key, _value)| key.to_owned())
        .collect::<Vec<String>>();
    unreserved_keywords.sort();

    let mut reserved_keywords = keywords
        .iter()
        .filter(|(_key, value)| matches!(value.category, KeywordCategory::Reserved))
        .map(|(key, _value)| key.to_owned())
        .collect::<Vec<String>>();
    reserved_keywords.sort();

    let mut all_keywords = keywords
        .keys()
        .map(|key| key.to_owned())
        .collect::<Vec<String>>();
    all_keywords.sort();

    let mut col_table_tokens = HashSet::new();
    let mut type_tokens = HashSet::new();
    for (key, meta) in &keywords {
        for variant in all::<KWType>() {
            match variant {
                KWType::ColumnTable => {
                    if keyword_allowed(meta.category, variant) {
                        col_table_tokens.insert(key);
                    }
                }
                KWType::Type => {
                    if keyword_allowed(meta.category, variant) {
                        type_tokens.insert(key);
                    }
                }
            }
        }
    }

    let mut col_table_keywords = col_table_tokens
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    col_table_keywords.sort();
    let mut type_keywords = type_tokens
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    type_keywords.sort();

    Ok(KeywordKinds {
        all_keywords,
        bare_label_keywords,
        unreserved_keywords,
        reserved_keywords,
        col_table_keywords,
        type_keywords,
    })
}
