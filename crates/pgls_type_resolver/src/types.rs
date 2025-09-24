use pgls_schema_cache::SchemaCache;

pub(crate) enum PossibleType {
    Null,
    AnyOf(Vec<i64>),
}

pub fn resolve_type(node: &pgls_query::NodeEnum, schema_cache: &SchemaCache) -> PossibleType {
    match node {
        pgls_query::NodeEnum::AConst(n) => {
            if n.isnull {
                PossibleType::Null
            } else {
                match n
                    .val
                    .as_ref()
                    .expect("expected non-nullable AConst to have a value")
                {
                    pgls_query::protobuf::a_const::Val::Ival(_) => {
                        let types: Vec<String> = ["int2", "int4", "int8"]
                            .iter()
                            .map(|s| s.to_string())
                            .collect();

                        PossibleType::AnyOf(
                            schema_cache
                                .types
                                .iter()
                                .filter(|t| {
                                    types.iter().any(|i| i == &t.name) && t.schema == "pg_catalog"
                                })
                                .map(|t| t.id)
                                .collect(),
                        )
                    }
                    pgls_query::protobuf::a_const::Val::Fval(_) => {
                        let types: Vec<String> =
                            ["float4", "float8"].iter().map(|s| s.to_string()).collect();

                        PossibleType::AnyOf(
                            schema_cache
                                .types
                                .iter()
                                .filter(|t| types.contains(&t.name) && t.schema == "pg_catalog")
                                .map(|t| t.id)
                                .collect(),
                        )
                    }
                    pgls_query::protobuf::a_const::Val::Boolval(_) => PossibleType::AnyOf(
                        schema_cache
                            .types
                            .iter()
                            .filter(|t| t.name == "bool" && t.schema == "pg_catalog")
                            .map(|t| t.id)
                            .collect(),
                    ),
                    pgls_query::protobuf::a_const::Val::Sval(v) => {
                        let types: Vec<String> =
                            ["text", "varchar"].iter().map(|s| s.to_string()).collect();

                        PossibleType::AnyOf(
                            schema_cache
                                .types
                                .iter()
                                .filter(|t| {
                                    (types.iter().any(|i| i == &t.name) && t.schema == "pg_catalog")
                                        || t.enums.values.contains(&v.sval)
                                })
                                .map(|t| t.id)
                                .collect(),
                        )
                    }
                    pgls_query::protobuf::a_const::Val::Bsval(_) => todo!(),
                }
            }
        }
        _ => todo!(),
    }
}
