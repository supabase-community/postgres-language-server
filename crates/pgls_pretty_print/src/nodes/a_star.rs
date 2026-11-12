use pgls_query::protobuf::AStar;

use super::prelude::*;

pub(super) fn emit_a_star(e: &mut EventEmitter, _n: &AStar) {
    e.token(TokenKind::IDENT("*".to_string()))
}
