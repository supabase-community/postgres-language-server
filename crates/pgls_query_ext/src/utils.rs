//! Helpers to read names and options out of the parse tree.
//!
//! Name lists like `funcname` or `TypeName::names` only ever hold `String` nodes. A list with
//! any other node is not a name, and the helpers return `None` for it.

use pgls_query::{
    NodeEnum,
    protobuf::{CreateFunctionStmt, Node},
};
use pgls_text_size::{TextRange, TextSize};

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

/// The body of a `LANGUAGE sql` function, written as a string (`AS '...'` or `AS $$...$$`).
#[derive(Debug, Clone)]
pub struct SqlFunctionBody {
    /// Where the body is in the statement.
    pub range: TextRange,
    pub body: String,
}

impl SqlFunctionBody {
    /// The ranges of the statements of the body, relative to the body, split like Postgres
    /// splits it ([`pg_parse_query`]). A body with a single statement, or one that doesn't parse,
    /// is a single range over the whole body, so its syntax error is reported for the body.
    ///
    /// [`pg_parse_query`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/tcop/postgres.c#L604
    pub fn statements(&self) -> Vec<TextRange> {
        let whole = || vec![TextRange::up_to(TextSize::of(self.body.as_str()))];
        let Ok(parsed) = pgls_query::parse(&self.body) else {
            return whole();
        };
        if parsed.protobuf.stmts.len() == 1 {
            return whole();
        }
        parsed
            .protobuf
            .stmts
            .iter()
            .filter_map(|stmt| {
                let start = usize::try_from(stmt.stmt_location).ok()?;
                // A length of 0 means the rest of the body.
                let end = match usize::try_from(stmt.stmt_len).ok()? {
                    0 => self.body.len(),
                    len => start + len,
                };
                let text = self.body.get(start..end)?;
                let start = start + (text.len() - text.trim_start().len());
                let mut end = start + text.trim().len();
                // Like top-level statements, a statement ends with its semicolon.
                if self.body[end..].starts_with(';') {
                    end += 1;
                }
                Some(TextRange::new(
                    TextSize::try_from(start).ok()?,
                    TextSize::try_from(end).ok()?,
                ))
            })
            .collect()
    }
}

/// The body of `stmt` if it creates a `LANGUAGE sql` function with a string body. `None` for
/// `BEGIN ATOMIC` and `RETURN` bodies, and for bodies whose text differs from the source, like
/// `'...'` with doubled quotes.
pub fn sql_function_body(stmt: &NodeEnum, sql: &str) -> Option<SqlFunctionBody> {
    let NodeEnum::CreateFunctionStmt(create_fn) = stmt else {
        return None;
    };
    if find_option_value(create_fn, "language")? != "sql" {
        return None;
    }
    let body = find_option_value(create_fn, "as")?.to_owned();
    let start = sql.find(&body)?;
    let range = TextRange::at(
        TextSize::try_from(start).ok()?,
        TextSize::try_from(body.len()).ok()?,
    );
    Some(SqlFunctionBody { range, body })
}

#[cfg(test)]
mod tests {
    use pgls_query::NodeEnum;

    use crate::utils::{QualifiedName, find_option_value, qualified_name, sql_function_body};

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

    fn body_statements(sql: &str) -> Vec<String> {
        let ast = pgls_query::parse(sql).unwrap().into_root().unwrap();
        let body = sql_function_body(&ast, sql).unwrap();
        body.statements()
            .into_iter()
            .map(|range| body.body[range].to_string())
            .collect()
    }

    #[test]
    fn sql_function_body_statements() {
        // A single statement is the whole body.
        assert_eq!(
            body_statements("create function f() returns int language sql as $$\n  select 1;\n$$"),
            ["\n  select 1;\n"]
        );
        assert_eq!(
            body_statements(
                "create function f(a int) returns int language sql as $$\n  insert into t values (a);\n  select 1\n$$"
            ),
            ["insert into t values (a);", "select 1"]
        );
        assert_eq!(
            body_statements(
                "create function f() returns int language sql as 'select 1; select 2;'"
            ),
            ["select 1;", "select 2;"]
        );
        // An empty body has no statements.
        assert!(
            body_statements("create function f() returns void language sql as $$ -- nothing $$")
                .is_empty()
        );
        // A body that doesn't parse is reported as a whole.
        assert_eq!(
            body_statements("create function f() returns void language sql as 'select 1; selec 2'"),
            ["select 1; selec 2"]
        );
    }
}
