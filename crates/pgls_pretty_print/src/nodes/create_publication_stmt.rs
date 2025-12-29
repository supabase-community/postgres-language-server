use crate::{
    TokenKind,
    emitter::{EventEmitter, GroupKind, LineType},
    nodes::node_list::emit_comma_separated_list,
};
use pgls_query::{NodeEnum, protobuf::CreatePublicationStmt};

pub(super) fn emit_create_publication_stmt(e: &mut EventEmitter, n: &CreatePublicationStmt) {
    e.group_start(GroupKind::CreatePublicationStmt);

    e.token(TokenKind::CREATE_KW);
    e.space();
    e.token(TokenKind::IDENT("PUBLICATION".to_string()));
    e.space();
    e.token(TokenKind::IDENT(n.pubname.clone()));

    if n.for_all_tables {
        e.space();
        e.token(TokenKind::FOR_KW);
        e.space();
        e.token(TokenKind::ALL_KW);
        e.space();
        e.token(TokenKind::IDENT("TABLES".to_string()));
    } else if !n.pubobjects.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::FOR_KW);
        e.space();
        // Publication objects (PublicationObjSpec) contain tables and other objects
        emit_comma_separated_list(e, &n.pubobjects, |node, e| {
            if let Some(NodeEnum::PublicationObjSpec(obj)) = &node.node {
                // PublicationObjSpec has pubobjtype:
                // 0=Undefined, 1=TABLE, 2=TABLES_IN_SCHEMA, 3=TABLES_IN_CUR_SCHEMA
                match obj.pubobjtype {
                    1 => {
                        // TABLE
                        e.token(TokenKind::TABLE_KW);
                        e.space();
                        if let Some(ref relation) = obj.pubtable {
                            if let Some(ref pubrel) = relation.relation {
                                super::emit_range_var(e, pubrel);
                            }
                            // Handle column list
                            if !relation.columns.is_empty() {
                                e.space();
                                e.token(TokenKind::L_PAREN);
                                emit_comma_separated_list(e, &relation.columns, super::emit_node);
                                e.token(TokenKind::R_PAREN);
                            }
                            // Handle WHERE clause
                            if let Some(ref where_clause) = relation.where_clause {
                                e.space();
                                e.token(TokenKind::WHERE_KW);
                                e.space();
                                e.token(TokenKind::L_PAREN);
                                super::emit_node(where_clause, e);
                                e.token(TokenKind::R_PAREN);
                            }
                        }
                    }
                    2 => {
                        // TABLES IN SCHEMA
                        e.token(TokenKind::IDENT("TABLES".to_string()));
                        e.space();
                        e.token(TokenKind::IN_KW);
                        e.space();
                        e.token(TokenKind::IDENT("SCHEMA".to_string()));
                        e.space();
                        if !obj.name.is_empty() {
                            e.token(TokenKind::IDENT(obj.name.clone()));
                        }
                    }
                    3 => {
                        // TABLES IN SCHEMA CURRENT_SCHEMA
                        e.token(TokenKind::IDENT("TABLES".to_string()));
                        e.space();
                        e.token(TokenKind::IN_KW);
                        e.space();
                        e.token(TokenKind::IDENT("SCHEMA".to_string()));
                        e.space();
                        e.token(TokenKind::IDENT("CURRENT_SCHEMA".to_string()));
                    }
                    _ => {}
                }
            }
        });
    }

    if !n.options.is_empty() {
        e.line(LineType::SoftOrSpace);
        e.token(TokenKind::WITH_KW);
        e.space();
        e.token(TokenKind::L_PAREN);
        emit_comma_separated_list(e, &n.options, super::emit_node);
        e.token(TokenKind::R_PAREN);
    }

    e.token(TokenKind::SEMICOLON);

    e.group_end();
}
