use pgt_text_size::TextRange;

#[derive(Debug, Clone)]
pub struct SQLFunctionArgs {
    pub name: Option<String>,
    pub type_: (Option<String>, String),
}

#[derive(Debug, Clone)]
pub struct SQLFunctionSignature {
    pub name: (Option<String>, String),
    pub args: Vec<SQLFunctionArgs>,
}

#[derive(Debug, Clone)]
pub struct SQLFunctionBody {
    pub range: TextRange,
    pub body: String,
}

/// Extracts the function signature from a SQL function definition
pub fn get_sql_fn_signature(ast: &pgt_query_ext::NodeEnum) -> Option<SQLFunctionSignature> {
    let create_fn = match ast {
        pgt_query_ext::NodeEnum::CreateFunctionStmt(cf) => cf,
        _ => return None,
    };

    // Extract language from function options
    let language = find_option_value(create_fn, "language")?;

    // Only process SQL functions
    if language != "sql" {
        return None;
    }

    let fn_name = parse_name(&create_fn.funcname)?;

    // we return None if anything is not expected
    let mut fn_args = Vec::new();
    for arg in &create_fn.parameters {
        if let Some(pgt_query_ext::NodeEnum::FunctionParameter(node)) = &arg.node {
            let arg_name = (!node.name.is_empty()).then_some(node.name.clone());

            let type_name = parse_name(&node.arg_type.as_ref().unwrap().names)?;

            fn_args.push(SQLFunctionArgs {
                name: arg_name,
                type_: type_name,
            });
        } else {
            return None;
        }
    }

    Some(SQLFunctionSignature {
        name: fn_name,
        args: fn_args,
    })
}

/// Extracts the SQL body from a function definition
pub fn get_sql_fn_body(ast: &pgt_query_ext::NodeEnum, content: &str) -> Option<SQLFunctionBody> {
    let create_fn = match ast {
        pgt_query_ext::NodeEnum::CreateFunctionStmt(cf) => cf,
        _ => return None,
    };

    // Extract language from function options
    let language = find_option_value(create_fn, "language")?;

    // Only process SQL functions
    if language != "sql" {
        return None;
    }

    // Extract SQL body from function options
    let sql_body = find_option_value(create_fn, "as")?;

    // Find the range of the SQL body in the content
    let start = content.find(&sql_body)?;
    let end = start + sql_body.len();

    let range = TextRange::new(start.try_into().unwrap(), end.try_into().unwrap());

    Some(SQLFunctionBody {
        range,
        body: sql_body.clone(),
    })
}

/// Helper function to find a specific option value from function options
fn find_option_value(
    create_fn: &pgt_query_ext::protobuf::CreateFunctionStmt,
    option_name: &str,
) -> Option<String> {
    create_fn
        .options
        .iter()
        .filter_map(|opt_wrapper| opt_wrapper.node.as_ref())
        .find_map(|opt| {
            if let pgt_query_ext::NodeEnum::DefElem(def_elem) = opt {
                if def_elem.defname == option_name {
                    def_elem
                        .arg
                        .iter()
                        .filter_map(|arg_wrapper| arg_wrapper.node.as_ref())
                        .find_map(|arg| {
                            if let pgt_query_ext::NodeEnum::String(s) = arg {
                                Some(s.sval.clone())
                            } else if let pgt_query_ext::NodeEnum::List(l) = arg {
                                l.items.iter().find_map(|item_wrapper| {
                                    if let Some(pgt_query_ext::NodeEnum::String(s)) =
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

fn parse_name(nodes: &Vec<pgt_query_ext::protobuf::Node>) -> Option<(Option<String>, String)> {
    let names = nodes
        .iter()
        .map(|n| match &n.node {
            Some(pgt_query_ext::NodeEnum::String(s)) => Some(s.sval.clone()),
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
    use super::*;

    use pgt_fs::PgTPath;

    #[test]
    fn sql_function_signature() {
        let input = "CREATE FUNCTION add(test0 integer, test1 integer) RETURNS integer
    AS 'select $1 + $2;'
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT;";

        let ast = pgt_query_ext::parse(input).unwrap();

        let sig = get_sql_fn_signature(&ast);

        println!("Function signature: {:?}", sig);
    }
}
