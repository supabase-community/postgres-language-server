use pg_query_proto_parser::{FieldType, Node, ProtoFile};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub fn get_node_properties_mod(proto_file: &ProtoFile) -> proc_macro2::TokenStream {
    let node_identifiers = node_identifiers(&proto_file.nodes);
    let node_handlers = node_handlers(&proto_file.nodes);

    quote! {
        #[derive(Debug, Clone, PartialEq)]
        pub struct TokenProperty {
            pub value: Option<String>,
            pub kind: Option<SyntaxKind>,
        }

        impl TokenProperty {
            pub fn new(value: Option<String>, kind: Option<SyntaxKind>) -> TokenProperty {
                if value.is_none() && kind.is_none() {
                    panic!("TokenProperty must have either value or kind");
                }
                TokenProperty { value, kind }
            }
        }

        impl From<i32> for TokenProperty {
            fn from(value: i32) -> TokenProperty {
                TokenProperty {
                    value: Some(value.to_string()),
                    kind: None,
                }
            }
        }

        impl From<u32> for TokenProperty {
            fn from(value: u32) -> TokenProperty {
                TokenProperty {
                    value: Some(value.to_string()),
                    kind: None,
                }
            }
        }


        impl From<i64> for TokenProperty {
            fn from(value: i64) -> TokenProperty {
                TokenProperty {
                    value: Some(value.to_string()),
                    kind: None,
                }
            }
        }

        impl From<u64> for TokenProperty {
            fn from(value: u64) -> TokenProperty {
                TokenProperty {
                    value: Some(value.to_string()),
                    kind: None,
                }
            }
        }

        impl From<f64> for TokenProperty {
            fn from(value: f64) -> TokenProperty {
                TokenProperty {
                    value: Some(value.to_string()),
                    kind: None,
                }
            }
        }

        impl From<bool> for TokenProperty {
            fn from(value: bool) -> TokenProperty {
                TokenProperty {
                    value: Some(value.to_string()),
                    kind: None,
                }
            }
        }

        impl From<String> for TokenProperty {
            fn from(value: String) -> TokenProperty {
                assert!(value.len() > 0, "String property value has length 0");
                TokenProperty {
                    value: Some(value.to_lowercase()),
                    kind: None,
                }
            }
        }


        impl From<&pg_query::protobuf::Integer> for TokenProperty {
            fn from(node: &pg_query::protobuf::Integer) -> TokenProperty {
                TokenProperty {
                        value: Some(node.ival.to_string()),
                        kind: Some(SyntaxKind::Iconst)
                    }
            }
        }

        impl From<&pg_query::protobuf::Boolean> for TokenProperty {
            fn from(node: &pg_query::protobuf::Boolean) -> TokenProperty {
                TokenProperty {
                        value: Some(node.boolval.to_string()),
                        kind: match node.boolval {
                            true => Some(SyntaxKind::TrueP),
                            false => Some(SyntaxKind::FalseP),
                        }
                    }
            }
        }

        impl From<SyntaxKind> for TokenProperty {
            fn from(kind: SyntaxKind) -> TokenProperty {
                TokenProperty {
                    value: None,
                    kind: Some(kind),
                }
            }
        }

        impl From<Token> for TokenProperty {
            fn from(token: Token) -> TokenProperty {
                TokenProperty {
                    value: None,
                    kind: Some(SyntaxKind::from(token)),
                }
            }
        }

        pub fn get_node_properties(node: &NodeEnum, parent: Option<&NodeEnum>) -> Vec<TokenProperty> {
            let mut tokens: Vec<TokenProperty> = Vec::new();

            match node {
                #(NodeEnum::#node_identifiers(n) => {#node_handlers}),*,
            };

            tokens
        }

    }
}

fn node_identifiers(nodes: &[Node]) -> Vec<Ident> {
    nodes
        .iter()
        .map(|node| format_ident!("{}", &node.name))
        .collect()
}

fn node_handlers(nodes: &[Node]) -> Vec<TokenStream> {
    nodes
        .iter()
        .map(|node| {
            let string_property_handlers = string_property_handlers(&node);
            let custom_handlers = custom_handlers(&node);
            quote! {
                #custom_handlers
                #(#string_property_handlers)*
            }
        })
        .collect()
}

