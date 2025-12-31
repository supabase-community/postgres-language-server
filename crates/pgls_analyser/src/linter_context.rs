use pgls_analyse::{GroupCategory, RuleCategory, RuleGroup, RuleMetadata};
use pgls_schema_cache::SchemaCache;

use crate::linter_rule::LinterRule;

pub struct LinterRuleContext<'a, R: LinterRule> {
    stmt: &'a pgls_query::NodeEnum,
    options: &'a R::Options,
    schema_cache: Option<&'a SchemaCache>,
    file_context: &'a AnalysedFileContext<'a>,
}

impl<'a, R> LinterRuleContext<'a, R>
where
    R: LinterRule + Sized + 'static,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        stmt: &'a pgls_query::NodeEnum,
        options: &'a R::Options,
        schema_cache: Option<&'a SchemaCache>,
        file_context: &'a AnalysedFileContext,
    ) -> Self {
        Self {
            stmt,
            options,
            schema_cache,
            file_context,
        }
    }

    /// Returns the group that belongs to the current rule
    pub fn group(&self) -> &'static str {
        <R::Group as RuleGroup>::NAME
    }

    /// Returns the category that belongs to the current rule
    pub fn category(&self) -> RuleCategory {
        <<R::Group as RuleGroup>::Category as GroupCategory>::CATEGORY
    }

    /// Returns the AST root
    pub fn stmt(&self) -> &pgls_query::NodeEnum {
        self.stmt
    }

    pub fn file_context(&self) -> &AnalysedFileContext {
        self.file_context
    }

    pub fn schema_cache(&self) -> Option<&SchemaCache> {
        self.schema_cache
    }

    /// Returns the metadata of the rule
    ///
    /// The metadata contains information about the rule, such as the name, version, language, and whether it is recommended.
    ///
    /// ## Examples
    /// ```rust,ignore
    /// declare_lint_rule! {
    ///     /// Some doc
    ///     pub(crate) Foo {
    ///         version: "0.0.0",
    ///         name: "foo",
    ///         recommended: true,
    ///     }
    /// }
    ///
    /// impl LinterRule for Foo {
    ///     const CATEGORY: RuleCategory = RuleCategory::Lint;
    ///     type State = ();
    ///     type Signals = ();
    ///     type Options = ();
    ///
    ///     fn run(ctx: &LinterRuleContext<Self>) -> Self::Signals {
    ///         assert_eq!(ctx.metadata().name, "foo");
    ///     }
    /// }
    /// ```
    pub fn metadata(&self) -> &RuleMetadata {
        &R::METADATA
    }

    /// It retrieves the options that belong to a rule, if they exist.
    ///
    /// In order to retrieve a typed data structure, you have to create a deserializable
    /// data structure and define it inside the generic type `type Options` of the [LinterRule]
    ///
    pub fn options(&self) -> &R::Options {
        self.options
    }
}

pub struct AnalysedFileContext<'a> {
    pub stmts: &'a Vec<pgls_query::NodeEnum>,
    pos: usize,
    transaction_state: TransactionState,
}

impl<'a> AnalysedFileContext<'a> {
    pub fn new(stmts: &'a Vec<pgls_query::NodeEnum>) -> Self {
        Self {
            stmts,
            pos: 0,
            transaction_state: TransactionState::default(),
        }
    }

    pub fn previous_stmts(&self) -> &[pgls_query::NodeEnum] {
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
/// - Whether an ACCESS EXCLUSIVE lock is currently being held
#[derive(Debug, Default)]
pub struct TransactionState {
    /// Whether `SET lock_timeout` has been called in this transaction
    lock_timeout_set: bool,
    /// Objects (schema, name) created in this transaction
    /// Schema names are normalized: empty string is stored as "public"
    created_objects: Vec<(String, String)>,
    /// Whether an ACCESS EXCLUSIVE lock is currently being held
    /// This is set when an ALTER TABLE is executed on an existing table
    holding_access_exclusive: bool,
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

    /// Returns true if the transaction is currently holding an ACCESS EXCLUSIVE lock
    pub fn is_holding_access_exclusive(&self) -> bool {
        self.holding_access_exclusive
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
    pub(crate) fn update_from_stmt(&mut self, stmt: &pgls_query::NodeEnum) {
        // Track SET lock_timeout
        if let pgls_query::NodeEnum::VariableSetStmt(set_stmt) = stmt {
            if set_stmt.name.eq_ignore_ascii_case("lock_timeout") {
                self.lock_timeout_set = true;
            }
        }

        // Track created objects
        match stmt {
            pgls_query::NodeEnum::CreateStmt(create_stmt) => {
                if let Some(relation) = &create_stmt.relation {
                    let schema = relation.schemaname.clone();
                    let name = relation.relname.clone();
                    self.add_created_object(schema, name);
                }
            }
            pgls_query::NodeEnum::IndexStmt(index_stmt) => {
                if !index_stmt.idxname.is_empty() {
                    let schema = index_stmt
                        .relation
                        .as_ref()
                        .map(|r| r.schemaname.clone())
                        .unwrap_or_default();
                    self.add_created_object(schema, index_stmt.idxname.clone());
                }
            }
            pgls_query::NodeEnum::CreateTableAsStmt(ctas) => {
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

        // Track ACCESS EXCLUSIVE lock acquisition
        // ALTER TABLE on an existing table acquires ACCESS EXCLUSIVE lock
        if let pgls_query::NodeEnum::AlterTableStmt(alter_stmt) = stmt {
            if let Some(relation) = &alter_stmt.relation {
                let schema = &relation.schemaname;
                let name = &relation.relname;
                // Only set the flag if altering an existing table (not one created in this transaction)
                if !self.has_created_object(schema, name) {
                    self.holding_access_exclusive = true;
                }
            }
        }
    }
}
