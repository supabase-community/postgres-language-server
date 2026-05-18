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

    pub fn file_context(&self) -> &AnalysedFileContext<'_> {
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
///
/// Transaction boundaries (BEGIN/COMMIT/ROLLBACK) reset accumulated state
/// to avoid false positives across separate transactions in the same file.
#[derive(Debug, Default)]
pub struct TransactionState {
    /// Whether `SET lock_timeout` has been called in this transaction
    lock_timeout_set: bool,
    /// Whether `SET statement_timeout` has been called in this transaction
    statement_timeout_set: bool,
    /// Whether `SET idle_in_transaction_session_timeout` has been called in this transaction
    idle_in_transaction_timeout_set: bool,
    /// Objects (schema, name) created in this transaction
    /// Schema names are normalized: empty string is stored as "public"
    created_objects: Vec<(String, String)>,
    /// Whether an ACCESS EXCLUSIVE lock is currently being held
    /// This is set when an ALTER TABLE is executed on an existing table
    holding_access_exclusive: bool,
    /// Constraints added with NOT VALID: (schema, table, constraint_name)
    not_valid_constraints: Vec<(String, String, String)>,
    /// Tables holding ACCESS EXCLUSIVE locks in this transaction (for wide lock window detection)
    access_exclusive_tables: Vec<(String, String)>,
    /// Transaction nesting depth (0 = not in explicit transaction)
    transaction_depth: usize,
}

fn normalized_schema(schema: &str) -> &str {
    if schema.is_empty() { "public" } else { schema }
}

fn def_elem_is_enabled(def: &pgls_query::protobuf::DefElem, name: &str) -> bool {
    if !def.defname.eq_ignore_ascii_case(name) {
        return false;
    }

    match &def.arg {
        Some(arg) => match &arg.node {
            Some(pgls_query::NodeEnum::Integer(i)) => i.ival != 0,
            Some(pgls_query::NodeEnum::Boolean(b)) => b.boolval,
            _ => true,
        },
        None => true,
    }
}

pub(crate) fn is_vacuum_full(stmt: &pgls_query::protobuf::VacuumStmt) -> bool {
    stmt.options.iter().any(|opt| {
        if let Some(pgls_query::NodeEnum::DefElem(def)) = &opt.node {
            def_elem_is_enabled(def, "full")
        } else {
            false
        }
    })
}

pub(crate) fn is_reindex_concurrent(stmt: &pgls_query::protobuf::ReindexStmt) -> bool {
    stmt.params.iter().any(|param| {
        if let Some(pgls_query::NodeEnum::DefElem(def)) = &param.node {
            def_elem_is_enabled(def, "concurrently")
        } else {
            false
        }
    })
}

impl TransactionState {
    /// Returns true if a lock timeout has been set in this transaction
    pub fn has_lock_timeout(&self) -> bool {
        self.lock_timeout_set
    }

    /// Returns true if a statement timeout has been set in this transaction
    pub fn has_statement_timeout(&self) -> bool {
        self.statement_timeout_set
    }

    /// Returns true if an idle-in-transaction timeout has been set in this transaction
    pub fn has_idle_in_transaction_timeout(&self) -> bool {
        self.idle_in_transaction_timeout_set
    }

    /// Returns true if a constraint with the given name on the given table was added with NOT VALID
    pub fn has_not_valid_constraint(&self, schema: &str, table: &str, name: &str) -> bool {
        self.not_valid_constraints.iter().any(|(s, t, n)| {
            normalized_schema(schema).eq_ignore_ascii_case(s)
                && table.eq_ignore_ascii_case(t)
                && name.eq_ignore_ascii_case(n)
        })
    }

    /// Returns the tables currently holding ACCESS EXCLUSIVE locks
    pub fn access_exclusive_tables(&self) -> &[(String, String)] {
        &self.access_exclusive_tables
    }

    /// Returns true if an object with the given schema and name was created in this transaction
    pub fn has_created_object(&self, schema: &str, name: &str) -> bool {
        self.created_objects.iter().any(|(s, n)| {
            normalized_schema(schema).eq_ignore_ascii_case(s) && name.eq_ignore_ascii_case(n)
        })
    }