fn custom_handlers(node: &Node) -> TokenStream {
    match node.name.as_str() {
        "SelectStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Select));
            if n.distinct_clause.len() > 0 {
                tokens.push(TokenProperty::from(Token::Distinct));
            }
            if n.values_lists.len() > 0 {
                tokens.push(TokenProperty::from(Token::Values));
            }
            if n.from_clause.len() > 0 {
                tokens.push(TokenProperty::from(Token::From));
            }
            if n.where_clause.is_some() {
                tokens.push(TokenProperty::from(Token::Where));
            }
            if n.group_clause.len() > 0 {
                tokens.push(TokenProperty::from(Token::GroupP));
                tokens.push(TokenProperty::from(Token::By));
            }
        },
        "BoolExpr" => quote! {
            match n.boolop() {
                protobuf::BoolExprType::AndExpr => tokens.push(TokenProperty::from(Token::And)),
                protobuf::BoolExprType::OrExpr => tokens.push(TokenProperty::from(Token::Or)),
                protobuf::BoolExprType::NotExpr => tokens.push(TokenProperty::from(Token::Not)),
                _ => panic!("Unknown BoolExpr {:#?}", n.boolop()),
            }
        },
        "JoinExpr" => quote! {
            tokens.push(TokenProperty::from(Token::Join));
            tokens.push(TokenProperty::from(Token::On));
            match n.jointype() {
                protobuf::JoinType::JoinInner => tokens.push(TokenProperty::from(Token::InnerP)),
                protobuf::JoinType::JoinLeft => tokens.push(TokenProperty::from(Token::Left)),
                protobuf::JoinType::JoinFull => tokens.push(TokenProperty::from(Token::Full)),
                protobuf::JoinType::JoinRight => tokens.push(TokenProperty::from(Token::Right)),
                _ => panic!("Unknown JoinExpr jointype {:#?}", n.jointype()),
            }

        },
        "ResTarget" => quote! {
            if n.name.len() > 0 {
                tokens.push(TokenProperty::from(Token::As));
            }
        },
        "Integer" => quote! {
            tokens.push(TokenProperty::from(n));
        },
        "DefElem" => quote! {
            match n.defname.as_str() {
                "location" => {
                    tokens.push(TokenProperty::from(Token::Default));
                },
                "connection_limit" => {
                    tokens.push(TokenProperty::from(Token::Limit));
                    tokens.push(TokenProperty::from(Token::Iconst));
                },
                "owner" => {
                    tokens.push(TokenProperty::from(Token::Owner));
                }
                _ => {}
            }
            match n.defaction() {
                protobuf::DefElemAction::DefelemUnspec => tokens.push(TokenProperty::from(Token::Ascii61)),
                _ => panic!("Unknown DefElem {:#?}", n.defaction()),
            }
        },
        "Alias" => quote! {
            tokens.push(TokenProperty::from(Token::As));
        },
        "CollateClause" => quote! {
            tokens.push(TokenProperty::from(Token::Collate));
        },
        "AExpr" => quote! {
            match n.kind() {
                protobuf::AExprKind::AexprOp => {}, // do nothing
                protobuf::AExprKind::AexprOpAny => tokens.push(TokenProperty::from(Token::Any)),
                protobuf::AExprKind::AexprIn => tokens.push(TokenProperty::from(Token::InP)),
                _ => panic!("Unknown AExpr kind {:#?}", n.kind()),
            }
        },
        "WindowDef" => quote! {
            if n.partition_clause.len() > 0 || n.order_clause.len() > 0 {
                tokens.push(TokenProperty::from(Token::Window));
                tokens.push(TokenProperty::from(Token::As));
            }
            if n.partition_clause.len() > 0 {
                tokens.push(TokenProperty::from(Token::Partition));
                tokens.push(TokenProperty::from(Token::By));
            }
        },
        "Boolean" => quote! {
            tokens.push(TokenProperty::from(n));
        },
        "AStar" => quote! {
            tokens.push(TokenProperty::from(Token::Ascii42));
        },
        "FuncCall" => quote! {
            if n.funcname.len() == 1 && n.args.len() == 0 {
                // check if count(*)
                if let Some(node) = &n.funcname[0].node {
                    if let NodeEnum::String(n) = node {
                        if n.sval == "count" {
                            tokens.push(TokenProperty::from(Token::Ascii42));
                        }
                    }
                }
            }
            if n.agg_filter.is_some() {
                tokens.push(TokenProperty::from(Token::Filter));
                tokens.push(TokenProperty::from(Token::Where));
            }
            if n.over.is_some() {
                tokens.push(TokenProperty::from(Token::Over));
            }
        },
        "SqlvalueFunction" => quote! {
            match n.op() {
                protobuf::SqlValueFunctionOp::SvfopCurrentRole => tokens.push(TokenProperty::from(Token::CurrentRole)),
                protobuf::SqlValueFunctionOp::SvfopCurrentUser => tokens.push(TokenProperty::from(Token::CurrentUser)),
                _ => panic!("Unknown SqlvalueFunction {:#?}", n.op()),
            }
        },
        "SortBy" => quote! {
            tokens.push(TokenProperty::from(Token::Order));
            tokens.push(TokenProperty::from(Token::By));
            match n.sortby_dir() {
                protobuf::SortByDir::SortbyAsc => tokens.push(TokenProperty::from(Token::Asc)),
                protobuf::SortByDir::SortbyDesc => tokens.push(TokenProperty::from(Token::Desc)),
                _ => {}
            }
        },
        "AConst" => quote! {
            if n.isnull {
                tokens.push(TokenProperty::from(Token::NullP));
            }
        },
        "AlterTableStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Alter));
            tokens.push(TokenProperty::from(Token::Table));
        },
        "AlterTableCmd" => quote! {
            tokens.push(TokenProperty::from(Token::Alter));
            match n.subtype() {
                protobuf::AlterTableType::AtColumnDefault => {
                    tokens.push(TokenProperty::from(Token::Column));
                    tokens.push(TokenProperty::from(Token::Set));
                    tokens.push(TokenProperty::from(Token::Default));
                },
                protobuf::AlterTableType::AtAddConstraint => tokens.push(TokenProperty::from(Token::AddP)),
                protobuf::AlterTableType::AtAlterColumnType => {
                    tokens.push(TokenProperty::from(Token::Alter));
                    tokens.push(TokenProperty::from(Token::Column));
                    tokens.push(TokenProperty::from(Token::TypeP));
                },
                _ => panic!("Unknown AlterTableCmd {:#?}", n.subtype()),
            }
        },
        "VariableSetStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Set));
            match n.kind() {
                protobuf::VariableSetKind::VarSetValue => tokens.push(TokenProperty::from(Token::To)),
                _ => panic!("Unknown VariableSetStmt {:#?}", n.kind()),
            }
        },
        "CreatePolicyStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            tokens.push(TokenProperty::from(Token::Policy));
            tokens.push(TokenProperty::from(Token::On));
            if n.roles.len() > 0 {
                tokens.push(TokenProperty::from(Token::To));
            }
            if n.qual.is_some() {
                tokens.push(TokenProperty::from(Token::Using));
            }
            if n.with_check.is_some() {
                tokens.push(TokenProperty::from(Token::With));
                tokens.push(TokenProperty::from(Token::Check));
            }
        },
        "CopyStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Copy));
            tokens.push(TokenProperty::from(Token::From));
        },
        "RenameStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Alter));
            tokens.push(TokenProperty::from(Token::Table));
            tokens.push(TokenProperty::from(Token::Rename));
            tokens.push(TokenProperty::from(Token::To));
        },
        "Constraint" => quote! {
            match n.contype() {
                protobuf::ConstrType::ConstrNotnull => {
                    tokens.push(TokenProperty::from(Token::Not));
                    tokens.push(TokenProperty::from(Token::NullP));
                },
                protobuf::ConstrType::ConstrDefault => tokens.push(TokenProperty::from(Token::Default)),
                protobuf::ConstrType::ConstrCheck => tokens.push(TokenProperty::from(Token::Check)),
                protobuf::ConstrType::ConstrPrimary => {
                    tokens.push(TokenProperty::from(Token::Primary));
                    tokens.push(TokenProperty::from(Token::Key));
                },
                protobuf::ConstrType::ConstrForeign => tokens.push(TokenProperty::from(Token::References)),
                _ => panic!("Unknown Constraint {:#?}", n.contype()),
            }
        },
        "PartitionSpec" => quote! {
            tokens.push(TokenProperty::from(Token::Partition));
            tokens.push(TokenProperty::from(Token::By));
        },
        "InsertStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Insert));
            tokens.push(TokenProperty::from(Token::Into));
        },
        "DeleteStmt" => quote! {
            tokens.push(TokenProperty::from(Token::DeleteP));
            tokens.push(TokenProperty::from(Token::From));
            if n.where_clause.is_some() {
                tokens.push(TokenProperty::from(Token::Where));
            }
            if n.using_clause.len() > 0 {
                tokens.push(TokenProperty::from(Token::Using));
            }
        },
        "ViewStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            tokens.push(TokenProperty::from(Token::View));
            if n.query.is_some() {
                tokens.push(TokenProperty::from(Token::As));
            }
            if n.replace {
                tokens.push(TokenProperty::from(Token::Or));
                tokens.push(TokenProperty::from(Token::Replace));
            }
            if let Some(n) = &n.view {
                match n.relpersistence.as_str() {
                    // Temporary
                    "t" => tokens.push(TokenProperty::from(Token::Temporary)),
                    _ => {},
                }
            }
            match n.with_check_option() {
                protobuf::ViewCheckOption::LocalCheckOption => {
                    tokens.push(TokenProperty::from(Token::With));
                    tokens.push(TokenProperty::from(Token::Local));
                    tokens.push(TokenProperty::from(Token::Check));
                    tokens.push(TokenProperty::from(Token::Option));
                },
                protobuf::ViewCheckOption::CascadedCheckOption => {
                    tokens.push(TokenProperty::from(Token::With));
                    tokens.push(TokenProperty::from(Token::Cascaded));
                    tokens.push(TokenProperty::from(Token::Check));
                    tokens.push(TokenProperty::from(Token::Option));
                },
                _ => {}
            }
        },
        "CreateStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            tokens.push(TokenProperty::from(Token::Table));
            if n.tablespacename.len() > 0 {
                tokens.push(TokenProperty::from(Token::Tablespace));
            }
            if n.options.len() > 0 {
                tokens.push(TokenProperty::from(Token::With));
            }
            if n.if_not_exists {
                tokens.push(TokenProperty::from(Token::IfP));
                tokens.push(TokenProperty::from(Token::Not));
                tokens.push(TokenProperty::from(Token::Exists));
            }
            if n.partbound.is_some() {
                tokens.push(TokenProperty::from(Token::Partition));
                tokens.push(TokenProperty::from(Token::Of));
                tokens.push(TokenProperty::from(Token::For));
                tokens.push(TokenProperty::from(Token::Values));
            }
        },
        "PartitionBoundSpec" => quote! {
            tokens.push(TokenProperty::from(Token::From));
            tokens.push(TokenProperty::from(Token::To));
        },
        "CaseExpr" => quote! {
            tokens.push(TokenProperty::from(Token::Case));
            tokens.push(TokenProperty::from(Token::EndP));
            if n.defresult.is_some() {
                tokens.push(TokenProperty::from(Token::Else));
            }
        },
        "NullTest" => quote! {
            match n.nulltesttype() {
                protobuf::NullTestType::IsNull => tokens.push(TokenProperty::from(Token::Is)),
                protobuf::NullTestType::IsNotNull => {
                    tokens.push(TokenProperty::from(Token::Is));
                    tokens.push(TokenProperty::from(Token::Not));
                },
                _ => panic!("Unknown NullTest {:#?}", n.nulltesttype()),
            }
            tokens.push(TokenProperty::from(Token::NullP));
        },
        "CreateFunctionStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            if n.is_procedure {
                tokens.push(TokenProperty::from(Token::Procedure));
            } else {
                tokens.push(TokenProperty::from(Token::Function));
            }
            if n.replace {
                tokens.push(TokenProperty::from(Token::Or));
                tokens.push(TokenProperty::from(Token::Replace));
            }
            if let Some(return_type) = &n.return_type {
                tokens.push(TokenProperty::from(Token::Returns));
                if return_type.setof {
                    tokens.push(TokenProperty::from(Token::Setof));
                }
            }
            for option in &n.options {
                if let Some(NodeEnum::DefElem(node)) = &option.node {
                    if node.defname == "strict" {
                        if let Some(NodeEnum::Boolean(node)) =
                            node.arg.as_ref().and_then(|arg| arg.node.as_ref())
                        {
                            if node.boolval {
                                tokens.push(TokenProperty::from(Token::NullP));
                                tokens.push(TokenProperty::from(Token::On));
                                tokens.push(TokenProperty::from(Token::NullP));
                                tokens.push(TokenProperty::from(Token::InputP));
                            } else {
                                tokens.push(TokenProperty::from(Token::On));
                                tokens.push(TokenProperty::from(Token::NullP));
                                tokens.push(TokenProperty::from(Token::InputP));
                            }
                        }
                    }
                }
            }
        },
        "FunctionParameter" => quote! {
            match n.mode() {
                protobuf::FunctionParameterMode::FuncParamIn => tokens.push(TokenProperty::from(Token::InP)),
                protobuf::FunctionParameterMode::FuncParamOut => tokens.push(TokenProperty::from(Token::OutP)),
                protobuf::FunctionParameterMode::FuncParamInout => tokens.push(TokenProperty::from(Token::Inout)),
                protobuf::FunctionParameterMode::FuncParamVariadic => tokens.push(TokenProperty::from(Token::Variadic)),
                // protobuf::FunctionParameterMode::FuncParamTable => tokens.push(TokenProperty::from(Token::Table)),
                protobuf::FunctionParameterMode::FuncParamDefault => {}, // do nothing
                _ => panic!("Unknown FunctionParameter {:#?}", n.mode()),
            };
            if n.defexpr.is_some() {
                tokens.push(TokenProperty::from(Token::Default));
            }
        },
        "NamedArgExpr" => quote! {
            // =>
            tokens.push(TokenProperty::from(Token::EqualsGreater));
        },
        "CaseWhen" => quote! {
            tokens.push(TokenProperty::from(Token::When));
            tokens.push(TokenProperty::from(Token::Then));
        },
        "TypeCast" => quote! {
            tokens.push(TokenProperty::from(Token::Typecast));
        },
        "CreateDomainStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            tokens.push(TokenProperty::from(Token::DomainP));
            if n.type_name.is_some() {
                tokens.push(TokenProperty::from(Token::As));
            }
        },
        "List" => quote! {
            if parent.is_some() {
                // if parent is `DefineStmt`, we need to check whether an ORDER BY needs to be added
                if let NodeEnum::DefineStmt(define_stmt) = parent.unwrap() {
                    // there *seems* to be an integer node in the last position of the DefineStmt args that
                    // defines whether the list contains an order by statement
                    let integer = define_stmt.args.last()
                        .and_then(|node| node.node.as_ref())
                        .and_then(|node| if let NodeEnum::Integer(n) = node { Some(n.ival) } else { None });
                    if integer.is_none() {
                        panic!("DefineStmt of type ObjectAggregate has no integer node in last position of args");
                    }
                    // if the integer is 1, then there is an order by statement
                    // we add it to the `List` node because that seems to make most sense based off the grammar definition
                    // ref: https://github.com/postgres/postgres/blob/REL_15_STABLE/src/backend/parser/gram.y#L8355
                    // ```
                    //  aggr_args:
                    //    | '(' aggr_args_list ORDER BY aggr_args_list ')'
                    // ```
                    if integer.unwrap() == 1 {
                        tokens.push(TokenProperty::from(Token::Order));
                        tokens.push(TokenProperty::from(Token::By));
                    }
                }
            }
        },
        "DefineStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            if n.replace {
                tokens.push(TokenProperty::from(Token::Or));
                tokens.push(TokenProperty::from(Token::Replace));
            }
            match n.kind() {
                protobuf::ObjectType::ObjectAggregate => {
                    tokens.push(TokenProperty::from(Token::Aggregate));

                    // n.args is always an array with two nodes
                    assert_eq!(n.args.len(), 2, "DefineStmt of type ObjectAggregate does not have exactly 2 args");
                    // the first is either a List or a Node { node: None }

                    if let Some(node) = &n.args.first() {
                        if node.node.is_none() {
                            // if first element is a Node { node: None }, then it's "*"
                            tokens.push(TokenProperty::from(Token::Ascii42));
                        }                     }
                        // if its a list, we handle it in the handler for `List`
                },
                _ => panic!("Unknown DefineStmt {:#?}", n.kind()),
            }
        },
        "CreateSchemaStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            tokens.push(TokenProperty::from(Token::Schema));
            if n.if_not_exists {
                tokens.push(TokenProperty::from(Token::IfP));
                tokens.push(TokenProperty::from(Token::Not));
                tokens.push(TokenProperty::from(Token::Exists));
            }
            if n.authrole.is_some() {
                tokens.push(TokenProperty::from(Token::Authorization));
            }
        },
        "CreateEnumStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            tokens.push(TokenProperty::from(Token::TypeP));
            tokens.push(TokenProperty::from(Token::As));
            tokens.push(TokenProperty::from(Token::EnumP));
        },
        "CreateCastStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            tokens.push(TokenProperty::from(Token::Cast));
            tokens.push(TokenProperty::from(Token::As));
            if n.inout {
                tokens.push(TokenProperty::from(Token::With));
                tokens.push(TokenProperty::from(Token::Inout));
            } else if n.func.is_some() {
                tokens.push(TokenProperty::from(Token::With));
                tokens.push(TokenProperty::from(Token::Function));
            } else {
                tokens.push(TokenProperty::from(Token::Without));
                tokens.push(TokenProperty::from(Token::Function));
            }
            match n.context() {
                protobuf::CoercionContext::CoercionImplicit => {
                    tokens.push(TokenProperty::from(Token::As));
                    tokens.push(TokenProperty::from(Token::ImplicitP));
                },
                protobuf::CoercionContext::CoercionAssignment => {
                    tokens.push(TokenProperty::from(Token::As));
                    tokens.push(TokenProperty::from(Token::Assignment));
                },
                protobuf::CoercionContext::CoercionPlpgsql => {},
                protobuf::CoercionContext::CoercionExplicit => {},
                _ => panic!("Unknown CreateCastStmt {:#?}", n.context())
            }
        },
        "CreateRangeStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            tokens.push(TokenProperty::from(Token::TypeP));
            tokens.push(TokenProperty::from(Token::As));
            tokens.push(TokenProperty::from(Token::Range));
        },
        "IndexStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            if n.unique {
                tokens.push(TokenProperty::from(Token::Unique));
            }
            tokens.push(TokenProperty::from(Token::Index));
            if n.concurrent {
                tokens.push(TokenProperty::from(Token::Concurrently));
            }
            if n.if_not_exists {
                tokens.push(TokenProperty::from(Token::IfP));
                tokens.push(TokenProperty::from(Token::Not));
                tokens.push(TokenProperty::from(Token::Exists));
            }
            tokens.push(TokenProperty::from(Token::On));
            // access_method is btree by default
            if n.access_method.len() > 0 {
                tokens.push(TokenProperty::from(Token::Using));
            }
            if n.index_including_params.len() > 0 {
                tokens.push(TokenProperty::from(Token::Include));
            }
            if n.options.len() > 0 {
                tokens.push(TokenProperty::from(Token::With));
            }
            // table_space is an empty string by default
            if n.table_space.len() > 0 {
                tokens.push(TokenProperty::from(Token::Tablespace));
            }
        },
        "IndexElem" => quote! {
            if n.collation.len() > 0 {
                tokens.push(TokenProperty::from(Token::Collate));
            }
            match n.nulls_ordering() {
                protobuf::SortByNulls::SortbyNullsDefault => {},
                protobuf::SortByNulls::SortbyNullsFirst => {
                    tokens.push(TokenProperty::from(Token::NullsP));
                    tokens.push(TokenProperty::from(Token::FirstP));
                },
                protobuf::SortByNulls::SortbyNullsLast => {
                    tokens.push(TokenProperty::from(Token::NullsP));
                    tokens.push(TokenProperty::from(Token::LastP));
                },
                _ => panic!("Unknown IndexElem {:#?}", n.nulls_ordering()),
            }
        },
        "CreateTableSpaceStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            tokens.push(TokenProperty::from(Token::Tablespace));
            tokens.push(TokenProperty::from(Token::Location));
            if n.owner.is_some() {
                tokens.push(TokenProperty::from(Token::Owner));
            }
            if n.options.len() > 0 {
                tokens.push(TokenProperty::from(Token::With));
            }
        },
        "CompositeTypeStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            tokens.push(TokenProperty::from(Token::TypeP));
        },
        "CreatedbStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            tokens.push(TokenProperty::from(Token::Database));
        },
        "CreatePublicationStmt" => quote! {
            tokens.push(TokenProperty::from(Token::Create));
            tokens.push(TokenProperty::from(Token::Publication));
            tokens.push(TokenProperty::from(Token::For));
            tokens.push(TokenProperty::from(Token::Tables));
            tokens.push(TokenProperty::from(Token::InP));
            tokens.push(TokenProperty::from(Token::Schema));
        },
        _ => quote! {},
    }
}

fn string_property_handlers(node: &Node) -> Vec<TokenStream> {
    node.fields
        .iter()
        .filter_map(|field| {
            if field.repeated {
                return None;
            }
            let field_name = format_ident!("{}", field.name.as_str());
            match field.field_type {
                // just handle string values for now
                FieldType::String => Some(quote! {
                    // most string values are never None, but an empty string
                    if n.#field_name.len() > 0 {
                        tokens.push(TokenProperty::from(n.#field_name.to_owned()));
                    }
                }),
                _ => None,
            }
        })
        .collect()
}
