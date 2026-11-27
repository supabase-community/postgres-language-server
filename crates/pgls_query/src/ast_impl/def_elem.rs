use std::borrow::Cow;

use crate::{
    NodeEnum,
    protobuf::{DefElem, Node, TypeName},
};

impl DefElem {
    /// The option named `name` in a list of options, the last one if there are several.
    ///
    /// Postgres compares option names with `strcmp`, like `strcmp(opt->defname, "full")` in
    /// [`ExecVacuum`]. The parser lowercases unquoted names, so `VACUUM (FULL)` has the option
    /// `full`, while `VACUUM ("FULL")` has `FULL`, which Postgres rejects as unknown. `VACUUM` and
    /// `REINDEX` accept an option more than once, and the last one wins; most other statements
    /// reject it with [`errorConflictingDefElem`].
    ///
    /// [`ExecVacuum`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/vacuum.c#L243
    /// [`errorConflictingDefElem`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/define.c#L371
    pub fn find<'a>(options: &'a [Node], name: &str) -> Option<&'a DefElem> {
        options
            .iter()
            .rev()
            .find_map(|option| match option.node.as_ref()? {
                NodeEnum::DefElem(def) if def.defname == name => Some(def.as_ref()),
                _ => None,
            })
    }

    /// [`Self::find`], to change the option.
    pub fn find_mut<'a>(options: &'a mut [Node], name: &str) -> Option<&'a mut DefElem> {
        options
            .iter_mut()
            .rev()
            .find_map(|option| match option.node.as_mut()? {
                NodeEnum::DefElem(def) if def.defname == name => Some(def.as_mut()),
                _ => None,
            })
    }

    /// The value of a boolean option, like [`defGetBoolean`]: on without a value, the integers `0`
    /// and `1`, and `true`, `false`, `on` and `off` in any case. `None` for any other value, which
    /// Postgres rejects, like `full 2` or `full '0'`.
    ///
    /// [`defGetBoolean`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/define.c#L94
    pub fn bool_value(&self) -> Option<bool> {
        let Some(arg) = self.arg.as_deref() else {
            return Some(true);
        };
        if let Some(NodeEnum::Integer(value)) = &arg.node {
            return match value.ival {
                0 => Some(false),
                1 => Some(true),
                _ => None,
            };
        }
        let value = self.string_value()?;
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

    /// The value of the option as a string, like [`defGetString`]. `None` without a value and for
    /// values that the grammar doesn't accept.
    ///
    /// [`defGetString`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/define.c#L35
    fn string_value(&self) -> Option<Cow<'_, str>> {
        Some(match self.arg.as_deref()?.node.as_ref()? {
            NodeEnum::Integer(value) => Cow::Owned(value.ival.to_string()),
            NodeEnum::Float(value) => Cow::Borrowed(value.fval.as_str()),
            NodeEnum::Boolean(value) => Cow::Borrowed(if value.boolval { "true" } else { "false" }),
            NodeEnum::String(value) => Cow::Borrowed(value.sval.as_str()),
            NodeEnum::TypeName(type_name) => Cow::Owned(type_name.to_name_string()?),
            NodeEnum::List(list) => Cow::Owned(Node::name_list_to_string(&list.items)?),
            NodeEnum::AStar(_) => Cow::Borrowed("*"),
            _ => return None,
        })
    }
}

impl TypeName {
    /// Like [`TypeNameToString`]. Unquoted keywords that aren't reserved, like `off` in
    /// `connect = off`, are parsed as type names. `None` for a type given by its oid, which the
    /// parser doesn't produce.
    ///
    /// [`TypeNameToString`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_type.c#L439
    fn to_name_string(&self) -> Option<String> {
        if self.names.is_empty() {
            return None;
        }
        let mut value = self
            .names
            .iter()
            .map(Node::string_value)
            .collect::<Option<Vec<_>>>()?
            .join(".");
        if self.pct_type {
            value.push_str("%TYPE");
        }
        if !self.array_bounds.is_empty() {
            value.push_str("[]");
        }
        Some(value)
    }
}

impl Node {
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
}

#[cfg(test)]
mod tests {
    use crate::{NodeEnum, protobuf::DefElem};

    /// The value of the subscription option `name`, `None` if it is missing or invalid.
    fn subscription_option(options: &str, name: &str) -> Option<Option<bool>> {
        let sql = format!("create subscription s connection 'c' publication p with ({options})");
        let NodeEnum::CreateSubscriptionStmt(stmt) =
            crate::parse(&sql).unwrap().into_root().unwrap()
        else {
            panic!("Expected CreateSubscriptionStmt");
        };
        DefElem::find(&stmt.options, name).map(DefElem::bool_value)
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
