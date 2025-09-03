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
            pgt_query::protobuf::node::Node::MergeStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::MergeWhenClause(clause) => clause.to_tokens(e),
            pgt_query::protobuf::node::Node::Alias(alias) => alias.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateSchemaStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::RoleSpec(role) => role.to_tokens(e),
            pgt_query::protobuf::node::Node::GrantStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AccessPriv(privilege) => privilege.to_tokens(e),
            pgt_query::protobuf::node::Node::TransactionStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::VariableSetStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::IndexStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::IndexElem(elem) => elem.to_tokens(e),
            pgt_query::protobuf::node::Node::CopyStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DefElem(elem) => elem.to_tokens(e),
            pgt_query::protobuf::node::Node::Boolean(b) => b.to_tokens(e),
            pgt_query::protobuf::node::Node::GrantRoleStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterDefaultPrivilegesStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::VariableShowStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateTableSpaceStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DropTableSpaceStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterTableSpaceOptionsStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::Float(f) => f.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterTableMoveAllStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateExtensionStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CommentStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterExtensionStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterExtensionContentsStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ObjectWithArgs(obj) => obj.to_tokens(e),
            pgt_query::protobuf::node::Node::FunctionParameter(param) => param.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateFdwStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateRoleStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::SetOperationStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateForeignServerStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterFdwStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterForeignServerStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateForeignTableStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateUserMappingStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterUserMappingStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DropUserMappingStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ImportForeignSchemaStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreatePolicyStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterPolicyStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateAmStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateSeqStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterSeqStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::Integer(i) => i.to_tokens(e),
            pgt_query::protobuf::node::Node::DefineStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateDomainStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CollateClause(clause) => clause.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterDomainStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::Constraint(constraint) => constraint.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateOpClassStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateOpClassItem(item) => item.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateOpFamilyStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterOpFamilyStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ReplicaIdentityStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::SecLabelStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterCollationStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DeclareCursorStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ClosePortalStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::FetchStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AStar(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::ReturnStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateStatsStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::StatsElem(elem) => elem.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterRoleStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterRoleSetStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::DropRoleStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::AlterStatsStmt(stmt) => stmt.to_tokens(e),
            pgt_query::protobuf::node::Node::CreateFunctionStmt(stmt) => stmt.to_tokens(e),
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
                pgt_query::protobuf::node::Node::MergeStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::MergeWhenClause(clause) => clause.to_tokens(e),
                pgt_query::protobuf::node::Node::Alias(alias) => alias.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateSchemaStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::RoleSpec(role) => role.to_tokens(e),
                pgt_query::protobuf::node::Node::GrantStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AccessPriv(privilege) => privilege.to_tokens(e),
                pgt_query::protobuf::node::Node::TransactionStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::VariableSetStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::IndexStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::IndexElem(elem) => elem.to_tokens(e),
                pgt_query::protobuf::node::Node::CopyStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DefElem(elem) => elem.to_tokens(e),
                pgt_query::protobuf::node::Node::Boolean(b) => b.to_tokens(e),
                pgt_query::protobuf::node::Node::GrantRoleStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterDefaultPrivilegesStmt(stmt) => {
                    stmt.to_tokens(e)
                }
                pgt_query::protobuf::node::Node::VariableShowStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateTableSpaceStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DropTableSpaceStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::Float(f) => f.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterTableMoveAllStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateExtensionStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CommentStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterExtensionStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterExtensionContentsStmt(stmt) => {
                    stmt.to_tokens(e)
                }
                pgt_query::protobuf::node::Node::ObjectWithArgs(obj) => obj.to_tokens(e),
                pgt_query::protobuf::node::Node::FunctionParameter(param) => param.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateFdwStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateRoleStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::SetOperationStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateForeignServerStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterFdwStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterForeignServerStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateForeignTableStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateUserMappingStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterUserMappingStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DropUserMappingStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ImportForeignSchemaStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreatePolicyStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterPolicyStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateAmStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateSeqStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterSeqStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::Integer(i) => i.to_tokens(e),
                pgt_query::protobuf::node::Node::DefineStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateDomainStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CollateClause(clause) => clause.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterDomainStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::Constraint(constraint) => constraint.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateOpClassStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateOpClassItem(item) => item.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateOpFamilyStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterOpFamilyStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ReplicaIdentityStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::SecLabelStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterCollationStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DeclareCursorStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ClosePortalStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::FetchStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AStar(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::ReturnStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateStatsStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::StatsElem(elem) => elem.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterRoleStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterRoleSetStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::DropRoleStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::AlterStatsStmt(stmt) => stmt.to_tokens(e),
                pgt_query::protobuf::node::Node::CreateFunctionStmt(stmt) => stmt.to_tokens(e),
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

        use pgt_query::protobuf::SetOperation;
        let is_set_operation = matches!(
            SetOperation::try_from(self.op).unwrap_or(SetOperation::SetopNone),
            SetOperation::SetopUnion | SetOperation::SetopIntersect | SetOperation::SetopExcept
        );

        if is_set_operation {
            if let Some(ref larg) = self.larg {
                larg.as_ref().to_tokens(e);
            }

            match SetOperation::try_from(self.op).unwrap() {
                SetOperation::SetopUnion => {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::UNION_KW);
                    if self.all {
                        e.space();
                        e.token(TokenKind::ALL_KW);
                    }
                    e.line(LineType::SoftOrSpace);
                }
                SetOperation::SetopIntersect => {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::INTERSECT_KW);
                    if self.all {
                        e.space();
                        e.token(TokenKind::ALL_KW);
                    }
                    e.line(LineType::SoftOrSpace);
                }
                SetOperation::SetopExcept => {
                    e.line(LineType::SoftOrSpace);
                    e.token(TokenKind::EXCEPT_KW);
                    if self.all {
                        e.space();
                        e.token(TokenKind::ALL_KW);
                    }
                    e.line(LineType::SoftOrSpace);
                }
                _ => {}
            }

            if let Some(ref rarg) = self.rarg {
                rarg.as_ref().to_tokens(e);
            }
        } else if !self.values_lists.is_empty() {
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

            if let Some(ref where_clause) = self.where_clause {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::WHERE_KW);
                e.space();
                where_clause.to_tokens(e);
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

        if let Some(ref alias) = self.alias {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            alias.to_tokens(e);
        }

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
        } else if e.is_within_group(GroupKind::CommentStmt) {
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
            AlterTableType::AtReplicaIdentity => {
                e.token(TokenKind::REPLICA_KW);
                e.space();
                e.token(TokenKind::IDENTITY_KW);
                e.space();
                if let Some(ref def) = self.def {
                    if let Some(pgt_query::protobuf::node::Node::String(s)) = &def.node {
                        e.token(TokenKind::IDENT(s.sval.clone()));
                    } else {
                        def.to_tokens(e);
                    }
                }
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

impl ToTokens for pgt_query::protobuf::MergeStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::MergeStmt, None, false);

        e.token(TokenKind::MERGE_KW);
        e.space();
        e.token(TokenKind::INTO_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::USING_KW);
        e.space();

        if let Some(ref source_relation) = self.source_relation {
            source_relation.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        if let Some(ref join_condition) = self.join_condition {
            join_condition.to_tokens(e);
        }

        for when_clause in &self.merge_when_clauses {
            e.line(LineType::SoftOrSpace);
            when_clause.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::MergeWhenClause {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::MergeWhenClause, None, false);

        e.token(TokenKind::WHEN_KW);
        e.space();

        use pgt_query::protobuf::{CmdType, MergeMatchKind};

        match MergeMatchKind::try_from(self.match_kind).unwrap() {
            MergeMatchKind::MergeWhenMatched => {
                e.token(TokenKind::MATCHED_KW);
            }
            MergeMatchKind::MergeWhenNotMatchedByTarget => {
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::MATCHED_KW);
            }
            MergeMatchKind::MergeWhenNotMatchedBySource => {
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::MATCHED_KW);
                e.space();
                e.token(TokenKind::BY_KW);
                e.space();
                e.token(TokenKind::SOURCE_KW);
            }
            _ => {}
        }

        if let Some(ref condition) = self.condition {
            e.space();
            e.token(TokenKind::AND_KW);
            e.space();
            condition.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::THEN_KW);
        e.space();

        match CmdType::try_from(self.command_type).unwrap() {
            CmdType::CmdInsert => {
                e.token(TokenKind::INSERT_KW);
                if !self.target_list.is_empty() {
                    e.space();
                    e.token(TokenKind::L_PAREN);
                    for (i, target) in self.target_list.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        target.to_tokens(e);
                    }
                    e.token(TokenKind::R_PAREN);
                }
                if !self.values.is_empty() {
                    e.space();
                    e.token(TokenKind::VALUES_KW);
                    e.space();
                    e.token(TokenKind::L_PAREN);
                    for (i, val) in self.values.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        val.to_tokens(e);
                    }
                    e.token(TokenKind::R_PAREN);
                }
            }
            CmdType::CmdUpdate => {
                e.group_start(GroupKind::UpdateStmt, None, false);
                e.token(TokenKind::UPDATE_KW);
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
                e.group_end();
            }
            CmdType::CmdDelete => {
                e.token(TokenKind::DELETE_KW);
            }
            CmdType::CmdNothing => {
                e.token(TokenKind::DO_KW);
                e.space();
                e.token(TokenKind::NOTHING_KW);
            }
            _ => {}
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::Alias {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.aliasname.clone()));
    }
}

