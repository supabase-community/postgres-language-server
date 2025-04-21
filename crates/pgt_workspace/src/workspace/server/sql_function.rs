use std::sync::Arc;

use dashmap::DashMap;
use pgt_text_size::TextRange;

use super::statement_identifier::StatementId;

#[derive(Debug, Clone)]
pub struct SQLFunctionBody {
    pub range: TextRange,
    pub body: String,
}

pub struct SQLFunctionBodyStore {
    db: DashMap<StatementId, Option<Arc<SQLFunctionBody>>>,
}

impl SQLFunctionBodyStore {
    pub fn new() -> SQLFunctionBodyStore {
        SQLFunctionBodyStore { db: DashMap::new() }
    }

    pub fn get_function_body(
        &self,
        statement: &StatementId,
        ast: &pgt_query_ext::NodeEnum,
        content: &str,
    ) -> Option<Arc<SQLFunctionBody>> {
        // First check if we already have this statement cached
        if let Some(existing) = self.db.get(statement).map(|x| x.clone()) {
            return existing;
        }

        // If not cached, try to extract it from the AST
        let fn_body = get_sql_fn(ast, content).map(Arc::new);

        // Cache the result and return it
        self.db.insert(statement.clone(), fn_body.clone());
        fn_body
    }

    pub fn clear_statement(&self, id: &StatementId) {
        self.db.remove(id);

        if let Some(child_id) = id.get_child_id() {
            self.db.remove(&child_id);
        }
    }
}

/// Extracts SQL function body and its text range from a CreateFunctionStmt node.
/// Returns None if the function is not an SQL function or if the body can't be found.
fn get_sql_fn(ast: &pgt_query_ext::NodeEnum, content: &str) -> Option<SQLFunctionBody> {
    let create_fn = match ast {
        pgt_query_ext::NodeEnum::CreateFunctionStmt(cf) => cf,
        _ => return None,
    };

    // Extract language from function options
    let language = find_option_value(create_fn, "language")?;

    // Only process SQL functions
    if language != "sql" {
        return None;
    }

    // Extract SQL body from function options
    let sql_body = find_option_value(create_fn, "as")?;

    // Find the range of the SQL body in the content
    let start = content.find(&sql_body)?;
    let end = start + sql_body.len();

    let range = TextRange::new(start.try_into().unwrap(), end.try_into().unwrap());

    Some(SQLFunctionBody {
        range,
        body: sql_body.clone(),
    })
}

/// Helper function to find a specific option value from function options
fn find_option_value(
    create_fn: &pgt_query_ext::protobuf::CreateFunctionStmt,
    option_name: &str,
) -> Option<String> {
    create_fn
        .options
        .iter()
        .filter_map(|opt_wrapper| opt_wrapper.node.as_ref())
        .find_map(|opt| {
            if let pgt_query_ext::NodeEnum::DefElem(def_elem) = opt {
                if def_elem.defname == option_name {
                    def_elem
                        .arg
                        .iter()
                        .filter_map(|arg_wrapper| arg_wrapper.node.as_ref())
                        .find_map(|arg| {
                            if let pgt_query_ext::NodeEnum::String(s) = arg {
                                Some(s.sval.clone())
                            } else if let pgt_query_ext::NodeEnum::List(l) = arg {
                                l.items.iter().find_map(|item_wrapper| {
                                    if let Some(pgt_query_ext::NodeEnum::String(s)) =
                                        item_wrapper.node.as_ref()
                                    {
                                        Some(s.sval.clone())
                                    } else {
                                        None
                                    }
                                })
                            } else {
                                None
                            }
                        })
                } else {
                    None
                }
            } else {
                None
            }
        })
}
