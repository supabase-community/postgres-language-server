use pgls_query::protobuf::{CreateFunctionStmt, FunctionParameter, FunctionParameterMode};

use super::prelude::*;

pub(super) fn emit_create_function_stmt(e: &mut EventEmitter, n: &CreateFunctionStmt) {
    e.group_start();

    // Inner group for signature (CREATE [OR REPLACE] FUNCTION name(params))
    // This allows the signature to fit on one line while RETURNS/options break
    e.group_start();

    e.token(TokenKind::CREATE_KW);
    e.space();

    if n.replace {
        emit_or_replace(e);
        e.space();
    }

    if n.is_procedure {
        e.token(TokenKind::PROCEDURE_KW);
    } else {
        e.token(TokenKind::FUNCTION_KW);
    }

    e.line(LineType::SoftOrSpace);

    // Function name (qualified name)
    emit_dot_separated_list(e, &n.funcname);

    let mut regular_params: Vec<&FunctionParameter> = Vec::new();
    let mut table_params: Vec<&FunctionParameter> = Vec::new();

    for param in &n.parameters {
        if let Some(pgls_query::NodeEnum::FunctionParameter(fp)) = &param.node {
            if fp.mode() == FunctionParameterMode::FuncParamTable {
                table_params.push(fp);
            } else {
                regular_params.push(fp);
            }
        }
    }

    // Parameters
    e.token(TokenKind::L_PAREN);
    if !regular_params.is_empty() {
        e.indent_start();
        e.line(LineType::Soft);
        emit_function_parameter_list(e, &regular_params);
        e.indent_end();
        e.line(LineType::Soft);
    }
    e.token(TokenKind::R_PAREN);

    e.group_end(); // End signature group

    // Return type (only for functions, not procedures)
    if !table_params.is_empty() {
        e.line(LineType::Hard);
        e.token(TokenKind::RETURNS_KW);
        e.space();
        e.token(TokenKind::TABLE_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        e.indent_start();
        e.line(LineType::Soft);
        emit_function_parameter_list(e, &table_params);
        e.indent_end();
        e.line(LineType::Soft);
        e.token(TokenKind::R_PAREN);
    } else if !n.is_procedure
        && let Some(ref return_type) = n.return_type
    {
        e.line(LineType::Hard);
        e.token(TokenKind::RETURNS_KW);
        e.space();
        super::emit_type_name(e, return_type);
    }

    // Options - sort according to Postgres's canonical order
    let sorted_options = sort_function_options(&n.options);
    for option in sorted_options {
        if let Some(pgls_query::NodeEnum::DefElem(def_elem)) = &option.node {
            e.line(LineType::Hard);
            format_function_option(e, def_elem);
        }
    }

    // SQL body (if present, modern syntax)
    if let Some(ref sql_body) = n.sql_body {
        match sql_body.node.as_ref() {
            Some(pgls_query::NodeEnum::ReturnStmt(_)) => {
                e.line(LineType::Hard);
                emit_node(e, sql_body);
            }
            _ => {
                e.line(LineType::Hard);
                e.token(TokenKind::BEGIN_KW);
                e.space();
                e.token(TokenKind::ATOMIC_KW);
                e.indent_start();
                // A BEGIN ATOMIC body is a single-item list wrapping the list of statements. Each
                // statement ends with a semicolon, which the statement itself doesn't write. It
                // goes before the comments that follow the statement, as it does in the source.
                for stmt in begin_atomic_statements(sql_body) {
                    e.line(LineType::Hard);
                    emit_node_with(e, stmt, |e, stmt| {
                        emit_node_enum(e, stmt);
                        e.token(TokenKind::SEMICOLON);
                    });
                }
                e.indent_end();
                e.line(LineType::Hard);
                e.token(TokenKind::END_KW);
            }
        }
    }

    e.group_end();
}

fn begin_atomic_statements(sql_body: &pgls_query::protobuf::Node) -> &[pgls_query::protobuf::Node] {
    let Some(pgls_query::NodeEnum::List(outer)) = &sql_body.node else {
        return std::slice::from_ref(sql_body);
    };

    match outer.items.as_slice() {
        [inner] => match &inner.node {
            Some(pgls_query::NodeEnum::List(stmts)) => &stmts.items,
            // An empty body: the statement list is NIL
            None => &[],
            _ => &outer.items,
        },
        _ => &outer.items,
    }
}

fn emit_function_parameter_list(e: &mut EventEmitter, params: &[&FunctionParameter]) {
    for (index, param) in params.iter().enumerate() {
        if index > 0 {
            e.token(TokenKind::COMMA);
            e.line(LineType::SoftOrSpace);
        }
        super::emit_function_parameter(e, param);
    }
}
