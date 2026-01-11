use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
};
use pgls_query::{
    NodeEnum,
    protobuf::{RangeTblFunction, String as PgString},
};

pub(super) fn emit_range_tbl_function(e: &mut EventEmitter, n: &RangeTblFunction) {
    e.group_start(GroupKind::RangeTblFunction);

    if let Some(func) = n.funcexpr.as_ref() {
        super::emit_node(func, e);
    } else {
        super::emit_identifier(e, "function#expr");
    }

    let column_defs = collect_column_names(n);
    if !column_defs.is_empty() {
        e.space();
        e.token(TokenKind::AS_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        e.indent_start();

        for (idx, name) in column_defs.iter().enumerate() {
            if idx > 0 {
                e.token(TokenKind::COMMA);
                e.line(LineType::SoftOrSpace);
            }
            super::emit_identifier_maybe_quoted(e, name);
        }

        e.indent_end();
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::R_PAREN);
    }

    if !n.funcparams.is_empty() {
        e.space();
        e.token(TokenKind::IDENT("params".to_string()));
        e.space();
        e.token(TokenKind::L_PAREN);
        let mut first = true;
        for param in &n.funcparams {
            if !first {
                e.token(TokenKind::COMMA);
                e.space();
            }
            super::emit_identifier(e, &format!("${param}"));
            first = false;
        }
        e.token(TokenKind::R_PAREN);
    }

    e.group_end();
}

fn collect_column_names(function: &RangeTblFunction) -> Vec<String> {
    if function.funccolnames.is_empty() && function.funccolcount <= 0 {
        return Vec::new();
    }

    if !function.funccolnames.is_empty() {
        function
            .funccolnames
            .iter()
            .filter_map(|node| match node.node.as_ref() {
                Some(NodeEnum::String(PgString { sval, .. })) if !sval.is_empty() => {
                    Some(sval.clone())
                }
                _ => None,
            })
            .collect()
    } else {
        (0..function.funccolcount)
            .map(|idx| format!("col#{}", idx + 1))
            .collect()
    }
}