    /// Returns true if the transaction is currently holding an ACCESS EXCLUSIVE lock
    pub fn is_holding_access_exclusive(&self) -> bool {
        self.holding_access_exclusive
    }

    /// Returns true if the statement takes a dangerous lock on an existing object.
    ///
    /// Covers: AlterTableStmt, non-concurrent IndexStmt, DropStmt (table/index),
    /// TruncateStmt, VacuumStmt, non-concurrent ReindexStmt, RenameStmt (with relation),
    /// and non-concurrent RefreshMatViewStmt.
    pub fn is_dangerous_lock_stmt(&self, stmt: &pgls_query::NodeEnum) -> bool {
        match stmt {
            pgls_query::NodeEnum::AlterTableStmt(alter_stmt) => {
                if let Some(relation) = &alter_stmt.relation {
                    !self.has_created_object(&relation.schemaname, &relation.relname)
                } else {
                    true
                }
            }
            pgls_query::NodeEnum::IndexStmt(index_stmt) => {
                if index_stmt.concurrent {
                    return false;
                }
                if let Some(relation) = &index_stmt.relation {
                    !self.has_created_object(&relation.schemaname, &relation.relname)
                } else {
                    true
                }
            }
            pgls_query::NodeEnum::DropStmt(drop_stmt) => {
                let obj_type = drop_stmt.remove_type();
                matches!(
                    obj_type,
                    pgls_query::protobuf::ObjectType::ObjectTable
                        | pgls_query::protobuf::ObjectType::ObjectIndex
                        | pgls_query::protobuf::ObjectType::ObjectMatview
                )
            }
            pgls_query::NodeEnum::TruncateStmt(_) => true,
            pgls_query::NodeEnum::VacuumStmt(stmt) => is_vacuum_full(stmt),
            pgls_query::NodeEnum::ReindexStmt(stmt) => !is_reindex_concurrent(stmt),
            pgls_query::NodeEnum::RenameStmt(rename_stmt) => rename_stmt.relation.is_some(),
            pgls_query::NodeEnum::RefreshMatViewStmt(stmt) => !stmt.concurrent,
            _ => false,
        }
    }

    /// Record that an object was created, normalizing the schema name
    fn add_created_object(&mut self, schema: String, name: String) {
        self.created_objects
            .push((normalized_schema(&schema).to_string(), name));
    }

    /// Reset per-transaction accumulated state.
    /// Called on COMMIT/ROLLBACK to avoid false positives across transactions.
    fn reset_transaction_state(&mut self) {
        self.lock_timeout_set = false;
        self.statement_timeout_set = false;
        self.idle_in_transaction_timeout_set = false;
        self.created_objects.clear();
        self.holding_access_exclusive = false;
        self.not_valid_constraints.clear();
        self.access_exclusive_tables.clear();
    }

    /// Returns true if an ALTER TABLE subcommand takes an ACCESS EXCLUSIVE lock.
    /// Some subtypes take lighter locks and should not be tracked.
    fn is_access_exclusive_subcommand(subtype: pgls_query::protobuf::AlterTableType) -> bool {
        use pgls_query::protobuf::AlterTableType;
        !matches!(
            subtype,
            // VALIDATE CONSTRAINT takes SHARE UPDATE EXCLUSIVE
            AlterTableType::AtValidateConstraint
        )
    }

    /// Returns the existing table targeted by an ALTER TABLE command that takes ACCESS EXCLUSIVE.
    pub fn access_exclusive_table_for_alter(
        &self,
        stmt: &pgls_query::protobuf::AlterTableStmt,
    ) -> Option<(String, String)> {
        let relation = stmt.relation.as_ref()?;
        let has_access_exclusive_cmd = stmt.cmds.iter().any(|cmd| {
            if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node {
                Self::is_access_exclusive_subcommand(cmd.subtype())
            } else {
                false
            }
        });

        if !has_access_exclusive_cmd {
            return None;
        }

        let schema = normalized_schema(&relation.schemaname).to_string();
        let name = relation.relname.clone();
        if self.has_created_object(&schema, &name) {
            return None;
        }

        Some((schema, name))
    }

