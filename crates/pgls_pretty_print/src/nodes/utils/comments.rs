//! Comments a parent emits for a child it doesn't reach through `emit_node`.

use pgls_query::{Node, NodeEnum};

use crate::emitter::EventEmitter;

/// Emits `node` with `emit` between the comments attached to its location, the way `emit_node`
/// does with `emit_node_enum`.
pub(crate) fn emit_node_with(
    e: &mut EventEmitter,
    node: &Node,
    emit: impl FnOnce(&mut EventEmitter, &NodeEnum),
) {
    let location = node
        .node
        .as_ref()
        .and_then(|inner| crate::codegen::node_location::node_location(&inner.to_ref()));

    if let Some(location) = location {
        e.take_leading_comments_at(location);
    }

    if let Some(ref inner) = node.node {
        emit(e, inner)
    }

    if let Some(location) = location {
        e.take_trailing_comments_at(location);
    }
}

/// Emits `body` with the comments attached to `location`, the way `emit_node` does for a child
/// reached as a `Node`.
///
/// A parent that reaches a child through a typed emitter, `emit_range_var` for instance, never
/// goes through `emit_node`. Without this helper the comments attached to that child stay in the
/// emitter map and the whole statement is refused rather than reformatted.
pub(crate) fn emit_with_comments_at(
    e: &mut EventEmitter,
    location: i32,
    body: impl FnOnce(&mut EventEmitter),
) {
    // Negative locations are dropped when the comment maps are built, so they hold nothing.
    if location < 0 {
        body(e);
        return;
    }

    e.take_leading_comments_at(location);
    body(e);
    e.take_trailing_comments_at(location);
}

/// Emits the comments written before `keyword` whose node is `node`, the expression the parent
/// writes after the keyword. Call it right before the break that precedes the keyword.
///
/// The comments are attached to `node` because the keyword has no node of its own. Without this,
/// `then 'x' /* why */ else 'y'` would come out as `else /* why */ 'y'`.
pub(crate) fn emit_comments_before_keyword(e: &mut EventEmitter, keyword: &str, node: &Node) {
    if let Some(location) = node
        .node
        .as_ref()
        .and_then(|node| crate::comments::first_node_location(node.to_ref()))
    {
        e.take_comments_before_keyword_at(location, keyword);
    }
}

/// Emits the comments written before `keyword` right after the node at `location`, the last node
/// the parent writes before the keyword, as in `a /* c */ as x`. Call it right before the break
/// that precedes the keyword.
///
/// For a keyword with no child of its own after it, such as `AS` before an alias, the comments are
/// attached to whichever node follows in the source.
pub(crate) fn emit_comments_before_keyword_after(
    e: &mut EventEmitter,
    keyword: &str,
    location: i32,
) {
    if let Some(next) = e.next_node_location(location) {
        e.take_comments_before_keyword_at(next, keyword);
    }
}

/// Emits the comments written before a clause that the parent writes with a typed emitter and
/// that starts with a keyword at its own location, such as `ON CONFLICT` or `PARTITION BY`. Call
/// it right before the break that precedes the clause.
///
/// The comments lead the clause's node, but a typed emitter doesn't take them, and emitted after
/// the break they would no longer end the line they were written on.
pub(crate) fn emit_comments_before_clause(e: &mut EventEmitter, location: i32) {
    if location >= 0 {
        e.take_leading_comments_as_trailing_at(location);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::emitter::{EventEmitter, LayoutEvent};
    use crate::{Comment, TokenKind};

    #[test]
    fn a_typed_child_emitter_consumes_the_comments_of_its_location() {
        let comment = Comment {
            text: "-- note".to_string(),
            line_comment: true,
            own_line: true,
            after_separator: false,
            before_keyword: None,
        };
        let leading = HashMap::from([(7, vec![comment])]);
        let mut e = EventEmitter::with_comments(Default::default(), leading, HashMap::new());

        super::emit_with_comments_at(&mut e, 7, |e| e.token(TokenKind::ONLY_KW));

        assert_eq!(e.pending_comments(), 0);
        assert!(e.events.iter().any(|event| matches!(
            event,
            LayoutEvent::Comment { text, .. } if text == "-- note"
        )));
    }

    #[test]
    fn a_negative_location_never_carries_a_comment() {
        let mut e = EventEmitter::new(Default::default());

        super::emit_with_comments_at(&mut e, -1, |e| e.token(TokenKind::ONLY_KW));

        assert_eq!(e.events, vec![LayoutEvent::Token(TokenKind::ONLY_KW)]);
    }
}