impl ToTokens for pgt_query::protobuf::CreateSchemaStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateSchemaStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::SCHEMA_KW);

        if self.if_not_exists {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if !self.schemaname.is_empty() {
            e.space();
            e.token(TokenKind::IDENT(self.schemaname.clone()));
        }

        if let Some(ref authrole) = self.authrole {
            e.space();
            e.token(TokenKind::AUTHORIZATION_KW);
            e.space();
            authrole.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::RoleSpec {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::RoleSpecType;
        match RoleSpecType::try_from(self.roletype).unwrap() {
            RoleSpecType::RolespecCstring => {
                if !self.rolename.is_empty() {
                    e.token(TokenKind::IDENT(self.rolename.clone()));
                }
            }
            RoleSpecType::RolespecCurrentRole => {
                e.token(TokenKind::CURRENT_ROLE_KW);
            }
            RoleSpecType::RolespecCurrentUser => {
                e.token(TokenKind::CURRENT_USER_KW);
            }
            RoleSpecType::RolespecSessionUser => {
                e.token(TokenKind::SESSION_USER_KW);
            }
            RoleSpecType::RolespecPublic => {
                e.token(TokenKind::IDENT("PUBLIC".to_string()));
            }
            _ => {
                if !self.rolename.is_empty() {
                    e.token(TokenKind::IDENT(self.rolename.clone()));
                }
            }
        }
    }
}

impl ToTokens for pgt_query::protobuf::GrantStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::GrantStmt, None, false);

        if self.is_grant {
            e.token(TokenKind::GRANT_KW);
        } else {
            e.token(TokenKind::REVOKE_KW);
        }
        e.space();

        if !self.privileges.is_empty() {
            for (i, privilege) in self.privileges.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                privilege.to_tokens(e);
            }
        } else {
            e.token(TokenKind::ALL_KW);
            e.space();
            e.token(TokenKind::PRIVILEGES_KW);
        }

        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        use pgt_query::protobuf::ObjectType;
        match ObjectType::try_from(self.objtype).ok() {
            Some(ObjectType::ObjectTable) => {
                if e.is_within_group(GroupKind::AlterDefaultPrivilegesStmt) {
                    e.token(TokenKind::TABLES_KW);
                } else {
                    e.token(TokenKind::TABLE_KW);
                }
                if !self.objects.is_empty() {
                    e.space();
                }
            }
            Some(ObjectType::ObjectSchema) => {
                e.token(TokenKind::SCHEMA_KW);
                if !self.objects.is_empty() {
                    e.space();
                }
            }
            Some(ObjectType::ObjectDatabase) => {
                e.token(TokenKind::DATABASE_KW);
                if !self.objects.is_empty() {
                    e.space();
                }
            }
            Some(ObjectType::ObjectFunction) => {
                if e.is_within_group(GroupKind::AlterDefaultPrivilegesStmt) {
                    e.token(TokenKind::FUNCTIONS_KW);
                } else {
                    e.token(TokenKind::FUNCTION_KW);
                }
                if !self.objects.is_empty() {
                    e.space();
                }
            }
            Some(ObjectType::ObjectProcedure) => {
                e.token(TokenKind::PROCEDURE_KW);
                if !self.objects.is_empty() {
                    e.space();
                }
            }
            Some(ObjectType::ObjectSequence) => {
                if e.is_within_group(GroupKind::AlterDefaultPrivilegesStmt) {
                    e.token(TokenKind::SEQUENCES_KW);
                } else {
                    e.token(TokenKind::SEQUENCE_KW);
                }
                if !self.objects.is_empty() {
                    e.space();
                }
            }
            _ => {}
        }

        for (i, obj) in self.objects.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            obj.to_tokens(e);
        }

        if self.is_grant {
            e.space();
            e.token(TokenKind::TO_KW);
        } else {
            e.space();
            e.token(TokenKind::FROM_KW);
        }
        e.space();

        for (i, grantee) in self.grantees.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            grantee.to_tokens(e);
        }

        if self.grant_option && self.is_grant {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::GRANT_KW);
            e.space();
            e.token(TokenKind::OPTION_KW);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AccessPriv {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.priv_name.to_uppercase()));

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
    }
}