    fn update_timeout_flags(&mut self, set_stmt: &pgls_query::protobuf::VariableSetStmt) {
        use pgls_query::protobuf::VariableSetKind;

        let kind = VariableSetKind::try_from(set_stmt.kind).unwrap_or(VariableSetKind::Undefined);
        if kind == VariableSetKind::VarResetAll {
            self.lock_timeout_set = false;
            self.statement_timeout_set = false;
            self.idle_in_transaction_timeout_set = false;
            return;
        }

        let value_sets_timeout = matches!(
            kind,
            VariableSetKind::VarSetValue
                | VariableSetKind::VarSetCurrent
                | VariableSetKind::VarSetMulti
        );

        let flag = if set_stmt.name.eq_ignore_ascii_case("lock_timeout") {
            &mut self.lock_timeout_set
        } else if set_stmt.name.eq_ignore_ascii_case("statement_timeout") {
            &mut self.statement_timeout_set
        } else if set_stmt
            .name
            .eq_ignore_ascii_case("idle_in_transaction_session_timeout")
        {
            &mut self.idle_in_transaction_timeout_set
        } else {
            return;
        };

        *flag = value_sets_timeout;
    }

    /// Update transaction state based on a statement
    pub(crate) fn update_from_stmt(&mut self, stmt: &pgls_query::NodeEnum) {
        // Handle transaction boundaries
        if let pgls_query::NodeEnum::TransactionStmt(tx_stmt) = stmt {
            use pgls_query::protobuf::TransactionStmtKind;
            match tx_stmt.kind() {
                TransactionStmtKind::TransStmtBegin | TransactionStmtKind::TransStmtStart => {
                    self.transaction_depth += 1;
                }
                TransactionStmtKind::TransStmtCommit | TransactionStmtKind::TransStmtRollback => {
                    if self.transaction_depth > 0 {
                        self.transaction_depth -= 1;
                    }
                    // Reset accumulated state — new transaction starts fresh
                    self.reset_transaction_state();
                }
                TransactionStmtKind::TransStmtSavepoint => {
                    self.transaction_depth += 1;
                }
                TransactionStmtKind::TransStmtRelease
                | TransactionStmtKind::TransStmtRollbackTo => {
                    if self.transaction_depth > 0 {
                        self.transaction_depth -= 1;
                    }
                }
                _ => {}
            }
            return;
        }

        // Track SET/RESET timeouts
        if let pgls_query::NodeEnum::VariableSetStmt(set_stmt) = stmt {
            self.update_timeout_flags(set_stmt);
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
                if let Some(into) = &ctas.into
                    && let Some(rel) = &into.rel
                {
                    let schema = rel.schemaname.clone();
                    let name = rel.relname.clone();
                    self.add_created_object(schema, name);
                }
            }
            _ => {}
        }

        // Track NOT VALID constraints (table-qualified to avoid false positives)
        if let pgls_query::NodeEnum::AlterTableStmt(alter_stmt) = stmt {
            let (table_schema, table_name) = alter_stmt
                .relation
                .as_ref()
                .map(|r| {
                    (
                        normalized_schema(&r.schemaname).to_string(),
                        r.relname.clone(),
                    )
                })
                .unwrap_or_default();

            for cmd in &alter_stmt.cmds {
                if let Some(pgls_query::NodeEnum::AlterTableCmd(cmd)) = &cmd.node
                    && cmd.subtype() == pgls_query::protobuf::AlterTableType::AtAddConstraint
                {
                    if let Some(pgls_query::NodeEnum::Constraint(constraint)) =
                        cmd.def.as_ref().and_then(|d| d.node.as_ref())
                    {
                        if constraint.skip_validation && !constraint.conname.is_empty() {
                            self.not_valid_constraints.push((
                                table_schema.clone(),
                                table_name.clone(),
                                constraint.conname.clone(),
                            ));
                        }
                    }
                }
            }
        }

        // Track ACCESS EXCLUSIVE lock acquisition.
        if let pgls_query::NodeEnum::AlterTableStmt(alter_stmt) = stmt
            && let Some((schema, name)) = self.access_exclusive_table_for_alter(alter_stmt)
        {
            self.holding_access_exclusive = true;
            if !self
                .access_exclusive_tables
                .iter()
                .any(|(s, n)| s == &schema && n == &name)
            {
                self.access_exclusive_tables.push((schema, name));
            }
        }
    }
}
