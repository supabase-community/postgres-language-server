use pgls_query::protobuf::AIndirection;

use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind},
};

pub(super) fn emit_a_indirection(e: &mut EventEmitter, n: &AIndirection) {
    e.group_start(GroupKind::AIndirection);

    // Collect all indirections, flattening nested AIndirection nodes
    let (base_arg, all_indirections) = flatten_a_indirection(n);

    // Emit the base expression
    // Some expressions need parentheses when used with indirection (e.g., ROW(...), FuncCall for .*)
    let needs_parens = if let Some(ref arg) = base_arg {
        let has_indices = all_indirections
            .iter()
            .any(|node| matches!(node.node.as_ref(), Some(pgls_query::NodeEnum::AIndices(_))));

        let has_star_or_field = all_indirections.iter().any(|node| {
            matches!(
                node.node.as_ref(),
                Some(pgls_query::NodeEnum::AStar(_) | pgls_query::NodeEnum::String(_))
            )
        });

        let safe_without_parens = matches!(
            arg.node.as_ref(),
            Some(pgls_query::NodeEnum::ColumnRef(_) | pgls_query::NodeEnum::ParamRef(_))
        );

        // Function calls and type casts need parens for .* expansion and field access
        let needs_parens_for_field_access = matches!(
            arg.node.as_ref(),
            Some(
                pgls_query::NodeEnum::FuncCall(_)
                    | pgls_query::NodeEnum::JsonFuncExpr(_)
                    | pgls_query::NodeEnum::TypeCast(_)
            )
        );

        matches!(arg.node.as_ref(), Some(pgls_query::NodeEnum::RowExpr(_)))
            || (has_indices && !safe_without_parens)
            || (has_star_or_field && needs_parens_for_field_access)
    } else {
        false
    };

    if needs_parens {
        e.token(TokenKind::L_PAREN);
    }

    if let Some(ref arg) = base_arg {
        super::emit_node(arg, e);
    }

    if needs_parens {
        e.token(TokenKind::R_PAREN);
    }

    // Emit indirection operators (array subscripts, field selections)
    for indirection in &all_indirections {
        // Field selection and star expansion need a dot before them
        match &indirection.node {
            Some(pgls_query::NodeEnum::String(_)) | Some(pgls_query::NodeEnum::AStar(_)) => {
                e.token(TokenKind::DOT);
            }
            _ => {}
        }
        super::emit_node(indirection, e);
    }

    e.group_end();
}

/// Flatten nested AIndirection nodes into a single base expression and a list of all indirections
fn flatten_a_indirection(
    n: &AIndirection,
) -> (Option<Box<pgls_query::Node>>, Vec<pgls_query::Node>) {
    let mut all_indirections = Vec::new();
    let mut current = n;

    // Traverse nested AIndirection nodes
    loop {
        // Prepend current indirections (so inner ones come first)
        let mut current_indirections = current.indirection.clone();
        current_indirections.append(&mut all_indirections);
        all_indirections = current_indirections;

        // Check if arg is another AIndirection
        if let Some(ref arg) = current.arg {
            if let Some(pgls_query::NodeEnum::AIndirection(inner)) = arg.node.as_ref() {
                current = inner;
                continue;
            }
        }
        break;
    }

    (current.arg.clone(), all_indirections)
}
