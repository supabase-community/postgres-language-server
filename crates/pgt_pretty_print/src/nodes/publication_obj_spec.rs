use crate::TokenKind;
use crate::emitter::{EventEmitter, GroupKind};
use pgt_query::protobuf::PublicationObjSpec;

pub(super) fn emit_publication_obj_spec(e: &mut EventEmitter, n: &PublicationObjSpec) {
    e.group_start(GroupKind::PublicationObjSpec);

    // pubobjtype: 0=Undefined, 1=TABLE, 2=TABLES_IN_SCHEMA, 3=TABLES_IN_CUR_SCHEMA, 4=CONTINUATION
    match n.pubobjtype {
        2 => {
            // TABLES IN SCHEMA schema_name
            e.token(TokenKind::IDENT("TABLES".to_string()));
            e.space();
            e.token(TokenKind::IN_KW);
            e.space();
            e.token(TokenKind::IDENT("SCHEMA".to_string()));
            if !n.name.is_empty() {
                e.space();
                e.token(TokenKind::IDENT(n.name.clone()));
            }
        }
        3 => {
            // TABLES IN CURRENT SCHEMA
            e.token(TokenKind::IDENT("TABLES".to_string()));
            e.space();
            e.token(TokenKind::IN_KW);
            e.space();
            e.token(TokenKind::IDENT("CURRENT".to_string()));
            e.space();
            e.token(TokenKind::IDENT("SCHEMA".to_string()));
        }
        1 | _ => {
            // TABLE table_name with optional columns and WHERE clause
            if let Some(ref pubtable) = n.pubtable {
                // Emit TABLE keyword for single table case
                e.token(TokenKind::TABLE_KW);
                e.space();

                if let Some(ref relation) = pubtable.relation {
                    super::emit_range_var(e, relation);
                }

                // Optional column list
                if !pubtable.columns.is_empty() {
                    e.space();
                    e.token(TokenKind::L_PAREN);
                    super::node_list::emit_comma_separated_list(
                        e,
                        &pubtable.columns,
                        super::emit_node,
                    );
                    e.token(TokenKind::R_PAREN);
                }

                // Optional WHERE clause
                if let Some(ref where_clause) = pubtable.where_clause {
                    e.space();
                    e.token(TokenKind::WHERE_KW);
                    e.space();
                    e.token(TokenKind::L_PAREN);
                    super::emit_node(where_clause, e);
                    e.token(TokenKind::R_PAREN);
                }
            }
        }
    }

    e.group_end();
}
