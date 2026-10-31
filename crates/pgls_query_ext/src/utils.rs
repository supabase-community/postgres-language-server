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
    use crate::utils::{find_option_value, parse_name};

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
}
