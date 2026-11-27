/// A keyword of Postgres' grammar, from [`kwlist.h`].
///
/// [`kwlist.h`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/parser/kwlist.h
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Keyword {
    /// The keyword in lowercase, e.g. `select`.
    pub name: &'static str,
    pub category: KeywordCategory,
}

include!(concat!(env!("OUT_DIR"), "/keywords.rs"));

/// How reserved a keyword is, i.e. where Postgres accepts it as a name.
///
/// See the categories in https://github.com/postgres/postgres/blob/REL_18_6/src/include/parser/kwlist.h
/// and the name classification hierarchy in https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L17619
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum KeywordCategory {
    /// Available for use as any kind of name.
    Unreserved,
    /// Can be a column or table name, but not a function or type name.
    ColName,
    /// Can be a function or type name, but not a column or table name.
    TypeFuncName,
    /// Can only be a column label after `AS`.
    Reserved,
}

impl KeywordCategory {
    /// The category of the keyword `name`, `None` if it's no keyword. `name` must be lowercase,
    /// the way the parser stores an unquoted name: `Select` is no keyword, but `"Select"` quoted.
    pub fn of(name: &str) -> Option<KeywordCategory> {
        let index = Keyword::ALL
            .binary_search_by(|keyword| keyword.name.cmp(name))
            .ok()?;
        Keyword::ALL.get(index).map(|keyword| keyword.category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_are_found_by_their_lowercase_name() {
        assert_eq!(Keyword::ALL.first().map(|k| k.name), Some("abort"));
        assert_eq!(
            KeywordCategory::of("abort"),
            Some(KeywordCategory::Unreserved)
        );
        assert_eq!(
            KeywordCategory::of("between"),
            Some(KeywordCategory::ColName)
        );
        assert_eq!(
            KeywordCategory::of("left"),
            Some(KeywordCategory::TypeFuncName)
        );
        assert_eq!(
            KeywordCategory::of("select"),
            Some(KeywordCategory::Reserved)
        );
        assert_eq!(
            KeywordCategory::of("zone"),
            Some(KeywordCategory::Unreserved)
        );
        assert_eq!(KeywordCategory::of("Select"), None);
        assert_eq!(KeywordCategory::of("users"), None);
        assert_eq!(KeywordCategory::of(""), None);
    }
}
