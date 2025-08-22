use crate::{
    TokenKind,
    emitter::{EventEmitter, LineType, ToTokens},
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
            pgt_query::protobuf::node::Node::ResTarget(target) => target.to_tokens(e),
            pgt_query::protobuf::node::Node::ColumnRef(col_ref) => col_ref.to_tokens(e),
            pgt_query::protobuf::node::Node::String(string) => string.to_tokens(e),
            pgt_query::protobuf::node::Node::RangeVar(string) => string.to_tokens(e),
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

        if !self.target_list.is_empty() {
            e.indent_start();
            e.line(LineType::SoftOrSpace);

            for (i, target) in self.target_list.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                target.to_tokens(e);
            }
            e.indent_end();
        }

        if !self.from_clause.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::FROM_KW);
            e.line(LineType::SoftOrSpace);

            e.indent_start();
            for (i, from) in self.from_clause.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                from.to_tokens(e);
            }
            e.indent_end();
        }

        e.token(TokenKind::SEMICOLON);

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ResTarget {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(None, false);

        if let Some(ref val) = self.val {
            val.to_tokens(e);
        }

        if !self.name.is_empty() {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            e.token(TokenKind::IDENT(self.name.clone()));
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ColumnRef {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(None, false);

        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            field.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::String {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.sval.clone()));
    }
}

impl ToTokens for pgt_query::protobuf::RangeVar {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(None, false);

        if !self.schemaname.is_empty() {
            e.token(TokenKind::IDENT(self.schemaname.clone()));
            e.token(TokenKind::DOT);
        }

        e.token(TokenKind::IDENT(self.relname.clone()));

        e.group_end();
    }
}

#[cfg(test)]
mod test {
    use crate::emitter::{EventEmitter, ToTokens};

    use insta::assert_debug_snapshot;

    #[test]
    fn simple_select() {
        let input = "select public.t.a as y, b as z, c from t where id = @id;";

        let parsed = pgt_query::parse(input).expect("Failed to parse SQL");

        let ast = parsed.root().expect("No root node found");

        let mut emitter = EventEmitter::new();
        ast.to_tokens(&mut emitter);

        assert_debug_snapshot!(emitter.events);
    }
}
