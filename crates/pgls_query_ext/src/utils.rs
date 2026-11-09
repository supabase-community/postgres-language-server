//! Helpers to read names and options out of the parse tree.
//!
//! Name lists like `funcname` or `TypeName::names` only ever hold `String` nodes. A list with
//! any other node is not a name, and the helpers return `None` for it.

use pgls_query::{
    NodeEnum,
    protobuf::{ColumnRef, CreateFunctionStmt, Node, RangeVar, TypeName},
};

/// The value of a `String` node.
pub fn string_value(node: &Node) -> Option<&str> {
    match node.node.as_ref()? {
        NodeEnum::String(value) => Some(&value.sval),
        _ => None,
    }
}

/// The values of a list of `String` nodes, e.g. `["public", "users"]`. `None` if a node is not
/// a string.
pub fn string_values(nodes: &[Node]) -> Option<Vec<String>> {
    nodes
        .iter()
        .map(|node| string_value(node).map(str::to_owned))
        .collect()
}

/// A possibly qualified name, like `name`, `schema.name` or `catalog.schema.name`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualifiedName<'a> {
    pub catalog: Option<&'a str>,
    pub schema: Option<&'a str>,
    pub name: &'a str,
}

/// Reads a possibly qualified name from a name list, like [`DeconstructQualifiedName`]. `None`
/// for an empty list, more than three parts, which Postgres rejects, and lists with nodes that
/// are not strings.
///
/// Postgres only accepts a catalog that is the current database, and then ignores it. We
/// don't know the current database, so callers use the schema and the name.
///
/// [`DeconstructQualifiedName`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L3304
pub fn qualified_name(nodes: &[Node]) -> Option<QualifiedName<'_>> {
    let names = nodes.iter().map(string_value).collect::<Option<Vec<_>>>()?;
    let (catalog, schema, name) = match names.as_slice() {
        [name] => (None, None, *name),
        [schema, name] => (None, Some(*schema), *name),
        [catalog, schema, name] => (Some(*catalog), Some(*schema), *name),
        _ => return None,
    };
    Some(QualifiedName {
        catalog,
        schema,
        name,
    })
}

/// The name of a `RangeVar`, like `t`, `s.t` or `db.s.t`.
pub fn range_var_name(range_var: &RangeVar) -> QualifiedName<'_> {
    fn part(value: &str) -> Option<&str> {
        (!value.is_empty()).then_some(value)
    }
    QualifiedName {
        catalog: part(&range_var.catalogname),
        schema: part(&range_var.schemaname),
        name: &range_var.relname,
    }
}

/// The name of a type as the parser stores it, e.g. `pg_catalog.int4` for `integer`. `None`
/// for names that are not a [`qualified_name`].
pub fn type_name(type_name: &TypeName) -> Option<QualifiedName<'_>> {
    qualified_name(&type_name.names)
}

/// Whether a select list entry is a `*` that expands to any number of columns, like `*`, `t.*`
/// or `(t.c).*`.
pub fn is_star(value: &NodeEnum) -> bool {
    let last = match value {
        NodeEnum::ColumnRef(column) => column.fields.last(),
        NodeEnum::AIndirection(indirection) => indirection.indirection.last(),
        _ => None,
    };
    matches!(
        last.and_then(|last| last.node.as_ref()),
        Some(NodeEnum::AStar(_))
    )
}

/// The fields before the `*` of a column reference like `*`, `t.*` or `s.t.*`. `None` if the
/// reference doesn't end in `*`.
pub fn star_qualifier(column: &ColumnRef) -> Option<&[Node]> {
    let (last, qualifier) = column.fields.split_last()?;
    matches!(last.node, Some(NodeEnum::AStar(_))).then_some(qualifier)
}

/// The string value of a `CREATE FUNCTION` option, like `language`, or `as` with the body.
///
/// The argument is a string or, for `as`, a list of strings. A list of two strings is the
/// object file and the link symbol of a C function ([`interpret_AS_clause`]), so only a list
/// of one string has a value.
///
/// [`interpret_AS_clause`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/functioncmds.c#L866
pub fn find_option_value<'a>(
    create_fn: &'a CreateFunctionStmt,
    option_name: &str,
) -> Option<&'a str> {
    create_fn.options.iter().find_map(|option| {
        let Some(NodeEnum::DefElem(option)) = &option.node else {
            return None;
        };
        if option.defname != option_name {
            return None;
        }
        match option.arg.as_deref()?.node.as_ref()? {
            NodeEnum::String(value) => Some(value.sval.as_str()),
            NodeEnum::List(list) => match list.items.as_slice() {
                [item] => string_value(item),
                _ => None,
            },
            _ => None,
        }
    })
}

