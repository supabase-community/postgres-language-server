/// Puts the formatted body into `CREATE FUNCTION ... AS '<body>'` for a `LANGUAGE sql` function.
///
/// A reformatted body goes on lines of its own between the quotes. A body that formatting leaves
/// alone keeps the layout it was written with. Returns true when the body was replaced.
pub fn set_formatted_sql_fn_body(ast: &mut pgls_query::NodeEnum, formatted: &str) -> bool {
    let Some(body) = sql_fn_body_mut(ast) else {
        return false;
    };

    if body.trim() == formatted.trim() {
        return false;
    }

    *body = format!("\n{formatted}\n");
    true
}

fn sql_fn_body_mut(ast: &mut pgls_query::NodeEnum) -> Option<&mut String> {
    let pgls_query::NodeEnum::CreateFunctionStmt(create_fn) = ast else {
        return None;
    };

    let language = pgls_query_ext::utils::find_option_value(create_fn, "language");
    if language.as_deref() != Some("sql") {
        return None;
    }

    let def_elem = create_fn
        .options
        .iter_mut()
        .find_map(|option| match option.node.as_mut() {
            Some(pgls_query::NodeEnum::DefElem(def_elem)) if def_elem.defname == "as" => {
                Some(def_elem)
            }
            _ => None,
        })?;

    match def_elem.arg.as_mut()?.node.as_mut()? {
        pgls_query::NodeEnum::String(s) => Some(&mut s.sval),
        pgls_query::NodeEnum::List(list) => match list.items.as_mut_slice() {
            [item] => match item.node.as_mut()? {
                pgls_query::NodeEnum::String(s) => Some(&mut s.sval),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_sql_function_body() {
        let input = "CREATE FUNCTION add(a integer, b integer) RETURNS integer
    AS 'SELECT $1+$2;'
    LANGUAGE SQL
    IMMUTABLE;";

        let mut ast = pgls_query::parse(input).unwrap().into_root().unwrap();

        assert!(set_formatted_sql_fn_body(&mut ast, "select $1 + $2;"));

        let create_fn = match &ast {
            pgls_query::NodeEnum::CreateFunctionStmt(stmt) => stmt,
            _ => panic!("Expected CreateFunctionStmt"),
        };

        let body = pgls_query_ext::utils::find_option_value(create_fn, "as");
        assert_eq!(body, Some("\nselect $1 + $2;\n".to_string()));
    }

    #[test]
    fn an_unchanged_sql_function_body_keeps_its_layout() {
        let input = "CREATE FUNCTION one() RETURNS integer AS $$\n  select 1;\n$$ LANGUAGE SQL;";

        let mut ast = pgls_query::parse(input).unwrap().into_root().unwrap();

        assert!(!set_formatted_sql_fn_body(&mut ast, "select 1;"));
    }
}
