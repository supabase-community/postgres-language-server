pub struct AnalysedFileContext<'a> {
    pub stmts: &'a Vec<pgt_query::NodeEnum>,
    pos: usize,
    transaction_state: TransactionState,
}

impl<'a> AnalysedFileContext<'a> {
    pub fn new(stmts: &'a Vec<pgt_query::NodeEnum>) -> Self {
        Self {
            stmts,
            pos: 0,
            transaction_state: TransactionState::default(),
        }
    }

    pub fn previous_stmts(&self) -> &[pgt_query::NodeEnum] {
        &self.stmts[0..self.pos]
    }

    pub fn stmt_count(&self) -> usize {
        self.stmts.len()
    }

    /// Move to the next statement and update transaction state with the current statement
    pub fn next(&mut self) {
        if self.pos < self.stmts.len() {
            self.transaction_state
                .update_from_stmt(&self.stmts[self.pos]);
        }
        self.pos += 1;
    }

    /// Get a reference to the transaction state
    pub fn transaction_state(&self) -> &TransactionState {
        &self.transaction_state
    }
}

/// Represents the state of a transaction as we analyze statements in a file.
///
/// This tracks properties that span multiple statements, such as:
/// - Whether a lock timeout has been set
/// - Which objects have been created in this transaction
#[derive(Debug, Default)]
pub struct TransactionState {
    /// Whether `SET lock_timeout` has been called in this transaction
    lock_timeout_set: bool,
    /// Objects (schema, name) created in this transaction
    /// Schema names are normalized: empty string is stored as "public"
    created_objects: Vec<(String, String)>,
}

impl TransactionState {
    /// Returns true if a lock timeout has been set in this transaction
    pub fn has_lock_timeout(&self) -> bool {
        self.lock_timeout_set
    }

    /// Returns true if an object with the given schema and name was created in this transaction
    pub fn has_created_object(&self, schema: &str, name: &str) -> bool {
        // Normalize schema: treat empty string as "public"
        let normalized_schema = if schema.is_empty() { "public" } else { schema };

        self.created_objects
            .iter()
            .any(|(s, n)| normalized_schema.eq_ignore_ascii_case(s) && name.eq_ignore_ascii_case(n))
    }

    /// Record that an object was created, normalizing the schema name
    fn add_created_object(&mut self, schema: String, name: String) {
        // Normalize schema: store "public" instead of empty string
        let normalized_schema = if schema.is_empty() {
            "public".to_string()
        } else {
            schema
        };
        self.created_objects.push((normalized_schema, name));
    }

    /// Update transaction state based on a statement
    pub(crate) fn update_from_stmt(&mut self, stmt: &pgt_query::NodeEnum) {
        // Track SET lock_timeout
        if let pgt_query::NodeEnum::VariableSetStmt(set_stmt) = stmt {
            if set_stmt.name.eq_ignore_ascii_case("lock_timeout") {
                self.lock_timeout_set = true;
            }
        }

        // Track created objects
        match stmt {
            pgt_query::NodeEnum::CreateStmt(create_stmt) => {
                if let Some(relation) = &create_stmt.relation {
                    let schema = relation.schemaname.clone();
                    let name = relation.relname.clone();
                    self.add_created_object(schema, name);
                }
            }
            pgt_query::NodeEnum::IndexStmt(index_stmt) => {
                if !index_stmt.idxname.is_empty() {
                    let schema = index_stmt
                        .relation
                        .as_ref()
                        .map(|r| r.schemaname.clone())
                        .unwrap_or_default();
                    self.add_created_object(schema, index_stmt.idxname.clone());
                }
            }
            pgt_query::NodeEnum::CreateTableAsStmt(ctas) => {
                if let Some(into) = &ctas.into {
                    if let Some(rel) = &into.rel {
                        let schema = rel.schemaname.clone();
                        let name = rel.relname.clone();
                        self.add_created_object(schema, name);
                    }
                }
            }
            _ => {}
        }
    }
}