/// [`find_option_value`], to change the value.
pub fn find_option_value_mut<'a>(
    create_fn: &'a mut CreateFunctionStmt,
    option_name: &str,
) -> Option<&'a mut String> {
    create_fn.options.iter_mut().find_map(|option| {
        let Some(NodeEnum::DefElem(option)) = &mut option.node else {
            return None;
        };
        if option.defname != option_name {
            return None;
        }
        match option.arg.as_deref_mut()?.node.as_mut()? {
            NodeEnum::String(value) => Some(&mut value.sval),
            NodeEnum::List(list) => match list.items.as_mut_slice() {
                [item] => match item.node.as_mut()? {
                    NodeEnum::String(value) => Some(&mut value.sval),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    })
}

#[cfg(test)]
mod tests {
    use pgls_query::NodeEnum;

    use crate::utils::{
        QualifiedName, find_option_value, is_star, qualified_name, range_var_name, star_qualifier,
        string_value,
    };

    fn create_function(sql: &str) -> pgls_query::protobuf::CreateFunctionStmt {
        match pgls_query::parse(sql).unwrap().into_root().unwrap() {
            NodeEnum::CreateFunctionStmt(stmt) => *stmt,
            _ => panic!("Expected CreateFunctionStmt"),
        }
    }

    #[test]
    fn test_find_option_value() {
        let input = "
            CREATE OR REPLACE FUNCTION public.f1()
            RETURNS boolean
            LANGUAGE plpgsql
            AS $function$
            declare r t1 := (select t1 from t1 where a = 1);
            BEGIN
              if r.c is null or
                 true is false
                then -- there is bug - table t1 missing \"c\" column
                RAISE NOTICE 'c is null';
              end if;
            END;
            $function$;
"
        .trim();

        let create_fn = create_function(input);

        assert_eq!(find_option_value(&create_fn, "language"), Some("plpgsql"));

        assert!(find_option_value(&create_fn, "as").is_some(),);

        assert_eq!(
            qualified_name(&create_fn.return_type.as_ref().unwrap().names),
            Some(QualifiedName {
                catalog: None,
                schema: Some("pg_catalog"),
                name: "bool",
            })
        );
    }

    #[test]
    fn the_object_file_of_a_c_function_is_not_its_body() {
        let create_fn =
            create_function("create function f() returns int language c as 'my_lib', 'my_symbol'");

        assert_eq!(find_option_value(&create_fn, "language"), Some("c"));
        assert_eq!(find_option_value(&create_fn, "as"), None);
    }

    #[test]
    fn qualified_names() {
        let funcname = |sql: &str| create_function(sql).funcname;

        assert_eq!(
            qualified_name(&funcname("create function f() returns int return 1")),
            Some(QualifiedName {
                catalog: None,
                schema: None,
                name: "f",
            })
        );
        assert_eq!(
            qualified_name(&funcname("create function db.s.f() returns int return 1")),
            Some(QualifiedName {
                catalog: Some("db"),
                schema: Some("s"),
                name: "f",
            })
        );
        // Postgres rejects more than three parts.
        assert_eq!(
            qualified_name(&funcname("create function a.db.s.f() returns int return 1")),
            None
        );
        assert_eq!(qualified_name(&[]), None);

        // `t.*` is not a name.
        let NodeEnum::SelectStmt(select) = pgls_query::parse("select t.* from t")
            .unwrap()
            .into_root()
            .unwrap()
        else {
            panic!("Expected SelectStmt");
        };
        let Some(NodeEnum::ResTarget(target)) = &select.target_list[0].node else {
            panic!("Expected ResTarget");
        };
        let Some(NodeEnum::ColumnRef(column)) = target.val.as_ref().unwrap().node.as_ref() else {
            panic!("Expected ColumnRef");
        };
        assert_eq!(qualified_name(&column.fields), None);
    }

    fn select_values(sql: &str) -> Vec<NodeEnum> {
        let NodeEnum::SelectStmt(select) = pgls_query::parse(sql).unwrap().into_root().unwrap()
        else {
            panic!("Expected SelectStmt");
        };
        select
            .target_list
            .iter()
            .map(|target| match target.node.as_ref().unwrap() {
                NodeEnum::ResTarget(target) => target.val.as_ref().unwrap().node.clone().unwrap(),
                _ => panic!("Expected ResTarget"),
            })
            .collect()
    }

    #[test]
    fn stars() {
        let values = select_values("select *, t.*, s.t.*, (t.c).*, t.c, (t.c).f from t");
        let stars: Vec<_> = values.iter().map(is_star).collect();
        assert_eq!(stars, [true, true, true, true, false, false]);

        let qualifiers: Vec<_> = values
            .iter()
            .map(|value| match value {
                NodeEnum::ColumnRef(column) => star_qualifier(column).map(|qualifier| {
                    qualifier
                        .iter()
                        .map(|field| string_value(field).unwrap())
                        .collect::<Vec<_>>()
                }),
                _ => None,
            })
            .collect();
        assert_eq!(
            qualifiers,
            [
                Some(vec![]),
                Some(vec!["t"]),
                Some(vec!["s", "t"]),
                None,
                None,
                None
            ]
        );
    }

    #[test]
    fn range_var_names() {
        let NodeEnum::SelectStmt(select) = pgls_query::parse("select from t, s.t, db.s.t")
            .unwrap()
            .into_root()
            .unwrap()
        else {
            panic!("Expected SelectStmt");
        };
        let names: Vec<_> = select
            .from_clause
            .iter()
            .map(|item| match item.node.as_ref().unwrap() {
                NodeEnum::RangeVar(range_var) => {
                    let name = range_var_name(range_var);
                    (name.catalog, name.schema, name.name)
                }
                _ => panic!("Expected RangeVar"),
            })
            .collect();
        assert_eq!(
            names,
            [
                (None, None, "t"),
                (None, Some("s"), "t"),
                (Some("db"), Some("s"), "t")
            ]
        );
    }
}
