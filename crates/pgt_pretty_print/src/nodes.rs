use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType, ToTokens},
};

impl ToTokens for pgt_query::NodeEnum {
    fn to_tokens(&self, e: &mut EventEmitter) {
        match self {
            pgt_query::protobuf::node::Node::SelectStmt(stmt) => stmt.as_ref().to_tokens(e),
            pgt_query::protobuf::node::Node::ResTarget(target) => target.to_tokens(e),
            pgt_query::protobuf::node::Node::ColumnRef(col_ref) => col_ref.to_tokens(e),
            pgt_query::protobuf::node::Node::String(string) => string.to_tokens(e),
            pgt_query::protobuf::node::Node::RangeVar(string) => string.to_tokens(e),
            pgt_query::protobuf::node::Node::FuncCall(func_call) => func_call.to_tokens(e),
            pgt_query::protobuf::node::Node::InsertStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::List(list) => list.to_tokens(e),
            pgt_query::protobuf::node::Node::AConst(const_val) => const_val.to_tokens(e),
            pgt_query::protobuf::node::Node::DeleteStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AExpr(expr) => expr.to_tokens(e),
            pgt_query::protobuf::node::Node::UpdateStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ColumnDef(def) => def.to_tokens(e),
            pgt_query::protobuf::node::Node::TypeName(type_name) => type_name.to_tokens(e),
            pgt_query::protobuf::node::Node::DropStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::TruncateStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterTableStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterTableCmd(cmd) => cmd.to_tokens(e),
            pgt_query::protobuf::node::Node::ViewStmt(stmt) => stmt.to_tokens(e),
            _ => {
                unimplemented!("Node type {:?} not implemented for to_tokens", self);
            }
        }
    }
}

impl ToTokens for pgt_query::Node {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(node) = &self.node {
            match node {
                pgt_query::protobuf::node::Node::SelectStmt(stmt) => stmt.as_ref().to_tokens(e),
                pgt_query::protobuf::node::Node::ResTarget(target) => target.to_tokens(e),
                pgt_query::protobuf::node::Node::ColumnRef(col_ref) => col_ref.to_tokens(e),
                pgt_query::protobuf::node::Node::String(string) => string.to_tokens(e),
                pgt_query::protobuf::node::Node::RangeVar(string) => string.to_tokens(e),
                pgt_query::protobuf::node::Node::FuncCall(func_call) => func_call.to_tokens(e),
                pgt_query::protobuf::node::Node::InsertStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::List(list) => list.to_tokens(e),
                pgt_query::protobuf::node::Node::AConst(const_val) => const_val.to_tokens(e),
                pgt_query::protobuf::node::Node::DeleteStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AExpr(expr) => expr.to_tokens(e),
                pgt_query::protobuf::node::Node::UpdateStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ColumnDef(def) => def.to_tokens(e),
                pgt_query::protobuf::node::Node::TypeName(type_name) => type_name.to_tokens(e),
                pgt_query::protobuf::node::Node::DropStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::TruncateStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterTableStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterTableCmd(cmd) => cmd.to_tokens(e),
                pgt_query::protobuf::node::Node::ViewStmt(stmt) => stmt.to_tokens(e),
                _ => {
                    unimplemented!("Node type {:?} not implemented for to_tokens", node);
                }
            }
        }
    }
}