impl ToTokens for pgt_query::protobuf::TransactionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::TransactionStmt, None, false);

        use pgt_query::protobuf::TransactionStmtKind;
        match TransactionStmtKind::try_from(self.kind).unwrap() {
            TransactionStmtKind::TransStmtBegin => {
                e.token(TokenKind::BEGIN_KW);
            }
            TransactionStmtKind::TransStmtStart => {
                e.token(TokenKind::START_KW);
                e.space();
                e.token(TokenKind::TRANSACTION_KW);
            }
            TransactionStmtKind::TransStmtCommit => {
                e.token(TokenKind::COMMIT_KW);
            }
            TransactionStmtKind::TransStmtRollback => {
                e.token(TokenKind::ROLLBACK_KW);
            }
            TransactionStmtKind::TransStmtSavepoint => {
                e.token(TokenKind::SAVEPOINT_KW);
                if !self.savepoint_name.is_empty() {
                    e.space();
                    e.token(TokenKind::IDENT(self.savepoint_name.clone()));
                }
            }
            TransactionStmtKind::TransStmtRelease => {
                e.token(TokenKind::RELEASE_KW);
                if !self.savepoint_name.is_empty() {
                    e.space();
                    e.token(TokenKind::SAVEPOINT_KW);
                    e.space();
                    e.token(TokenKind::IDENT(self.savepoint_name.clone()));
                }
            }
            TransactionStmtKind::TransStmtRollbackTo => {
                e.token(TokenKind::ROLLBACK_KW);
                e.space();
                e.token(TokenKind::TO_KW);
                if !self.savepoint_name.is_empty() {
                    e.space();
                    e.token(TokenKind::SAVEPOINT_KW);
                    e.space();
                    e.token(TokenKind::IDENT(self.savepoint_name.clone()));
                }
            }
            TransactionStmtKind::TransStmtPrepare => {
                e.token(TokenKind::PREPARE_KW);
                e.space();
                e.token(TokenKind::TRANSACTION_KW);
                if !self.gid.is_empty() {
                    e.space();
                    e.token(TokenKind::STRING(format!("'{}'", self.gid)));
                }
            }
            TransactionStmtKind::TransStmtCommitPrepared => {
                e.token(TokenKind::COMMIT_KW);
                e.space();
                e.token(TokenKind::PREPARED_KW);
                if !self.gid.is_empty() {
                    e.space();
                    e.token(TokenKind::STRING(format!("'{}'", self.gid)));
                }
            }
            TransactionStmtKind::TransStmtRollbackPrepared => {
                e.token(TokenKind::ROLLBACK_KW);
                e.space();
                e.token(TokenKind::PREPARED_KW);
                if !self.gid.is_empty() {
                    e.space();
                    e.token(TokenKind::STRING(format!("'{}'", self.gid)));
                }
            }
            _ => {}
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::VariableSetStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::VariableSetStmt, None, false);

        use pgt_query::protobuf::VariableSetKind;
        match VariableSetKind::try_from(self.kind).unwrap() {
            VariableSetKind::VarSetValue => {
                e.token(TokenKind::SET_KW);
                e.space();

                if self.is_local {
                    e.token(TokenKind::LOCAL_KW);
                    e.space();
                }

                e.token(TokenKind::IDENT(self.name.clone()));

                if !self.args.is_empty() {
                    e.space();
                    e.token(TokenKind::TO_KW);
                    e.space();

                    for (i, arg) in self.args.iter().enumerate() {
                        if i > 0 {
                            e.token(TokenKind::COMMA);
                            e.space();
                        }
                        arg.to_tokens(e);
                    }
                }
            }
            VariableSetKind::VarSetDefault => {
                e.token(TokenKind::SET_KW);
                e.space();

                if self.is_local {
                    e.token(TokenKind::LOCAL_KW);
                    e.space();
                }

                e.token(TokenKind::IDENT(self.name.clone()));
                e.space();
                e.token(TokenKind::TO_KW);
                e.space();
                e.token(TokenKind::DEFAULT_KW);
            }
            VariableSetKind::VarSetCurrent => {
                e.token(TokenKind::SET_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
                e.space();
                e.token(TokenKind::FROM_KW);
                e.space();
                e.token(TokenKind::CURRENT_KW);
            }
            VariableSetKind::VarSetMulti => {
                e.token(TokenKind::SET_KW);
                e.space();

                if self.is_local {
                    e.token(TokenKind::LOCAL_KW);
                    e.space();
                }

                e.token(TokenKind::IDENT(self.name.clone()));
            }
            VariableSetKind::VarReset => {
                e.token(TokenKind::RESET_KW);
                e.space();

                if self.is_local {
                    e.token(TokenKind::LOCAL_KW);
                    e.space();
                }

                e.token(TokenKind::IDENT(self.name.clone()));
            }
            VariableSetKind::VarResetAll => {
                e.token(TokenKind::RESET_KW);
                e.space();
                e.token(TokenKind::ALL_KW);
            }
            _ => {}
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::IndexStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::IndexStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();

        if self.unique {
            e.token(TokenKind::UNIQUE_KW);
            e.space();
        }

        if self.concurrent {
            e.token(TokenKind::CONCURRENTLY_KW);
            e.space();
        }

        e.token(TokenKind::INDEX_KW);
        e.space();

        if self.if_not_exists {
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
            e.space();
        }

        if !self.idxname.is_empty() {
            e.token(TokenKind::IDENT(self.idxname.clone()));
            e.space();
        }

        e.token(TokenKind::ON_KW);
        e.space();

        if let Some(ref relation) = self.relation {
            relation.to_tokens(e);
        }

        if !self.index_params.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, param) in self.index_params.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                param.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
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

impl ToTokens for pgt_query::protobuf::IndexElem {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::IndexElem, None, false);

        if let Some(ref expr) = self.expr {
            expr.to_tokens(e);
        } else if !self.name.is_empty() {
            e.token(TokenKind::IDENT(self.name.clone()));
        }

        if !self.opclass.is_empty() {
            e.space();
            for (i, opclass) in self.opclass.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                opclass.to_tokens(e);
            }
        }

        use pgt_query::protobuf::SortByDir;
        if self.ordering != SortByDir::SortbyDefault as i32 {
            e.space();
            match SortByDir::try_from(self.ordering).unwrap() {
                SortByDir::SortbyAsc => e.token(TokenKind::ASC_KW),
                SortByDir::SortbyDesc => e.token(TokenKind::DESC_KW),
                _ => {}
            }
        }

        use pgt_query::protobuf::SortByNulls;
        if self.nulls_ordering != SortByNulls::SortbyNullsDefault as i32 {
            e.space();
            e.token(TokenKind::NULLS_KW);
            e.space();
            match SortByNulls::try_from(self.nulls_ordering).unwrap() {
                SortByNulls::SortbyNullsFirst => e.token(TokenKind::FIRST_KW),
                SortByNulls::SortbyNullsLast => e.token(TokenKind::LAST_KW),
                _ => {}
            }
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CopyStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CopyStmt, None, false);

        e.token(TokenKind::COPY_KW);
        e.space();

        if self.is_from {
            if !self.filename.is_empty() {
                e.token(TokenKind::STRING(format!("'{}'", self.filename)));
                e.space();
                e.token(TokenKind::TO_KW);
                e.space();
            }

            if let Some(ref relation) = self.relation {
                relation.to_tokens(e);
            }
        } else {
            if let Some(ref relation) = self.relation {
                relation.to_tokens(e);
            }

            if !self.attlist.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                for (i, attr) in self.attlist.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    attr.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }

            if let Some(ref query) = self.query {
                e.space();
                e.token(TokenKind::L_PAREN);
                query.to_tokens(e);
                e.token(TokenKind::R_PAREN);
            }

            e.space();
            e.token(TokenKind::TO_KW);
            e.space();

            if !self.filename.is_empty() {
                e.token(TokenKind::STRING(format!("'{}'", self.filename)));
            } else {
                e.token(TokenKind::STDOUT_KW);
            }
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();

            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.space();
                }
                option.to_tokens(e);
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

