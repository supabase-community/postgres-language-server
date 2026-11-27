//! Helpers to read names, options and `ALTER TABLE` subcommands out of the parse tree.
//!
//! Name lists like `funcname` or `TypeName::names` only ever hold `String` nodes. A list with
//! any other node is not a name, and the helpers return `None` for it.

use std::fmt;

use pgls_query::{
    NodeEnum,
    protobuf::{
        AlterTableCmd, AlterTableStmt, ColumnRef, CreateFunctionStmt, Node, RangeVar, TypeName,
    },
};

use crate::options::find_def_elem;

/// The most parts a name has: `catalog.schema.table.column`.
const NAME_PARTS_MAX: usize = 4;

/// The parts of a name list, like `["public", "users"]`, borrowed from the parse tree. Unlike
/// [`Node::string_values`], reading them doesn't allocate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameParts<'a> {
    parts: [&'a str; NAME_PARTS_MAX],
    len: usize,
}

impl<'a> NameParts<'a> {
    /// The parts of a list of `String` nodes. `None` if a node is not a string, and for lists
    /// with more parts than a name can have, which Postgres rejects.
    pub fn of(nodes: &'a [Node]) -> Option<Self> {
        if nodes.len() > NAME_PARTS_MAX {
            return None;
        }
        let mut parts = [""; NAME_PARTS_MAX];
        for (part, node) in parts.iter_mut().zip(nodes) {
            *part = node.string_value()?;
        }
        Some(Self {
            parts,
            len: nodes.len(),
        })
    }

    /// The last part of a list of `String` nodes of any length, like the operator of
    /// `OPERATOR(pg_catalog.=)`. `None` if the list is empty or a node is not a string.
    pub fn last(nodes: &'a [Node]) -> Option<&'a str> {
        nodes
            .iter()
            .try_fold(None, |_, node| node.string_value().map(Some))?
    }

    pub fn as_slice(&self) -> &[&'a str] {
        self.parts.get(..self.len).unwrap_or_default()
    }
}

/// A possibly qualified name, like `name`, `schema.name` or `catalog.schema.name`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualifiedName<'a> {
    pub catalog: Option<&'a str>,
    pub schema: Option<&'a str>,
    pub name: &'a str,
}

impl<'a> QualifiedName<'a> {
    /// Reads a possibly qualified name from a name list, like [`DeconstructQualifiedName`].
    /// `None` for an empty list, more than three parts, which Postgres rejects, and lists with
    /// nodes that are not strings.
    ///
    /// Postgres only accepts a catalog that is the current database, and then ignores it. We
    /// don't know the current database, so callers use the schema and the name.
    ///
    /// [`DeconstructQualifiedName`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L3304
    pub fn of(nodes: &'a [Node]) -> Option<Self> {
        let names = NameParts::of(nodes)?;
        let (catalog, schema, name) = match names.as_slice() {
            [name] => (None, None, *name),
            [schema, name] => (None, Some(*schema), *name),
            [catalog, schema, name] => (Some(*catalog), Some(*schema), *name),
            _ => return None,
        };
        Some(Self {
            catalog,
            schema,
            name,
        })
    }

    /// The name of a `RangeVar`, like `t`, `s.t` or `db.s.t`.
    pub fn of_range_var(range_var: &'a RangeVar) -> Self {
        let part = |value: &'a str| (!value.is_empty()).then_some(value);
        Self {
            catalog: part(&range_var.catalogname),
            schema: part(&range_var.schemaname),
            name: &range_var.relname,
        }
    }

    /// The name of a type as the parser stores it, e.g. `pg_catalog.int4` for `integer`. `None`
    /// for names that [`Self::of`] doesn't read.
    pub fn of_type_name(type_name: &'a TypeName) -> Option<Self> {
        Self::of(&type_name.names)
    }
}

/// The name with its qualifiers, like `myschema.mood`, unquoted like in Postgres' messages.
impl fmt::Display for QualifiedName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for qualifier in [self.catalog, self.schema].into_iter().flatten() {
            write!(f, "{qualifier}.")?;
        }
        f.write_str(self.name)
    }
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

