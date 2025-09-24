use pgls_text_size::TextRange;

#[derive(Debug, Clone)]
pub struct ArgType {
    pub schema: Option<String>,
    pub name: String,
    pub is_array: bool,
}

#[derive(Debug, Clone)]
pub struct SQLFunctionArg {
    pub name: Option<String>,
    pub type_: ArgType,
}

#[derive(Debug, Clone)]
pub struct SQLFunctionSignature {
    #[allow(dead_code)]
    pub schema: Option<String>,
    pub name: String,
    pub args: Vec<SQLFunctionArg>,
}

#[derive(Debug, Clone)]
pub struct SQLFunctionBody {
    pub range: TextRange,
    pub body: String,
}

/// Extracts the function signature from a SQL function definition
pub fn get_sql_fn_signature(ast: &pgls_query::NodeEnum) -> Option<SQLFunctionSignature> {
    let create_fn = match ast {
        pgls_query::NodeEnum::CreateFunctionStmt(cf) => cf,
        _ => return None,
    };

    // Extract language from function options
    let language = pgls_query_ext::utils::find_option_value(create_fn, "language")?;

    // Only process SQL functions
    if language != "sql" {
        return None;
    }

    let fn_name = pgls_query_ext::utils::parse_name(&create_fn.funcname)?;

    // we return None if anything is not expected
    let mut fn_args = Vec::new();
    for arg in &create_fn.parameters {
        if let Some(pgls_query::NodeEnum::FunctionParameter(node)) = &arg.node {
            let arg_name = (!node.name.is_empty()).then_some(node.name.clone());

            let arg_type = node.arg_type.as_ref()?;
            let type_name = pgls_query_ext::utils::parse_name(&arg_type.names)?;
            fn_args.push(SQLFunctionArg {
                name: arg_name,
                type_: ArgType {
                    schema: type_name.0,
                    name: type_name.1,
                    is_array: node
                        .arg_type
                        .as_ref()
                        .map(|t| !t.array_bounds.is_empty())
                        .unwrap_or(false),
                },
            });
        } else {
            return None;
        }
    }

    Some(SQLFunctionSignature {
        schema: fn_name.0,
        name: fn_name.1,
        args: fn_args,
    })
}

/// Extracts the SQL body from a function definition
pub fn get_sql_fn_body(ast: &pgls_query::NodeEnum, content: &str) -> Option<SQLFunctionBody> {
    let create_fn = match ast {
        pgls_query::NodeEnum::CreateFunctionStmt(cf) => cf,
        _ => return None,
    };

    // Extract language from function options
    let language = pgls_query_ext::utils::find_option_value(create_fn, "language")?;

    // Only process SQL functions
    if language != "sql" {
        return None;
    }

    // Extract SQL body from function options
    let sql_body = pgls_query_ext::utils::find_option_value(create_fn, "as")?;

    // Find the range of the SQL body in the content
    let start = content.find(&sql_body)?;
    let end = start + sql_body.len();

    let range = TextRange::new(start.try_into().unwrap(), end.try_into().unwrap());

    Some(SQLFunctionBody {
        range,
        body: sql_body.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_function_signature() {
        let input = "CREATE FUNCTION add(test0 integer, test1 integer) RETURNS integer
    AS 'select $1 + $2;'
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT;";

        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();

        let sig = get_sql_fn_signature(&ast);

        assert!(sig.is_some());

        let sig = sig.unwrap();

        let arg1 = sig.args.first().unwrap();

        assert_eq!(arg1.name, Some("test0".to_string()));
        assert_eq!(arg1.type_.name, "int4");

        let arg2 = sig.args.get(1).unwrap();
        assert_eq!(arg2.name, Some("test1".to_string()));
        assert_eq!(arg2.type_.name, "int4");
    }

    #[test]
    fn array_type() {
        let input = "CREATE FUNCTION add(test0 integer[], test1 integer) RETURNS integer
    AS 'select $1 + $2;'
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT;";

        let ast = pgls_query::parse(input).unwrap().into_root().unwrap();

        let sig = get_sql_fn_signature(&ast);

        assert!(sig.is_some());

        let sig = sig.unwrap();

        assert!(
            sig.args
                .iter()
                .find(|arg| arg.type_.is_array)
                .map(|arg| {
                    assert_eq!(arg.type_.name, "int4");
                    assert!(arg.type_.is_array);
                })
                .is_some()
        );
    }
}
