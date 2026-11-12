//! Helpers to read the options of a statement, the `DefElem`s of e.g. `VACUUM (FULL)`,
//! `REINDEX (CONCURRENTLY)` or `CREATE SUBSCRIPTION ... WITH (connect = false)`.

use std::borrow::Cow;

use pgls_query::{
    NodeEnum,
    protobuf::{DefElem, Node, ReindexStmt, TypeName, VacuumStmt},
};

use crate::utils::string_value;

/// The option named `name`, the last one if there are several.
///
/// Postgres compares option names with `strcmp`, like `strcmp(opt->defname, "full")` in
/// [`ExecVacuum`]. The parser lowercases unquoted names, so `VACUUM (FULL)` has the option `full`,
/// while `VACUUM ("FULL")` has `FULL`, which Postgres rejects as unknown. `VACUUM` and `REINDEX`
/// accept an option more than once, and the last one wins; most other statements reject it with
/// [`errorConflictingDefElem`].
///
/// [`ExecVacuum`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/vacuum.c#L243
/// [`errorConflictingDefElem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/define.c#L371
pub fn find_def_elem<'a>(options: &'a [Node], name: &str) -> Option<&'a DefElem> {
    options
        .iter()
        .rev()
        .find_map(|option| match option.node.as_ref()? {
            NodeEnum::DefElem(def) if def.defname == name => Some(def.as_ref()),
            _ => None,
        })
}

/// The value of a boolean option, like [`defGetBoolean`]: on without a value, the integers `0`
/// and `1`, and `true`, `false`, `on` and `off` in any case. `None` for any other value, which
/// Postgres rejects, like `full 2` or `full '0'`.
///
/// [`defGetBoolean`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/define.c#L94
pub fn def_elem_bool(def: &DefElem) -> Option<bool> {
    let Some(arg) = def.arg.as_deref() else {
        return Some(true);
    };
    if let Some(NodeEnum::Integer(value)) = &arg.node {
        return match value.ival {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        };
    }
    let value = def_elem_string(arg)?;
    // The strings of the grammar's `opt_boolean_or_string`.
    [
        ("true", true),
        ("false", false),
        ("on", true),
        ("off", false),
    ]
    .into_iter()
    .find_map(|(word, enabled)| value.eq_ignore_ascii_case(word).then_some(enabled))
}

/// The value of an option as a string, like [`defGetString`]. `None` for nodes it doesn't
/// accept.
///
/// [`defGetString`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/define.c#L35
fn def_elem_string(arg: &Node) -> Option<Cow<'_, str>> {
    Some(match arg.node.as_ref()? {
        NodeEnum::Integer(value) => Cow::Owned(value.ival.to_string()),
        NodeEnum::Float(value) => Cow::Borrowed(value.fval.as_str()),
        NodeEnum::Boolean(value) => Cow::Borrowed(if value.boolval { "true" } else { "false" }),
        NodeEnum::String(value) => Cow::Borrowed(value.sval.as_str()),
        NodeEnum::TypeName(type_name) => Cow::Owned(type_name_to_string(type_name)?),
        NodeEnum::List(list) => Cow::Owned(name_list_to_string(&list.items)?),
        NodeEnum::AStar(_) => Cow::Borrowed("*"),
        _ => return None,
    })
}

/// Like [`TypeNameToString`]. Unquoted keywords that aren't reserved, like `off` in
/// `connect = off`, are parsed as type names. `None` for a type given by its oid, which the
/// parser doesn't produce.
///
/// [`TypeNameToString`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_type.c#L439
fn type_name_to_string(type_name: &TypeName) -> Option<String> {
    if type_name.names.is_empty() {
        return None;
    }
    let mut value = type_name
        .names
        .iter()
        .map(string_value)
        .collect::<Option<Vec<_>>>()?
        .join(".");
    if type_name.pct_type {
        value.push_str("%TYPE");
    }
    if !type_name.array_bounds.is_empty() {
        value.push_str("[]");
    }
    Some(value)
}