/// The subcommands of an `ALTER TABLE` statement, e.g. `ADD COLUMN` and `DROP CONSTRAINT`.
pub fn alter_table_cmds(stmt: &AlterTableStmt) -> impl Iterator<Item = &AlterTableCmd> {
    stmt.cmds.iter().filter_map(|cmd| match cmd.node.as_ref()? {
        NodeEnum::AlterTableCmd(cmd) => Some(cmd.as_ref()),
        _ => None,
    })
}

/// The definition of an `ALTER TABLE` subcommand, e.g. the column of `ADD COLUMN` or the
/// constraint of `ADD CONSTRAINT`.
pub fn cmd_def(cmd: &AlterTableCmd) -> Option<&NodeEnum> {
    cmd.def.as_ref()?.node.as_ref()
}

/// Whether the subcommand is `DETACH PARTITION ... CONCURRENTLY`, the only subcommand that
/// Postgres can run concurrently.
pub fn detaches_concurrently(cmd: &AlterTableCmd) -> bool {
    matches!(cmd_def(cmd), Some(NodeEnum::PartitionCmd(partition)) if partition.concurrent)
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
    let option = find_def_elem(&create_fn.options, option_name)?;
    match option.arg.as_deref()?.node.as_ref()? {
        NodeEnum::String(value) => Some(value.sval.as_str()),
        NodeEnum::List(list) => match list.items.as_slice() {
            [item] => item.string_value(),
            _ => None,
        },
        _ => None,
    }
}

/// [`find_option_value`], to change the value. Like [`find_def_elem`], it reads the last option
/// with the name; Postgres rejects a repeated option of `CREATE FUNCTION`.
pub fn find_option_value_mut<'a>(
    create_fn: &'a mut CreateFunctionStmt,
    option_name: &str,
) -> Option<&'a mut String> {
    let option =
        create_fn
            .options
            .iter_mut()
            .rev()
            .find_map(|option| match option.node.as_mut()? {
                NodeEnum::DefElem(option) if option.defname == option_name => Some(option),
                _ => None,
            })?;
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
}

#[cfg(test)]
mod tests {
    use pgls_query::NodeEnum;

    use crate::utils::{NameParts, QualifiedName, find_option_value, is_star, star_qualifier};

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
            QualifiedName::of(&create_fn.return_type.as_ref().unwrap().names),
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
    fn name_parts() {
        let funcname = |sql: &str| create_function(sql).funcname;

        let names = funcname("create function db.s.f() returns int return 1");
        assert_eq!(
            NameParts::of(&names).map(|parts| parts.as_slice().to_vec()),
            Some(vec!["db", "s", "f"])
        );
        assert_eq!(NameParts::last(&names), Some("f"));
        assert_eq!(NameParts::last(&[]), None);
    }

    #[test]
    fn qualified_names() {
        let funcname = |sql: &str| create_function(sql).funcname;

        assert_eq!(
            QualifiedName::of(&funcname("create function f() returns int return 1")),
            Some(QualifiedName {
                catalog: None,
                schema: None,
                name: "f",
            })
        );
        assert_eq!(
            QualifiedName::of(&funcname("create function db.s.f() returns int return 1")),
            Some(QualifiedName {
                catalog: Some("db"),
                schema: Some("s"),
                name: "f",
            })
        );
        // Postgres rejects more than three parts.
        assert_eq!(
            QualifiedName::of(&funcname("create function a.db.s.f() returns int return 1")),
            None
        );
        assert_eq!(QualifiedName::of(&[]), None);

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
        assert_eq!(QualifiedName::of(&column.fields), None);
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
                        .map(|field| field.string_value().unwrap())
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
                    let name = QualifiedName::of_range_var(range_var);
                    (name.catalog, name.schema, name.name, name.to_string())
                }
                _ => panic!("Expected RangeVar"),
            })
            .collect();
        assert_eq!(
            names,
            [
                (None, None, "t", "t".to_string()),
                (None, Some("s"), "t", "s.t".to_string()),
                (Some("db"), Some("s"), "t", "db.s.t".to_string())
            ]
        );
    }
}
