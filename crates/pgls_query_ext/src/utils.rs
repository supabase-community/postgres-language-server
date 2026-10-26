use pgls_text_size::{TextRange, TextSize};

/// Helper function to find a specific option value from function options
pub fn find_option_value(
    create_fn: &pgls_query::protobuf::CreateFunctionStmt,
    option_name: &str,
) -> Option<String> {
    create_fn
        .options
        .iter()
        .filter_map(|opt_wrapper| opt_wrapper.node.as_ref())
        .find_map(|opt| {
            if let pgls_query::NodeEnum::DefElem(def_elem) = opt {
                if def_elem.defname == option_name {
                    def_elem
                        .arg
                        .iter()
                        .filter_map(|arg_wrapper| arg_wrapper.node.as_ref())
                        .find_map(|arg| {
                            if let pgls_query::NodeEnum::String(s) = arg {
                                Some(s.sval.clone())
                            } else if let pgls_query::NodeEnum::List(l) = arg {
                                l.items.iter().find_map(|item_wrapper| {
                                    if let Some(pgls_query::NodeEnum::String(s)) =
                                        item_wrapper.node.as_ref()
                                    {
                                        Some(s.sval.clone())
                                    } else {
                                        None
                                    }
                                })
                            } else {
                                None
                            }
                        })
                } else {
                    None
                }
            } else {
                None
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
pub fn sql_function_body(stmt: &pgls_query::NodeEnum, sql: &str) -> Option<SqlFunctionBody> {
    let pgls_query::NodeEnum::CreateFunctionStmt(create_fn) = stmt else {
        return None;
    };
    if find_option_value(create_fn, "language")? != "sql" {
        return None;
    }
    let body = find_option_value(create_fn, "as")?;
    let start = sql.find(&body)?;
    let range = TextRange::at(
        TextSize::try_from(start).ok()?,
        TextSize::try_from(body.len()).ok()?,
    );
    Some(SqlFunctionBody { range, body })
}

pub fn parse_name(nodes: &[pgls_query::protobuf::Node]) -> Option<(Option<String>, String)> {
    let names = nodes
        .iter()
        .map(|n| match &n.node {
            Some(pgls_query::NodeEnum::String(s)) => Some(s.sval.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    match names.as_slice() {
        [Some(schema), Some(name)] => Some((Some(schema.clone()), name.clone())),
        [Some(name)] => Some((None, name.clone())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{find_option_value, parse_name, sql_function_body};

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

        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();
        let create_fn = match &ast {
            pgls_query::NodeEnum::CreateFunctionStmt(stmt) => stmt,
            _ => panic!("Expected CreateFunctionStmt"),
        };

        assert_eq!(
            find_option_value(create_fn, "language"),
            Some("plpgsql".to_string())
        );

        assert!(find_option_value(create_fn, "as").is_some(),);

        assert_eq!(
            parse_name(&create_fn.return_type.as_ref().unwrap().names),
            Some((Some("pg_catalog".to_string()), "bool".to_string()))
        );
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
