use crate::{
    TokenKind,
    emitter::{EventEmitter, ToTokens},
};

impl ToTokens for pgt_query::Node {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(node) = &self.node {
            node.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::node::Node {
    fn to_tokens(&self, e: &mut EventEmitter) {
        match self {
            pgt_query::protobuf::node::Node::SelectStmt(stmt) => stmt.as_ref().to_tokens(e),
            _ => {
                unimplemented!("Node type {:?} not implemented for to_tokens", self);
            }
        }
    }
}

impl ToTokens for pgt_query::protobuf::SelectStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(None, false);
        e.token(TokenKind::SELECT_KW);
        e.space();
        self.target_list
            .iter()
            .for_each(|target| target.to_tokens(e));
        e.space();
        self.from_clause.iter().for_each(|from| from.to_tokens(e));
        e.group_end();
    }
}