impl ToTokens for pgt_query::protobuf::DefElem {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DefElem, None, false);

        if self.defname == "format" {
            if let Some(ref arg) = self.arg {
                if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                    e.token(TokenKind::IDENT(s.sval.to_uppercase()));
                }
            }
        } else if self.defname == "header" {
            e.token(TokenKind::HEADER_KW);
        } else if self.defname == "delimiter" {
            e.token(TokenKind::DELIMITER_KW);
            if let Some(ref arg) = self.arg {
                e.space();
                arg.to_tokens(e);
            }
        } else if self.defname == "quote" {
            e.token(TokenKind::QUOTE_KW);
            if let Some(ref arg) = self.arg {
                e.space();
                arg.to_tokens(e);
            }
        } else if self.defname == "escape" {
            e.token(TokenKind::ESCAPE_KW);
            if let Some(ref arg) = self.arg {
                e.space();
                arg.to_tokens(e);
            }
        } else if self.defname == "null" {
            e.token(TokenKind::NULL_KW);
            if let Some(ref arg) = self.arg {
                e.space();
                arg.to_tokens(e);
            }
        } else if self.defname == "encoding" {
            e.token(TokenKind::ENCODING_KW);
            if let Some(ref arg) = self.arg {
                e.space();
                arg.to_tokens(e);
            }
        } else {
            if e.is_within_group(GroupKind::AlterTableSpaceOptionsStmt) {
                e.token(TokenKind::IDENT(self.defname.clone()));
                if let Some(ref arg) = self.arg {
                    e.space();
                    e.token(TokenKind::IDENT("=".to_string()));
                    e.space();
                    arg.to_tokens(e);
                }
            } else if e.is_within_group(GroupKind::AlterExtensionStmt)
                && self.defname == "new_version"
            {
                e.token(TokenKind::UPDATE_KW);
                if let Some(ref arg) = self.arg {
                    e.space();
                    e.token(TokenKind::TO_KW);
                    e.space();
                    if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                        e.token(TokenKind::STRING(format!("'{}'", s.sval)));
                    } else {
                        arg.to_tokens(e);
                    }
                }
            } else if e.is_within_group(GroupKind::CreateRoleStmt)
                || e.is_within_group(GroupKind::AlterRoleStmt)
            {
                if self.defname == "canlogin" {
                    e.token(TokenKind::IDENT("LOGIN".to_string()));
                } else if self.defname == "password" {
                    e.token(TokenKind::PASSWORD_KW);
                    if let Some(ref arg) = self.arg {
                        e.space();
                        if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                            e.token(TokenKind::STRING(format!("'{}'", s.sval)));
                        } else {
                            arg.to_tokens(e);
                        }
                    }
                } else if self.defname == "superuser" {
                    e.token(TokenKind::IDENT("SUPERUSER".to_string()));
                } else if self.defname == "createdb" {
                    e.token(TokenKind::IDENT("CREATEDB".to_string()));
                } else if self.defname == "createrole" {
                    e.token(TokenKind::IDENT("CREATEROLE".to_string()));
                } else if self.defname == "inherit" {
                    e.token(TokenKind::INHERIT_KW);
                } else if self.defname == "replication" {
                    e.token(TokenKind::IDENT("REPLICATION".to_string()));
                } else {
                    e.token(TokenKind::IDENT(self.defname.to_uppercase()));
                    if let Some(ref arg) = self.arg {
                        e.space();
                        arg.to_tokens(e);
                    }
                }
            } else if e.is_within_group(GroupKind::AlterSeqStmt) {
                if self.defname == "restart" {
                    e.token(TokenKind::RESTART_KW);
                    if let Some(ref arg) = self.arg {
                        e.space();
                        e.token(TokenKind::WITH_KW);
                        e.space();
                        arg.to_tokens(e);
                    }
                } else {
                    e.token(TokenKind::IDENT(self.defname.to_uppercase()));
                    if let Some(ref arg) = self.arg {
                        e.space();
                        arg.to_tokens(e);
                    }
                }
            } else if e.is_within_group(GroupKind::CreateForeignTableStmt)
                || e.is_within_group(GroupKind::CreateForeignServerStmt)
                || e.is_within_group(GroupKind::AlterForeignServerStmt)
                || e.is_within_group(GroupKind::CreateUserMappingStmt)
                || e.is_within_group(GroupKind::AlterUserMappingStmt)
                || e.is_within_group(GroupKind::AlterFdwStmt)
            {
                e.token(TokenKind::IDENT(self.defname.clone()));
                if let Some(ref arg) = self.arg {
                    e.space();
                    if let Some(pgt_query::protobuf::node::Node::String(s)) = &arg.node {
                        e.token(TokenKind::STRING(format!("'{}'", s.sval)));
                    } else {
                        arg.to_tokens(e);
                    }
                }
            } else if e.is_within_group(GroupKind::CreateFunctionStmt) {
                if self.defname == "as" {
                    e.token(TokenKind::AS_KW);
                } else if self.defname == "language" {
                    e.token(TokenKind::LANGUAGE_KW);
                } else {
                    e.token(TokenKind::IDENT(self.defname.to_uppercase()));
                }
                if let Some(ref arg) = self.arg {
                    e.space();
                    arg.to_tokens(e);
                }
            } else {
                e.token(TokenKind::IDENT(self.defname.to_uppercase()));
                if let Some(ref arg) = self.arg {
                    e.space();
                    arg.to_tokens(e);
                }
            }
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::Boolean {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::Boolean, None, false);

        if self.boolval {
            e.token(TokenKind::TRUE_KW);
        } else {
            e.token(TokenKind::FALSE_KW);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::GrantRoleStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::GrantRoleStmt, None, false);

        if self.is_grant {
            e.token(TokenKind::GRANT_KW);
        } else {
            e.token(TokenKind::REVOKE_KW);
        }
        e.space();

        for (i, role) in self.granted_roles.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            role.to_tokens(e);
        }

        e.space();
        if self.is_grant {
            e.token(TokenKind::TO_KW);
        } else {
            e.token(TokenKind::FROM_KW);
        }
        e.space();

        for (i, grantee) in self.grantee_roles.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            grantee.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterDefaultPrivilegesStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterDefaultPrivilegesStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::DEFAULT_KW);
        e.space();
        e.token(TokenKind::PRIVILEGES_KW);

        if let Some(ref action) = self.action {
            e.space();
            action.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::VariableShowStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::VariableShowStmt, None, false);

        e.token(TokenKind::SHOW_KW);
        e.space();
        e.token(TokenKind::IDENT(self.name.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateTableSpaceStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateTableSpaceStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.tablespacename.clone()));

        if let Some(ref owner) = self.owner {
            e.space();
            e.token(TokenKind::OWNER_KW);
            e.space();
            owner.to_tokens(e);
        }

        if !self.location.is_empty() {
            e.space();
            e.token(TokenKind::LOCATION_KW);
            e.space();
            e.token(TokenKind::STRING(format!("'{}'", self.location)));
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DropTableSpaceStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DropTableSpaceStmt, None, false);

        e.token(TokenKind::DROP_KW);
        e.space();
        e.token(TokenKind::TABLESPACE_KW);

        if self.missing_ok {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        e.token(TokenKind::IDENT(self.tablespacename.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterTableSpaceOptionsStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterTableSpaceOptionsStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.tablespacename.clone()));

        if self.is_reset {
            e.space();
            e.token(TokenKind::RESET_KW);
        } else {
            e.space();
            e.token(TokenKind::SET_KW);
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::Float {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.fval.clone()));
    }
}