impl ToTokens for pgt_query::protobuf::SelectStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::SelectStmt, None, false);

        if !self.values_lists.is_empty() {
            e.token(TokenKind::VALUES_KW);
            e.space();
            for (i, values_list) in self.values_lists.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                values_list.to_tokens(e);
            }
        } else {
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
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ResTarget {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ResTarget, None, false);

        if e.is_within_group(GroupKind::UpdateStmt) {
            if !self.name.is_empty() {
                e.token(TokenKind::IDENT(self.name.clone()));
                e.space();
                e.token(TokenKind::IDENT("=".to_string()));
                e.space();
            }
            if let Some(ref val) = self.val {
                val.to_tokens(e);
            }
        } else {
            if let Some(ref val) = self.val {
                val.to_tokens(e);
                if !self.name.is_empty() {
                    e.space();
                    e.token(TokenKind::AS_KW);
                    e.space();
                    e.token(TokenKind::IDENT(self.name.clone()));
                }
            } else if !self.name.is_empty() {
                e.token(TokenKind::IDENT(self.name.clone()));
            }
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ColumnRef {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ColumnRef, None, false);

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
        e.group_start(GroupKind::RangeVar, None, false);

        if !self.schemaname.is_empty() {
            e.token(TokenKind::IDENT(self.schemaname.clone()));
            e.token(TokenKind::DOT);
        }

        e.token(TokenKind::IDENT(self.relname.clone()));

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::FuncCall {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::FuncCall, None, false);

        for (i, name) in self.funcname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.line(LineType::SoftOrSpace);
            }
            name.to_tokens(e);
        }

        e.token(TokenKind::L_PAREN);

        if !self.args.is_empty() {
            e.group_start(GroupKind::FuncCall, None, true);
            e.line(LineType::SoftOrSpace);
            e.indent_start();

            for (i, arg) in self.args.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.line(LineType::SoftOrSpace);
                }
                arg.to_tokens(e);
            }

            e.indent_end();
            e.line(LineType::SoftOrSpace);
            e.group_end();
        }

        e.token(TokenKind::R_PAREN);
        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::InsertStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::InsertStmt, None, false);

        e.token(TokenKind::INSERT_KW);
        e.space();
        e.token(TokenKind::INTO_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if !self.cols.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, col) in self.cols.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                col.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if let Some(ref select_stmt) = self.select_stmt {
            e.space();
            select_stmt.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::List {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::List, None, false);

        if e.is_within_group(GroupKind::DropStmt) {
            for (i, item) in self.items.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                item.to_tokens(e);
            }
        } else {
            e.token(TokenKind::L_PAREN);
            for (i, item) in self.items.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                item.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AConst {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref val) = self.val {
            match val {
                pgt_query::protobuf::a_const::Val::Ival(ival) => {
                    e.token(TokenKind::IDENT(ival.ival.to_string()));
                }
                pgt_query::protobuf::a_const::Val::Fval(fval) => {
                    e.token(TokenKind::IDENT(fval.fval.clone()));
                }
                pgt_query::protobuf::a_const::Val::Boolval(boolval) => {
                    let val_str = if boolval.boolval { "TRUE" } else { "FALSE" };
                    e.token(TokenKind::IDENT(val_str.to_string()));
                }
                pgt_query::protobuf::a_const::Val::Sval(sval) => {
                    e.token(TokenKind::STRING(format!("'{}'", sval.sval)));
                }
                pgt_query::protobuf::a_const::Val::Bsval(bsval) => {
                    e.token(TokenKind::STRING(bsval.bsval.clone()));
                }
            }
        }
    }
}

impl ToTokens for pgt_query::protobuf::DeleteStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DeleteStmt, None, false);

        e.token(TokenKind::DELETE_KW);
        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if let Some(ref where_clause) = self.where_clause {
            e.space();
            e.token(TokenKind::WHERE_KW);
            e.space();
            where_clause.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AExpr {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AExpr, None, false);

        if let Some(ref lexpr) = self.lexpr {
            lexpr.to_tokens(e);
        }

        if !self.name.is_empty() {
            e.space();
            for name in &self.name {
                name.to_tokens(e);
            }
            e.space();
        }

        if let Some(ref rexpr) = self.rexpr {
            rexpr.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::UpdateStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::UpdateStmt, None, false);

        e.token(TokenKind::UPDATE_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if !self.target_list.is_empty() {
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            for (i, target) in self.target_list.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                target.to_tokens(e);
            }
        }

        if let Some(ref where_clause) = self.where_clause {
            e.space();
            e.token(TokenKind::WHERE_KW);
            e.space();
            where_clause.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::TABLE_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if !self.table_elts.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, elt) in self.table_elts.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                elt.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ColumnDef {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ColumnDef, None, false);

        e.token(TokenKind::IDENT(self.colname.clone()));

        if let Some(ref type_name) = self.type_name {
            e.space();
            type_name.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::TypeName {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::TypeName, None, false);

        for (i, name) in self.names.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DropStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DropStmt, None, false);

        e.token(TokenKind::DROP_KW);
        e.space();
        e.token(TokenKind::TABLE_KW);
        e.space();

        for (i, obj) in self.objects.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            obj.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::TruncateStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::TruncateStmt, None, false);

        e.token(TokenKind::TRUNCATE_KW);
        e.space();
        e.token(TokenKind::TABLE_KW);
        e.space();

        for (i, rel) in self.relations.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            rel.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterTableStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterTableStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::TABLE_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        for (i, cmd) in self.cmds.iter().enumerate() {
            if i == 0 {
                e.space();
            } else {
                e.token(TokenKind::COMMA);
                e.space();
            }
            cmd.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterTableCmd {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterTableCmd, None, false);

        use pgt_query::protobuf::AlterTableType;

        match AlterTableType::try_from(self.subtype).unwrap() {
            AlterTableType::AtAddColumn => {
                e.token(TokenKind::ADD_KW);
                e.space();
                e.token(TokenKind::COLUMN_KW);
                if let Some(ref def) = self.def {
                    e.space();
                    def.to_tokens(e);
                }
            }
            AlterTableType::AtDropColumn => {
                e.token(TokenKind::DROP_KW);
                e.space();
                e.token(TokenKind::COLUMN_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
            _ => todo!(),
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ViewStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ViewStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::VIEW_KW);
        e.space();

        if let Some(ref view) = self.view {
            view.to_tokens(e);
        }

        if let Some(ref query) = self.query {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            query.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

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