/// Like [`NameListToString`].
///
/// [`NameListToString`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L3597
fn name_list_to_string(names: &[Node]) -> Option<String> {
    Some(
        names
            .iter()
            .map(|name| match name.node.as_ref()? {
                NodeEnum::String(value) => Some(value.sval.as_str()),
                NodeEnum::AStar(_) => Some("*"),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?
            .join("."),
    )
}

/// Whether the statement is `VACUUM FULL`, like the `full` option of [`ExecVacuum`]. Not for an
/// invalid value, which Postgres rejects.
///
/// [`ExecVacuum`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/vacuum.c#L243
pub fn is_vacuum_full(stmt: &VacuumStmt) -> bool {
    find_def_elem(&stmt.options, "full").and_then(def_elem_bool) == Some(true)
}

/// Whether the statement is `REINDEX CONCURRENTLY`, like the `concurrently` option of
/// [`ExecReindex`]. Not for an invalid value, which Postgres rejects.
///
/// [`ExecReindex`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/indexcmds.c#L2857
pub fn is_reindex_concurrent(stmt: &ReindexStmt) -> bool {
    find_def_elem(&stmt.params, "concurrently").and_then(def_elem_bool) == Some(true)
}

#[cfg(test)]
mod tests {
    use pgls_query::NodeEnum;

    use super::{def_elem_bool, find_def_elem, is_reindex_concurrent, is_vacuum_full};

    fn parse(sql: &str) -> NodeEnum {
        pgls_query::parse(sql).unwrap().into_root().unwrap()
    }

    fn vacuum_full(sql: &str) -> bool {
        match parse(sql) {
            NodeEnum::VacuumStmt(stmt) => is_vacuum_full(&stmt),
            _ => panic!("Expected VacuumStmt"),
        }
    }

    fn reindex_concurrent(sql: &str) -> bool {
        match parse(sql) {
            NodeEnum::ReindexStmt(stmt) => is_reindex_concurrent(&stmt),
            _ => panic!("Expected ReindexStmt"),
        }
    }

    /// The value of the subscription option `name`, `None` if it is missing or invalid.
    fn subscription_option(options: &str, name: &str) -> Option<Option<bool>> {
        let sql = format!("create subscription s connection 'c' publication p with ({options})");
        let NodeEnum::CreateSubscriptionStmt(stmt) = parse(&sql) else {
            panic!("Expected CreateSubscriptionStmt");
        };
        find_def_elem(&stmt.options, name).map(def_elem_bool)
    }

    #[test]
    fn vacuum_full_values() {
        assert!(vacuum_full("vacuum full t"));
        assert!(vacuum_full("vacuum (full) t"));
        assert!(vacuum_full("vacuum (FULL true) t"));
        assert!(vacuum_full("vacuum (full 1) t"));
        assert!(vacuum_full("vacuum (full 'ON') t"));
        assert!(!vacuum_full("vacuum t"));
        assert!(!vacuum_full("vacuum (full off) t"));
        assert!(!vacuum_full("vacuum (full 0) t"));
        // Postgres rejects these.
        assert!(!vacuum_full("vacuum (full 2) t"));
        assert!(!vacuum_full("vacuum (full '0') t"));
        assert!(!vacuum_full("vacuum (full 1.0) t"));
        assert!(!vacuum_full("vacuum (\"FULL\") t"));
        // The last one wins.
        assert!(!vacuum_full("vacuum (full, full false) t"));
        assert!(vacuum_full("vacuum (full false, full) t"));
    }

    #[test]
    fn reindex_concurrent_values() {
        assert!(reindex_concurrent("reindex index concurrently i"));
        assert!(reindex_concurrent("reindex (concurrently) index i"));
        assert!(!reindex_concurrent("reindex index i"));
        assert!(!reindex_concurrent("reindex (concurrently false) index i"));
        // `CONCURRENTLY` after the object kind comes after the options in the list.
        assert!(reindex_concurrent(
            "reindex (concurrently false) index concurrently i"
        ));
    }

    #[test]
    fn unreserved_keywords_are_type_names() {
        assert_eq!(
            subscription_option("connect = off", "connect"),
            Some(Some(false))
        );
        assert_eq!(
            subscription_option("connect = on", "connect"),
            Some(Some(true))
        );
        assert_eq!(
            subscription_option("connect = false", "connect"),
            Some(Some(false))
        );
        assert_eq!(
            subscription_option("connect = 'yes'", "connect"),
            Some(None)
        );
        assert_eq!(
            subscription_option("connect = public.off", "connect"),
            Some(None)
        );
        assert_eq!(subscription_option("create_slot", "connect"), None);
    }
}