impl ToTokens for pgt_query::protobuf::AlterTableMoveAllStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterTableMoveAllStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();

        match self.objtype {
            x if x == pgt_query::protobuf::ObjectType::ObjectTable as i32 => {
                e.token(TokenKind::TABLE_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectIndex as i32 => {
                e.token(TokenKind::INDEX_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectMatview as i32 => {
                e.token(TokenKind::MATERIALIZED_KW);
                e.space();
                e.token(TokenKind::VIEW_KW);
            }
            _ => {}
        }

        e.space();
        e.token(TokenKind::ALL_KW);
        e.space();
        e.token(TokenKind::IN_KW);
        e.space();
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.orig_tablespacename.clone()));

        if !self.roles.is_empty() {
            e.space();
            e.token(TokenKind::OWNED_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            e.space();
            for (i, role) in self.roles.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                role.to_tokens(e);
            }
        }

        e.space();
        e.token(TokenKind::SET_KW);
        e.space();
        e.token(TokenKind::TABLESPACE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.new_tablespacename.clone()));

        if self.nowait {
            e.space();
            e.token(TokenKind::NOWAIT_KW);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateExtensionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateExtensionStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::EXTENSION_KW);

        if self.if_not_exists {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        e.token(TokenKind::IDENT(self.extname.clone()));

        if !self.options.is_empty() {
            for option in &self.options {
                e.space();
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CommentStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CommentStmt, None, false);

        e.token(TokenKind::COMMENT_KW);
        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        match self.objtype {
            x if x == pgt_query::protobuf::ObjectType::ObjectTable as i32 => {
                e.token(TokenKind::TABLE_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectColumn as i32 => {
                e.token(TokenKind::COLUMN_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectFunction as i32 => {
                e.token(TokenKind::FUNCTION_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectIndex as i32 => {
                e.token(TokenKind::INDEX_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectView as i32 => {
                e.token(TokenKind::VIEW_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectSequence as i32 => {
                e.token(TokenKind::SEQUENCE_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectSchema as i32 => {
                e.token(TokenKind::SCHEMA_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectDatabase as i32 => {
                e.token(TokenKind::DATABASE_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectRole as i32 => {
                e.token(TokenKind::ROLE_KW);
            }
            x if x == pgt_query::protobuf::ObjectType::ObjectType as i32 => {
                e.token(TokenKind::TYPE_KW);
            }
            _ => {}
        }

        e.space();
        if let Some(ref object) = self.object {
            object.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::IS_KW);
        e.space();

        if self.comment.is_empty() {
            e.token(TokenKind::NULL_KW);
        } else {
            e.token(TokenKind::STRING(format!("'{}'", self.comment)));
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterExtensionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterExtensionStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::EXTENSION_KW);
        e.space();
        e.token(TokenKind::IDENT(self.extname.clone()));

        if !self.options.is_empty() {
            for option in &self.options {
                e.space();
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterExtensionContentsStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterExtensionContentsStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::EXTENSION_KW);
        e.space();
        e.token(TokenKind::IDENT(self.extname.clone()));
        e.space();

        if self.action > 0 {
            e.token(TokenKind::ADD_KW);
        } else {
            e.token(TokenKind::DROP_KW);
        }
        e.space();

        use pgt_query::protobuf::ObjectType;
        match ObjectType::try_from(self.objtype).ok() {
            Some(ObjectType::ObjectFunction) => {
                e.token(TokenKind::FUNCTION_KW);
            }
            Some(ObjectType::ObjectTable) => {
                e.token(TokenKind::TABLE_KW);
            }
            Some(ObjectType::ObjectType) => {
                e.token(TokenKind::TYPE_KW);
            }
            Some(ObjectType::ObjectOperator) => {
                e.token(TokenKind::OPERATOR_KW);
            }
            _ => {}
        }

        if let Some(ref object) = self.object {
            e.space();
            object.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ObjectWithArgs {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ObjectWithArgs, None, false);

        for (i, name) in self.objname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        if !self.objargs.is_empty() || !self.objfuncargs.is_empty() {
            e.token(TokenKind::L_PAREN);

            let args_to_print = if !self.objfuncargs.is_empty() {
                &self.objfuncargs
            } else {
                &self.objargs
            };

            for (i, arg) in args_to_print.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                arg.to_tokens(e);
            }

            e.token(TokenKind::R_PAREN);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::FunctionParameter {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::FunctionParameter, None, false);

        if !self.name.is_empty() {
            e.token(TokenKind::IDENT(self.name.clone()));
            e.space();
        }

        if let Some(ref arg_type) = self.arg_type {
            arg_type.to_tokens(e);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateFdwStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateFdwStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::FOREIGN_KW);
        e.space();
        e.token(TokenKind::DATA_KW);
        e.space();
        e.token(TokenKind::WRAPPER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.fdwname.clone()));

        if !self.func_options.is_empty() {
            e.space();
            e.token(TokenKind::HANDLER_KW);
            e.space();
            for (i, option) in self.func_options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateRoleStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateRoleStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();

        use pgt_query::protobuf::RoleStmtType;
        match RoleStmtType::try_from(self.stmt_type).unwrap() {
            RoleStmtType::RolestmtRole => e.token(TokenKind::ROLE_KW),
            RoleStmtType::RolestmtUser => e.token(TokenKind::USER_KW),
            RoleStmtType::RolestmtGroup => e.token(TokenKind::GROUP_KW),
            _ => e.token(TokenKind::ROLE_KW),
        }

        e.space();
        e.token(TokenKind::IDENT(self.role.clone()));

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::WITH_KW);
            for option in &self.options {
                e.space();
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::SetOperationStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::SetOperationStmt, None, false);

        if let Some(ref larg) = self.larg {
            larg.to_tokens(e);
        }

        use pgt_query::protobuf::SetOperation;
        match SetOperation::try_from(self.op).unwrap() {
            SetOperation::SetopUnion => {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::UNION_KW);
                if self.all {
                    e.space();
                    e.token(TokenKind::ALL_KW);
                }
                e.line(LineType::SoftOrSpace);
            }
            SetOperation::SetopIntersect => {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::INTERSECT_KW);
                if self.all {
                    e.space();
                    e.token(TokenKind::ALL_KW);
                }
                e.line(LineType::SoftOrSpace);
            }
            SetOperation::SetopExcept => {
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::EXCEPT_KW);
                if self.all {
                    e.space();
                    e.token(TokenKind::ALL_KW);
                }
                e.line(LineType::SoftOrSpace);
            }
            _ => {}
        }

        if let Some(ref rarg) = self.rarg {
            rarg.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateForeignServerStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateForeignServerStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::SERVER_KW);

        if self.if_not_exists {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        e.token(TokenKind::IDENT(self.servername.clone()));

        if !self.servertype.is_empty() {
            e.space();
            e.token(TokenKind::TYPE_KW);
            e.space();
            e.token(TokenKind::STRING(format!("'{}'", self.servertype)));
        }

        if !self.version.is_empty() {
            e.space();
            e.token(TokenKind::VERSION_KW);
            e.space();
            e.token(TokenKind::STRING(format!("'{}'", self.version)));
        }

        e.space();
        e.token(TokenKind::FOREIGN_KW);
        e.space();
        e.token(TokenKind::DATA_KW);
        e.space();
        e.token(TokenKind::WRAPPER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.fdwname.clone()));

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterFdwStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterFdwStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::FOREIGN_KW);
        e.space();
        e.token(TokenKind::DATA_KW);
        e.space();
        e.token(TokenKind::WRAPPER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.fdwname.clone()));

        if !self.func_options.is_empty() {
            e.space();
            e.token(TokenKind::HANDLER_KW);
            e.space();
            for (i, option) in self.func_options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterForeignServerStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterForeignServerStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.servername.clone()));

        if self.has_version {
            e.space();
            e.token(TokenKind::VERSION_KW);
            e.space();
            if !self.version.is_empty() {
                e.token(TokenKind::STRING(format!("'{}'", self.version)));
            } else {
                e.token(TokenKind::NULL_KW);
            }
        }

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateForeignTableStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateForeignTableStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::FOREIGN_KW);
        e.space();
        e.token(TokenKind::TABLE_KW);
        e.space();

        if let Some(ref base_stmt) = self.base_stmt {
            if let Some(ref relation) = base_stmt.relation {
                relation.to_tokens(e);
            }

            if !base_stmt.table_elts.is_empty() {
                e.space();
                e.token(TokenKind::L_PAREN);
                e.indent_start();
                e.line(LineType::SoftOrSpace);
                for (i, elt) in base_stmt.table_elts.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.line(LineType::SoftOrSpace);
                    }
                    elt.to_tokens(e);
                }
                e.indent_end();
                e.line(LineType::SoftOrSpace);
                e.token(TokenKind::R_PAREN);
            }
        }

        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.servername.clone()));

        if !self.options.is_empty() {
            e.line(LineType::SoftOrSpace);
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateUserMappingStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateUserMappingStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::USER_KW);
        e.space();
        e.token(TokenKind::MAPPING_KW);

        if self.if_not_exists {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();

        if let Some(ref user) = self.user {
            user.to_tokens(e);
        } else {
            e.token(TokenKind::CURRENT_USER_KW);
        }

        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.servername.clone()));

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterUserMappingStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterUserMappingStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::USER_KW);
        e.space();
        e.token(TokenKind::MAPPING_KW);
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();

        if let Some(ref user) = self.user {
            user.to_tokens(e);
        } else {
            e.token(TokenKind::CURRENT_USER_KW);
        }

        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.servername.clone()));

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DropUserMappingStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DropUserMappingStmt, None, false);

        e.token(TokenKind::DROP_KW);
        e.space();
        e.token(TokenKind::USER_KW);
        e.space();
        e.token(TokenKind::MAPPING_KW);

        if self.missing_ok {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();

        if let Some(ref user) = self.user {
            user.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.servername.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ImportForeignSchemaStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ImportForeignSchemaStmt, None, false);

        e.token(TokenKind::IMPORT_KW);
        e.space();
        e.token(TokenKind::FOREIGN_KW);
        e.space();
        e.token(TokenKind::SCHEMA_KW);
        e.space();
        e.token(TokenKind::IDENT(self.remote_schema.clone()));

        use pgt_query::protobuf::ImportForeignSchemaType;
        match ImportForeignSchemaType::try_from(self.list_type).unwrap() {
            ImportForeignSchemaType::FdwImportSchemaLimitTo => {
                e.space();
                e.token(TokenKind::LIMIT_KW);
                e.space();
                e.token(TokenKind::TO_KW);
                e.space();
                e.token(TokenKind::L_PAREN);
                for (i, table) in self.table_list.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    table.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }
            ImportForeignSchemaType::FdwImportSchemaExcept => {
                e.space();
                e.token(TokenKind::EXCEPT_KW);
                e.space();
                e.token(TokenKind::L_PAREN);
                for (i, table) in self.table_list.iter().enumerate() {
                    if i > 0 {
                        e.token(TokenKind::COMMA);
                        e.space();
                    }
                    table.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }
            _ => {}
        }

        e.space();
        e.token(TokenKind::FROM_KW);
        e.space();
        e.token(TokenKind::SERVER_KW);
        e.space();
        e.token(TokenKind::IDENT(self.server_name.clone()));

        e.space();
        e.token(TokenKind::INTO_KW);
        e.space();
        e.token(TokenKind::IDENT(self.local_schema.clone()));

        if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::OPTIONS_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                option.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreatePolicyStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreatePolicyStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::POLICY_KW);
        e.space();
        e.token(TokenKind::IDENT(self.policy_name.clone()));

        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        if let Some(ref table) = self.table {
            table.to_tokens(e);
        }

        if !self.permissive {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            e.token(TokenKind::IDENT("RESTRICTIVE".to_string()));
        }

        if !self.cmd_name.is_empty() && self.cmd_name != "all" {
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();
            match self.cmd_name.as_str() {
                "select" => e.token(TokenKind::SELECT_KW),
                "insert" => e.token(TokenKind::INSERT_KW),
                "update" => e.token(TokenKind::UPDATE_KW),
                "delete" => e.token(TokenKind::DELETE_KW),
                _ => e.token(TokenKind::IDENT(self.cmd_name.clone())),
            }
        } else if self.cmd_name == "all" {
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();
            e.token(TokenKind::ALL_KW);
        }

        if !self.roles.is_empty() {
            e.space();
            e.token(TokenKind::TO_KW);
            e.space();
            for (i, role) in self.roles.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                role.to_tokens(e);
            }
        }

        if let Some(ref qual) = self.qual {
            e.space();
            e.token(TokenKind::USING_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            qual.to_tokens(e);
            e.token(TokenKind::R_PAREN);
        }

        if let Some(ref with_check) = self.with_check {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::CHECK_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            with_check.to_tokens(e);
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterPolicyStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterPolicyStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::POLICY_KW);
        e.space();
        e.token(TokenKind::IDENT(self.policy_name.clone()));

        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        if let Some(ref table) = self.table {
            table.to_tokens(e);
        }

        if !self.roles.is_empty() {
            e.space();
            e.token(TokenKind::TO_KW);
            e.space();
            for (i, role) in self.roles.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                role.to_tokens(e);
            }
        }

        if let Some(ref qual) = self.qual {
            e.space();
            e.token(TokenKind::USING_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            qual.to_tokens(e);
            e.token(TokenKind::R_PAREN);
        }

        if let Some(ref with_check) = self.with_check {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            e.token(TokenKind::CHECK_KW);
            e.space();
            e.token(TokenKind::L_PAREN);
            with_check.to_tokens(e);
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateAmStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateAmStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::ACCESS_KW);
        e.space();
        e.token(TokenKind::METHOD_KW);
        e.space();
        e.token(TokenKind::IDENT(self.amname.clone()));

        e.space();
        e.token(TokenKind::TYPE_KW);
        e.space();
        match self.amtype.as_str() {
            "t" => e.token(TokenKind::TABLE_KW),
            "i" => e.token(TokenKind::INDEX_KW),
            _ => e.token(TokenKind::IDENT(self.amtype.clone())),
        }

        e.space();
        e.token(TokenKind::HANDLER_KW);
        e.space();
        for (i, handler) in self.handler_name.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            handler.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateSeqStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateSeqStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::SEQUENCE_KW);

        if self.if_not_exists {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if let Some(ref sequence) = self.sequence {
            e.space();
            sequence.to_tokens(e);
        }

        if !self.options.is_empty() {
            e.space();
            for option in &self.options {
                option.to_tokens(e);
                e.space();
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterSeqStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterSeqStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::SEQUENCE_KW);

        if self.missing_ok {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if let Some(ref sequence) = self.sequence {
            e.space();
            sequence.to_tokens(e);
        }

        if !self.options.is_empty() {
            e.space();
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.space();
                }
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::Integer {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT(self.ival.to_string()));
    }
}

impl ToTokens for pgt_query::protobuf::DefineStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DefineStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();

        use pgt_query::protobuf::ObjectType;
        match ObjectType::try_from(self.kind).unwrap() {
            ObjectType::ObjectAggregate => {
                e.token(TokenKind::AGGREGATE_KW);
            }
            ObjectType::ObjectOperator => {
                e.token(TokenKind::OPERATOR_KW);
            }
            ObjectType::ObjectType => {
                e.token(TokenKind::TYPE_KW);
            }
            ObjectType::ObjectCollation => {
                e.token(TokenKind::COLLATION_KW);
            }
            _ => todo!(),
        }

        if !self.defnames.is_empty() {
            e.space();
            for (i, name) in self.defnames.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        if !self.args.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, arg) in self.args.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                arg.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if !self.definition.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, def) in self.definition.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                def.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateDomainStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateDomainStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::DOMAIN_KW);
        e.space();

        for (i, name) in self.domainname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        if let Some(ref type_name) = self.type_name {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            type_name.to_tokens(e);
        }

        if let Some(ref coll_clause) = self.coll_clause {
            e.space();
            coll_clause.to_tokens(e);
        }

        for constraint in &self.constraints {
            e.space();
            constraint.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CollateClause {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::COLLATE_KW);
        e.space();

        for (i, name) in self.collname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }
    }
}

impl ToTokens for pgt_query::protobuf::AlterDomainStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterDomainStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::DOMAIN_KW);

        if self.missing_ok && self.subtype == "T" {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        e.space();
        for (i, name) in self.type_name.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        match self.subtype.as_str() {
            "T" => {
                e.space();
                e.token(TokenKind::TYPE_KW);
            }
            "N" => {
                e.space();
                if let Some(ref def) = self.def {
                    if let Some(pgt_query::protobuf::node::Node::String(s)) = &def.node {
                        if s.sval == "NOT NULL" {
                            e.token(TokenKind::SET_KW);
                            e.space();
                            e.token(TokenKind::NOT_KW);
                            e.space();
                            e.token(TokenKind::NULL_KW);
                        } else if s.sval == "NULL" {
                            e.token(TokenKind::DROP_KW);
                            e.space();
                            e.token(TokenKind::NOT_KW);
                            e.space();
                            e.token(TokenKind::NULL_KW);
                        }
                    }
                }
            }
            "O" => {
                e.space();
                e.token(TokenKind::OWNER_KW);
                e.space();
                e.token(TokenKind::TO_KW);
                e.space();
                if let Some(ref def) = self.def {
                    def.to_tokens(e);
                }
            }
            "R" => {
                e.space();
                e.token(TokenKind::RENAME_KW);
                e.space();
                e.token(TokenKind::TO_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
            "S" => {
                e.space();
                e.token(TokenKind::SET_KW);
                e.space();
                e.token(TokenKind::SCHEMA_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
            "A" => {
                e.space();
                e.token(TokenKind::ADD_KW);
                e.space();
                if let Some(ref def) = self.def {
                    def.to_tokens(e);
                }
            }
            "D" | "X" => {
                e.space();
                e.token(TokenKind::DROP_KW);
                e.space();
                e.token(TokenKind::CONSTRAINT_KW);
                if self.missing_ok {
                    e.space();
                    e.token(TokenKind::IF_KW);
                    e.space();
                    e.token(TokenKind::EXISTS_KW);
                }
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
            "V" => {
                e.space();
                e.token(TokenKind::VALIDATE_KW);
                e.space();
                e.token(TokenKind::CONSTRAINT_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
            _ => panic!("Unknown ALTER DOMAIN subtype: {}", self.subtype),
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::Constraint {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::Constraint, None, false);

        if !self.conname.is_empty() {
            e.token(TokenKind::CONSTRAINT_KW);
            e.space();
            e.token(TokenKind::IDENT(self.conname.clone()));
            e.space();
        }

        use pgt_query::protobuf::ConstrType;
        match ConstrType::try_from(self.contype).unwrap() {
            ConstrType::ConstrCheck => {
                e.token(TokenKind::CHECK_KW);
                e.space();
                e.token(TokenKind::L_PAREN);
                if let Some(ref raw_expr) = self.raw_expr {
                    raw_expr.to_tokens(e);
                }
                e.token(TokenKind::R_PAREN);
            }
            ConstrType::ConstrPrimary => {
                e.token(TokenKind::PRIMARY_KW);
                e.space();
                e.token(TokenKind::KEY_KW);
            }
            ConstrType::ConstrUnique => {
                e.token(TokenKind::UNIQUE_KW);
            }
            ConstrType::ConstrForeign => {
                e.token(TokenKind::FOREIGN_KW);
                e.space();
                e.token(TokenKind::KEY_KW);
            }
            ConstrType::ConstrNotnull => {
                e.token(TokenKind::NOT_KW);
                e.space();
                e.token(TokenKind::NULL_KW);
            }
            ConstrType::ConstrDefault => {
                e.token(TokenKind::DEFAULT_KW);
                e.space();
                if let Some(ref raw_expr) = self.raw_expr {
                    raw_expr.to_tokens(e);
                }
            }
            _ => todo!(),
        }

        if self.is_no_inherit {
            e.space();
            e.token(TokenKind::NO_KW);
            e.space();
            e.token(TokenKind::INHERIT_KW);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateOpClassStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateOpClassStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::OPERATOR_KW);
        e.space();
        e.token(TokenKind::CLASS_KW);
        e.space();

        for (i, name) in self.opclassname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        if self.is_default {
            e.space();
            e.token(TokenKind::DEFAULT_KW);
        }

        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        e.token(TokenKind::TYPE_KW);
        e.space();

        if let Some(ref datatype) = self.datatype {
            datatype.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        e.token(TokenKind::IDENT(self.amname.clone()));

        if !self.opfamilyname.is_empty() {
            e.space();
            e.token(TokenKind::FAMILY_KW);
            e.space();
            for (i, name) in self.opfamilyname.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        if !self.items.is_empty() {
            e.space();
            e.token(TokenKind::AS_KW);
            e.space();
            for (i, item) in self.items.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                item.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateOpClassItem {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateOpClassItem, None, false);

        match self.itemtype {
            1 => {
                e.token(TokenKind::OPERATOR_KW);
                e.space();
                e.token(TokenKind::IDENT(self.number.to_string()));
                e.space();
                if let Some(ref name) = self.name {
                    name.to_tokens(e);
                }
            }
            2 => {
                e.token(TokenKind::FUNCTION_KW);
                e.space();
                e.token(TokenKind::IDENT(self.number.to_string()));
                e.space();
                if let Some(ref name) = self.name {
                    name.to_tokens(e);
                }
            }
            3 => {
                e.token(TokenKind::STORAGE_KW);
                e.space();
                if let Some(ref storedtype) = self.storedtype {
                    storedtype.to_tokens(e);
                }
            }
            _ => todo!(),
        }

        if !self.order_family.is_empty() {
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();
            e.token(TokenKind::ORDER_KW);
            e.space();
            e.token(TokenKind::BY_KW);
            e.space();
            for (i, name) in self.order_family.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateOpFamilyStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateOpFamilyStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::OPERATOR_KW);
        e.space();
        e.token(TokenKind::FAMILY_KW);
        e.space();

        for (i, name) in self.opfamilyname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        e.token(TokenKind::IDENT(self.amname.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterOpFamilyStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterOpFamilyStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::OPERATOR_KW);
        e.space();
        e.token(TokenKind::FAMILY_KW);
        e.space();

        for (i, name) in self.opfamilyname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::USING_KW);
        e.space();
        e.token(TokenKind::IDENT(self.amname.clone()));
        e.space();

        if self.is_drop {
            e.token(TokenKind::DROP_KW);
        } else {
            e.token(TokenKind::ADD_KW);
        }
        e.space();

        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            item.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ReplicaIdentityStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        match self.identity_type.as_str() {
            "d" => e.token(TokenKind::DEFAULT_KW),
            "n" => e.token(TokenKind::NOTHING_KW),
            "f" => e.token(TokenKind::FULL_KW),
            "i" => {
                e.token(TokenKind::USING_KW);
                e.space();
                e.token(TokenKind::INDEX_KW);
                e.space();
                e.token(TokenKind::IDENT(self.name.clone()));
            }
            _ => panic!("Unknown replica identity type: {}", self.identity_type),
        }
    }
}

impl ToTokens for pgt_query::protobuf::AlterCollationStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterCollationStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::COLLATION_KW);
        e.space();

        for (i, name) in self.collname.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::DOT);
            }
            name.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::REFRESH_KW);
        e.space();
        e.token(TokenKind::VERSION_KW);

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DeclareCursorStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DeclareCursorStmt, None, false);

        e.token(TokenKind::DECLARE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.portalname.clone()));
        e.space();
        e.token(TokenKind::CURSOR_KW);

        // TODO: Handle cursor options (options field)

        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();

        if let Some(ref query) = self.query {
            query.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::ClosePortalStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ClosePortalStmt, None, false);

        e.token(TokenKind::CLOSE_KW);
        e.space();
        e.token(TokenKind::IDENT(self.portalname.clone()));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::SecLabelStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::SecLabelStmt, None, false);

        e.token(TokenKind::SECURITY_KW);
        e.space();
        e.token(TokenKind::LABEL_KW);

        if !self.provider.is_empty() {
            e.space();
            e.token(TokenKind::FOR_KW);
            e.space();
            e.token(TokenKind::IDENT(self.provider.clone()));
        }

        e.space();
        e.token(TokenKind::ON_KW);
        e.space();

        use pgt_query::protobuf::ObjectType;
        match ObjectType::try_from(self.objtype).unwrap() {
            ObjectType::ObjectTable => e.token(TokenKind::TABLE_KW),
            ObjectType::ObjectColumn => e.token(TokenKind::COLUMN_KW),
            ObjectType::ObjectFunction => e.token(TokenKind::FUNCTION_KW),
            ObjectType::ObjectRole => e.token(TokenKind::ROLE_KW),
            ObjectType::ObjectDatabase => e.token(TokenKind::DATABASE_KW),
            ObjectType::ObjectTablespace => e.token(TokenKind::TABLESPACE_KW),
            ObjectType::ObjectSchema => e.token(TokenKind::SCHEMA_KW),
            ObjectType::ObjectType => e.token(TokenKind::TYPE_KW),
            ObjectType::ObjectDomain => e.token(TokenKind::DOMAIN_KW),
            ObjectType::ObjectSequence => e.token(TokenKind::SEQUENCE_KW),
            ObjectType::ObjectLanguage => e.token(TokenKind::LANGUAGE_KW),
            ObjectType::ObjectLargeobject => {
                e.token(TokenKind::LARGE_KW);
                e.space();
                e.token(TokenKind::OBJECT_KW);
            }
            ObjectType::ObjectProcedure => e.token(TokenKind::PROCEDURE_KW),
            ObjectType::ObjectRoutine => e.token(TokenKind::ROUTINE_KW),
            _ => todo!(),
        }

        if let Some(ref object) = self.object {
            e.space();
            object.to_tokens(e);
        }

        e.space();
        e.token(TokenKind::IS_KW);
        e.space();
        e.token(TokenKind::STRING(format!("'{}'", self.label)));

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AStar {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.token(TokenKind::IDENT("*".to_string()));
    }
}

impl ToTokens for pgt_query::protobuf::ReturnStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::ReturnStmt, None, false);

        e.token(TokenKind::RETURN_KW);

        if let Some(ref returnval) = self.returnval {
            returnval.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::FetchStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        use pgt_query::protobuf::FetchDirection;

        e.group_start(GroupKind::FetchStmt, None, false);

        if self.ismove {
            e.token(TokenKind::MOVE_KW);
        } else {
            e.token(TokenKind::FETCH_KW);
        }

        match FetchDirection::try_from(self.direction).unwrap_or(FetchDirection::Undefined) {
            FetchDirection::FetchForward => {
                if self.how_many == 0 {
                    e.token(TokenKind::ALL_KW);
                } else if self.how_many > 1 {
                    e.token(TokenKind::FORWARD_KW);
                    e.token(TokenKind::IDENT(self.how_many.to_string()));
                } else {
                    e.token(TokenKind::NEXT_KW);
                }
            }
            FetchDirection::FetchBackward => {
                if self.how_many == 0 {
                    e.token(TokenKind::BACKWARD_KW);
                    e.token(TokenKind::ALL_KW);
                } else if self.how_many > 1 {
                    e.token(TokenKind::BACKWARD_KW);
                    e.token(TokenKind::IDENT(self.how_many.to_string()));
                } else {
                    e.token(TokenKind::PRIOR_KW);
                }
            }
            FetchDirection::FetchAbsolute => {
                e.token(TokenKind::ABSOLUTE_KW);
                e.token(TokenKind::IDENT(self.how_many.to_string()));
            }
            FetchDirection::FetchRelative => {
                e.token(TokenKind::RELATIVE_KW);
                e.token(TokenKind::IDENT(self.how_many.to_string()));
            }
            _ => {}
        }

        if !self.portalname.is_empty() {
            e.token(TokenKind::FROM_KW);
            e.token(TokenKind::IDENT(self.portalname.clone()));
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateStatsStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateStatsStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();
        e.token(TokenKind::STATISTICS_KW);

        if self.if_not_exists {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::NOT_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if !self.defnames.is_empty() {
            e.space();
            for (i, name) in self.defnames.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        if !self.stat_types.is_empty() {
            e.space();
            e.token(TokenKind::L_PAREN);
            for (i, stat_type) in self.stat_types.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                stat_type.to_tokens(e);
            }
            e.token(TokenKind::R_PAREN);
        }

        if !self.exprs.is_empty() {
            e.space();
            e.token(TokenKind::ON_KW);
            e.space();
            for (i, expr) in self.exprs.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                expr.to_tokens(e);
            }
        }

        if !self.relations.is_empty() {
            e.space();
            e.token(TokenKind::FROM_KW);
            e.space();
            for (i, relation) in self.relations.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                relation.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::StatsElem {
    fn to_tokens(&self, e: &mut EventEmitter) {
        if let Some(ref expr) = self.expr {
            expr.to_tokens(e);
        } else {
            e.token(TokenKind::IDENT(self.name.clone()));
        }
    }
}

impl ToTokens for pgt_query::protobuf::AlterRoleStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterRoleStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::ROLE_KW);

        if let Some(ref role) = self.role {
            e.space();
            role.to_tokens(e);
        }

        if self.action == 1 {
            e.space();
            e.token(TokenKind::SET_KW);
        } else if self.action == -1 {
            e.space();
            e.token(TokenKind::RESET_KW);
        } else if !self.options.is_empty() {
            e.space();
            e.token(TokenKind::WITH_KW);
            e.space();
            for (i, option) in self.options.iter().enumerate() {
                if i > 0 {
                    e.space();
                }
                option.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterRoleSetStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterRoleSetStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::ROLE_KW);

        if let Some(ref role) = self.role {
            e.space();
            role.to_tokens(e);
        }

        if !self.database.is_empty() {
            e.space();
            e.token(TokenKind::IN_KW);
            e.space();
            e.token(TokenKind::DATABASE_KW);
            e.space();
            e.token(TokenKind::IDENT(self.database.clone()));
        }

        if let Some(ref setstmt) = self.setstmt {
            e.space();
            setstmt.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::DropRoleStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::DropRoleStmt, None, false);

        e.token(TokenKind::DROP_KW);
        e.space();
        e.token(TokenKind::ROLE_KW);

        if self.missing_ok {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if !self.roles.is_empty() {
            e.space();
            for (i, role) in self.roles.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::COMMA);
                    e.space();
                }
                role.to_tokens(e);
            }
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::AlterStatsStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::AlterStatsStmt, None, false);

        e.token(TokenKind::ALTER_KW);
        e.space();
        e.token(TokenKind::STATISTICS_KW);

        if self.missing_ok {
            e.space();
            e.token(TokenKind::IF_KW);
            e.space();
            e.token(TokenKind::EXISTS_KW);
        }

        if !self.defnames.is_empty() {
            e.space();
            for (i, name) in self.defnames.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        if let Some(ref target) = self.stxstattarget {
            e.space();
            e.token(TokenKind::SET_KW);
            e.space();
            e.token(TokenKind::STATISTICS_KW);
            e.space();
            target.to_tokens(e);
        }

        if e.is_top_level() {
            e.token(TokenKind::SEMICOLON);
        }

        e.group_end();
    }
}

impl ToTokens for pgt_query::protobuf::CreateFunctionStmt {
    fn to_tokens(&self, e: &mut EventEmitter) {
        e.group_start(GroupKind::CreateFunctionStmt, None, false);

        e.token(TokenKind::CREATE_KW);
        e.space();

        if self.replace {
            e.token(TokenKind::OR_KW);
            e.space();
            e.token(TokenKind::REPLACE_KW);
            e.space();
        }

        if self.is_procedure {
            e.token(TokenKind::PROCEDURE_KW);
        } else {
            e.token(TokenKind::FUNCTION_KW);
        }

        if !self.funcname.is_empty() {
            e.space();
            for (i, name) in self.funcname.iter().enumerate() {
                if i > 0 {
                    e.token(TokenKind::DOT);
                }
                name.to_tokens(e);
            }
        }

        e.space();
        e.token(TokenKind::L_PAREN);
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                e.token(TokenKind::COMMA);
                e.space();
            }
            param.to_tokens(e);
        }
        e.token(TokenKind::R_PAREN);

        if let Some(ref return_type) = self.return_type {
            e.space();
            e.token(TokenKind::RETURNS_KW);
            e.space();
            return_type.to_tokens(e);
        }

        if !self.options.is_empty() {
            e.space();
            for option in &self.options {
                option.to_tokens(e);
                e.space();
            }
        }

        if let Some(ref sql_body) = self.sql_body {
            e.space();
            sql_body.to_tokens(e);
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
