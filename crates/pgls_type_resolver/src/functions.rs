use pgls_schema_cache::{Function, SchemaCache};

use crate::{
    types::{PossibleType, resolve_type},
    util::get_string_from_node,
};

pub fn resolve_func_call<'b>(
    node: &pgls_query::protobuf::FuncCall,
    schema_cache: &'b SchemaCache,
) -> Option<&'b Function> {
    let (schema, name) = resolve_func_identifier(node);

    let fns = schema_cache
        .functions
        .iter()
        .filter(|f| {
            function_matches(
                f,
                schema.as_deref(),
                name.as_str(),
                node.args
                    .iter()
                    .map(|a| resolve_type(a.node.as_ref().unwrap(), schema_cache))
                    .collect(),
            )
        })
        .collect::<Vec<&Function>>();

    if fns.len() == 1 { Some(fns[0]) } else { None }
}

fn resolve_func_identifier(node: &pgls_query::protobuf::FuncCall) -> (Option<String>, String) {
    match node.funcname.as_slice() {
        [name] => (None, get_string_from_node(name)),
        [schema, name] => (
            Some(get_string_from_node(schema)),
            get_string_from_node(name),
        ),
        _ => panic!("Function name has more than 2 parts"),
    }
}

fn function_matches(
    func: &Function,
    schema: Option<&str>,
    name: &str,
    arg_types: Vec<PossibleType>,
) -> bool {
    if func.name != name {
        return false;
    }

    if schema.is_some() && Some(func.schema.as_str()) != schema {
        return false;
    }

    let arg_count = arg_types.len();
    let args_with_default = func
        .args
        .args
        .iter()
        .filter(|a| a.has_default.is_some())
        .count();
    let total_args = func.args.args.len();

    if total_args < arg_count || total_args - args_with_default > arg_count {
        return false;
    }

    for (i, (func_arg, possible_type)) in func.args.args.iter().zip(arg_types.iter()).enumerate() {
        match possible_type {
            PossibleType::Null => {
                // can be any type
            }
            PossibleType::AnyOf(types) => {
                if types.iter().all(|type_id| *type_id != func_arg.type_id) {
                    return false;
                }
            }
        }

        if i >= arg_count && !func_arg.has_default.unwrap_or(false) {
            return false;
        }
    }
    true
}
