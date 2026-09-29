//! Session settings and transaction state tracked while analysing a file.
use pgls_query::{NodeEnum, protobuf};

/// Represents the state of a session as we analyze statements in a file.
///
/// This tracks properties that span multiple statements, such as:
/// - Whether a lock timeout has been set
/// - Which objects have been created in this transaction
/// - Whether an ACCESS EXCLUSIVE lock is currently being held
///
/// Transaction boundaries (BEGIN/COMMIT/ROLLBACK) reset accumulated state
/// to avoid false positives across separate transactions in the same file.
#[derive(Debug, Default, Clone)]
pub struct Session {
    /// Whether `SET lock_timeout` has been called in this transaction
    lock_timeout_set: bool,
    /// Whether `SET statement_timeout` has been called in this transaction
    statement_timeout_set: bool,
    /// Whether `SET idle_in_transaction_session_timeout` has been called in this transaction
    idle_in_transaction_timeout_set: bool,
    /// Objects (schema, name) created in this transaction
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
    /// Initial search path provided when the session was created.
    initial_search_path: Vec<String>,
    /// Current search path.
    search_path: Vec<String>,
    /// Current role set by `SET ROLE`.
    role: Option<String>,
    /// Search path at the start of the outermost explicit transaction.
    transaction_search_path: Option<Vec<String>>,
    /// Search path to restore when the transaction ends after `SET LOCAL`.
    local_search_path: Option<Vec<String>>,
    /// Role at the start of the outermost explicit transaction.
    transaction_role: Option<Option<String>>,
    /// Role to restore when the transaction ends after `SET LOCAL ROLE`.
    local_role: Option<Option<String>>,
}

/// The schema of an object name. Unqualified names are created in the first schema of the
/// search path.
fn normalized_schema<'a>(schema: &'a str, search_path: &'a [String]) -> &'a str {
    if schema.is_empty() {
        search_path
            .iter()
            .map(String::as_str)
            .find(|schema| *schema != "$user")
            .unwrap_or("public")
    } else {
        schema
    }
}

fn def_elem_is_enabled(def: &protobuf::DefElem, name: &str) -> bool {
    if !def.defname.eq_ignore_ascii_case(name) {
        return false;
    }
    match &def.arg {
        Some(arg) => match &arg.node {
            Some(NodeEnum::Integer(i)) => i.ival != 0,
            Some(NodeEnum::Boolean(b)) => b.boolval,
            _ => true,
        },
        None => true,
    }
}

pub fn is_vacuum_full(stmt: &protobuf::VacuumStmt) -> bool {
    stmt.options.iter().any(
        |opt| matches!(&opt.node, Some(NodeEnum::DefElem(def)) if def_elem_is_enabled(def, "full")),
    )
}

pub fn is_reindex_concurrent(stmt: &protobuf::ReindexStmt) -> bool {
    stmt.params.iter().any(|param| matches!(&param.node, Some(NodeEnum::DefElem(def)) if def_elem_is_enabled(def, "concurrently")))
}

impl Session {
    /// Create a session with its initial search path.
    pub fn new(search_path: Vec<String>) -> Self {
        Self {
            initial_search_path: search_path.clone(),
            search_path,
            ..Self::default()
        }
    }

    /// Returns the current search path.
    pub fn search_path(&self) -> &[String] {
        &self.search_path
    }
    /// Returns the current role, if one has been set.
    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }
    /// Returns the transaction nesting depth (0 = not in an explicit transaction).
    pub fn transaction_depth(&self) -> usize {
        self.transaction_depth
    }

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
            normalized_schema(schema, &self.search_path).eq_ignore_ascii_case(s)
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
            normalized_schema(schema, &self.search_path).eq_ignore_ascii_case(s)
                && name.eq_ignore_ascii_case(n)
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
    pub fn is_dangerous_lock_stmt(&self, stmt: &NodeEnum) -> bool {
        match stmt {
            NodeEnum::AlterTableStmt(s) => s
                .relation
                .as_ref()
                .is_none_or(|r| !self.has_created_object(&r.schemaname, &r.relname)),
            NodeEnum::IndexStmt(s) => {
                !s.concurrent
                    && s.relation
                        .as_ref()
                        .is_none_or(|r| !self.has_created_object(&r.schemaname, &r.relname))
            }
            NodeEnum::DropStmt(s) => matches!(
                s.remove_type(),
                protobuf::ObjectType::ObjectTable
                    | protobuf::ObjectType::ObjectIndex
                    | protobuf::ObjectType::ObjectMatview
            ),
            NodeEnum::TruncateStmt(_) => true,
            NodeEnum::VacuumStmt(s) => is_vacuum_full(s),
            NodeEnum::ReindexStmt(s) => !is_reindex_concurrent(s),
            NodeEnum::RenameStmt(s) => s.relation.is_some(),
            NodeEnum::RefreshMatViewStmt(s) => !s.concurrent,
            _ => false,
        }
    }

    /// Record that an object was created, normalizing the schema name
    fn add_created_object(&mut self, schema: String, name: String) {
        self.created_objects.push((
            normalized_schema(&schema, &self.search_path).to_string(),
            name,
        ));
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
    fn is_access_exclusive_subcommand(subtype: protobuf::AlterTableType) -> bool {
        !matches!(subtype, protobuf::AlterTableType::AtValidateConstraint)
    }
    /// Returns the existing table targeted by an ALTER TABLE command that takes ACCESS EXCLUSIVE.
    pub fn access_exclusive_table_for_alter(
        &self,
        stmt: &protobuf::AlterTableStmt,
    ) -> Option<(String, String)> {
        let relation = stmt.relation.as_ref()?;
        let has_access_exclusive_cmd = stmt.cmds.iter().any(|cmd| matches!(&cmd.node, Some(NodeEnum::AlterTableCmd(c)) if Self::is_access_exclusive_subcommand(c.subtype())));
        if !has_access_exclusive_cmd {
            return None;
        }
        let schema = normalized_schema(&relation.schemaname, &self.search_path).to_string();
        let name = relation.relname.clone();
        if self.has_created_object(&schema, &name) {
            return None;
        }
        Some((schema, name))
    }

    fn update_timeout_flags(&mut self, stmt: &protobuf::VariableSetStmt) {
        use protobuf::VariableSetKind;
        let kind = VariableSetKind::try_from(stmt.kind).unwrap_or(VariableSetKind::Undefined);
        if kind == VariableSetKind::VarResetAll {
            self.lock_timeout_set = false;
            self.statement_timeout_set = false;
            self.idle_in_transaction_timeout_set = false;
            return;
        }
        let sets = matches!(
            kind,
            VariableSetKind::VarSetValue
                | VariableSetKind::VarSetCurrent
                | VariableSetKind::VarSetMulti
        );
        let flag = if stmt.name.eq_ignore_ascii_case("lock_timeout") {
            &mut self.lock_timeout_set
        } else if stmt.name.eq_ignore_ascii_case("statement_timeout") {
            &mut self.statement_timeout_set
        } else if stmt
            .name
            .eq_ignore_ascii_case("idle_in_transaction_session_timeout")
        {
            &mut self.idle_in_transaction_timeout_set
        } else {
            return;
        };
        *flag = sets;
    }
    fn variable_value(node: &NodeEnum) -> Option<String> {
        match node {
            NodeEnum::String(v) => Some(v.sval.clone()),
            NodeEnum::AConst(v) => match v.val.as_ref()? {
                protobuf::a_const::Val::Sval(s) => Some(s.sval.clone()),
                _ => None,
            },
            NodeEnum::ColumnRef(v) => v.fields.last().and_then(|f| match f.node.as_ref()? {
                NodeEnum::String(s) => Some(s.sval.clone()),
                _ => None,
            }),
            _ => None,
        }
    }
    fn apply_variable_settings(&mut self, stmt: &protobuf::VariableSetStmt) {
        use protobuf::VariableSetKind;
        let kind = VariableSetKind::try_from(stmt.kind).unwrap_or(VariableSetKind::Undefined);
        self.update_timeout_flags(stmt);
        if kind == VariableSetKind::VarResetAll {
            self.search_path = self.initial_search_path.clone();
            self.role = None;
            return;
        }
        if stmt.name.eq_ignore_ascii_case("search_path") {
            if stmt.is_local && self.local_search_path.is_none() {
                self.local_search_path = Some(self.search_path.clone());
            }
            if matches!(
                kind,
                VariableSetKind::VarReset
                    | VariableSetKind::VarResetAll
                    | VariableSetKind::VarSetDefault
            ) {
                self.search_path = self.initial_search_path.clone();
            } else {
                let path: Vec<_> = stmt
                    .args
                    .iter()
                    .filter_map(|a| a.node.as_ref().and_then(Self::variable_value))
                    .collect();
                if !path.is_empty() {
                    self.search_path = path;
                }
            }
        } else if stmt.name.eq_ignore_ascii_case("role") {
            if stmt.is_local && self.local_role.is_none() {
                self.local_role = Some(self.role.clone());
            }
            let role = stmt
                .args
                .first()
                .and_then(|a| a.node.as_ref().and_then(Self::variable_value));
            self.role = if matches!(
                kind,
                VariableSetKind::VarReset
                    | VariableSetKind::VarResetAll
                    | VariableSetKind::VarSetDefault
            ) || stmt
                .args
                .iter()
                .any(|a| matches!(a.node.as_ref(), Some(NodeEnum::AConst(c)) if c.isnull))
                || role
                    .as_deref()
                    .is_some_and(|r| r.eq_ignore_ascii_case("none"))
            {
                None
            } else {
                role
            };
        }
    }

    /// Update session state based on a statement.
    pub fn apply(&mut self, stmt: &NodeEnum) {
        if let NodeEnum::TransactionStmt(tx) = stmt {
            use protobuf::TransactionStmtKind;
            match tx.kind() {
                TransactionStmtKind::TransStmtBegin | TransactionStmtKind::TransStmtStart => {
                    if self.transaction_depth == 0 {
                        self.transaction_search_path = Some(self.search_path.clone());
                        self.transaction_role = Some(self.role.clone());
                    }
                    self.transaction_depth += 1;
                }
                TransactionStmtKind::TransStmtCommit => {
                    self.transaction_depth = self.transaction_depth.saturating_sub(1);
                    if let Some(path) = self.local_search_path.take() {
                        self.search_path = path;
                    }
                    if let Some(role) = self.local_role.take() {
                        self.role = role;
                    }
                    self.transaction_search_path = None;
                    self.transaction_role = None;
                    self.reset_transaction_state();
                }
                TransactionStmtKind::TransStmtRollback => {
                    if let Some(path) = self.transaction_search_path.take() {
                        self.search_path = path;
                    }
                    if let Some(role) = self.transaction_role.take() {
                        self.role = role;
                    }
                    self.transaction_depth = self.transaction_depth.saturating_sub(1);
                    self.local_search_path = None;
                    self.reset_transaction_state();
                }
                TransactionStmtKind::TransStmtSavepoint => self.transaction_depth += 1,
                TransactionStmtKind::TransStmtRelease
                | TransactionStmtKind::TransStmtRollbackTo => {
                    self.transaction_depth = self.transaction_depth.saturating_sub(1)
                }
                _ => {}
            }
            return;
        }
        if let NodeEnum::VariableSetStmt(s) = stmt {
            self.apply_variable_settings(s);
        }
        match stmt {
            NodeEnum::CreateStmt(s) => {
                if let Some(r) = &s.relation {
                    self.add_created_object(r.schemaname.clone(), r.relname.clone());
                }
            }
            NodeEnum::IndexStmt(s) => {
                if !s.idxname.is_empty() {
                    self.add_created_object(
                        s.relation
                            .as_ref()
                            .map(|r| r.schemaname.clone())
                            .unwrap_or_default(),
                        s.idxname.clone(),
                    );
                }
            }
            NodeEnum::CreateTableAsStmt(s) => {
                if let Some(i) = &s.into
                    && let Some(r) = &i.rel
                {
                    self.add_created_object(r.schemaname.clone(), r.relname.clone());
                }
            }
            _ => {}
        }
        if let NodeEnum::AlterTableStmt(s) = stmt {
            let (schema, table) = s
                .relation
                .as_ref()
                .map(|r| {
                    (
                        normalized_schema(&r.schemaname, &self.search_path).to_string(),
                        r.relname.clone(),
                    )
                })
                .unwrap_or_default();
            for item in &s.cmds {
                if let Some(NodeEnum::AlterTableCmd(c)) = &item.node
                    && c.subtype() == protobuf::AlterTableType::AtAddConstraint
                {
                    if let Some(NodeEnum::Constraint(c)) =
                        c.def.as_ref().and_then(|d| d.node.as_ref())
                    {
                        if c.skip_validation && !c.conname.is_empty() {
                            self.not_valid_constraints.push((
                                schema.clone(),
                                table.clone(),
                                c.conname.clone(),
                            ));
                        }
                    }
                }
            }
        }
        if let NodeEnum::AlterTableStmt(s) = stmt
            && let Some((schema, name)) = self.access_exclusive_table_for_alter(s)
        {
            self.holding_access_exclusive = true;
            if !self
                .access_exclusive_tables
                .iter()
                .any(|(a, b)| a == &schema && b == &name)
            {
                self.access_exclusive_tables.push((schema, name));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn apply(s: &mut Session, q: &str) {
        let n = pgls_query::parse(q).unwrap().into_root().unwrap();
        s.apply(&n);
    }
    fn dangerous(s: &Session, q: &str) -> bool {
        let n = pgls_query::parse(q).unwrap().into_root().unwrap();
        s.is_dangerous_lock_stmt(&n)
    }

    #[test]
    fn transaction_boundaries_reset_state_and_timeouts() {
        let mut s = Session::new(vec!["public".into()]);
        apply(&mut s, "SET lock_timeout='1s'");
        apply(&mut s, "CREATE TABLE made (id int)");
        assert!(s.has_lock_timeout() && s.has_created_object("", "made"));
        apply(&mut s, "BEGIN");
        assert_eq!(s.transaction_depth(), 1);
        apply(&mut s, "COMMIT");
        assert_eq!(s.transaction_depth(), 0);
        assert!(!s.has_lock_timeout() && !s.has_created_object("", "made"));
        apply(&mut s, "SET statement_timeout='1s'");
        apply(&mut s, "ROLLBACK");
        assert!(!s.has_statement_timeout());
        apply(&mut s, "SET lock_timeout='1s'");
        apply(&mut s, "SET statement_timeout='1s'");
        apply(&mut s, "SET idle_in_transaction_session_timeout='1s'");
        assert!(
            s.has_lock_timeout()
                && s.has_statement_timeout()
                && s.has_idle_in_transaction_timeout()
        );
        apply(&mut s, "RESET ALL");
        assert!(
            !s.has_lock_timeout()
                && !s.has_statement_timeout()
                && !s.has_idle_in_transaction_timeout()
        );
    }
    #[test]
    fn created_objects_and_schema_use_search_path() {
        let mut empty_path = Session::default();
        apply(&mut empty_path, "CREATE TABLE public_default (id int)");
        assert!(empty_path.has_created_object("", "public_default"));
        let mut s = Session::new(vec!["app".into(), "public".into()]);
        apply(&mut s, "CREATE TABLE made (id int)");
        assert!(s.has_created_object("", "made"));
        assert!(!s.has_created_object("public", "made"));
        apply(&mut s, "CREATE INDEX made_idx ON made (id)");
        assert!(s.has_created_object("app", "made_idx"));
    }
    #[test]
    fn not_valid_and_access_exclusive_are_tracked() {
        let mut s = Session::new(vec!["app".into()]);
        apply(
            &mut s,
            "ALTER TABLE things ADD CONSTRAINT things_fk FOREIGN KEY (id) REFERENCES other(id) NOT VALID",
        );
        assert!(s.has_not_valid_constraint("", "things", "things_fk"));
        let parsed = pgls_query::parse("ALTER TABLE things ADD COLUMN name text")
            .unwrap()
            .into_root()
            .unwrap();
        let NodeEnum::AlterTableStmt(stmt) = parsed else {
            panic!()
        };
        assert_eq!(
            s.access_exclusive_table_for_alter(&stmt),
            Some(("app".into(), "things".into()))
        );
        s.apply(&NodeEnum::AlterTableStmt(stmt));
        assert!(s.is_holding_access_exclusive());
        assert_eq!(s.access_exclusive_tables().len(), 1);
        apply(&mut s, "CREATE TABLE fresh (id int)");
        let fresh = pgls_query::parse("ALTER TABLE fresh ADD COLUMN x int")
            .unwrap()
            .into_root()
            .unwrap();
        let NodeEnum::AlterTableStmt(stmt) = fresh else {
            panic!()
        };
        assert_eq!(s.access_exclusive_table_for_alter(&stmt), None);
        let validate = pgls_query::parse("ALTER TABLE things VALIDATE CONSTRAINT x")
            .unwrap()
            .into_root()
            .unwrap();
        let NodeEnum::AlterTableStmt(stmt) = validate else {
            panic!()
        };
        assert_eq!(s.access_exclusive_table_for_alter(&stmt), None);
    }
    #[test]
    fn dangerous_lock_statements() {
        let s = Session::new(vec!["public".into()]);
        for q in [
            "ALTER TABLE t ADD COLUMN x int",
            "CREATE INDEX i ON t(id)",
            "DROP TABLE t",
            "TRUNCATE t",
            "VACUUM (FULL) t",
            "REINDEX TABLE t",
            "ALTER TABLE t RENAME TO x",
            "REFRESH MATERIALIZED VIEW t",
        ] {
            assert!(dangerous(&s, q), "{q}");
        }
        for q in [
            "CREATE INDEX CONCURRENTLY i ON t(id)",
            "VACUUM t",
            "REINDEX TABLE CONCURRENTLY t",
            "REFRESH MATERIALIZED VIEW CONCURRENTLY t",
        ] {
            assert!(!dangerous(&s, q), "{q}");
        }
    }
    #[test]
    fn search_path_and_role_settings() {
        let mut s = Session::new(vec!["initial".into()]);
        apply(&mut s, "SET search_path TO app, public");
        assert_eq!(s.search_path(), ["app", "public"]);
        apply(&mut s, "SET search_path = DEFAULT");
        assert_eq!(s.search_path(), ["initial"]);
        apply(
            &mut s,
            "SET search_path TO \"QuotedSchema\", 'other schema'",
        );
        assert_eq!(s.search_path(), ["QuotedSchema", "other schema"]);
        apply(&mut s, "BEGIN");
        apply(&mut s, "SET search_path TO changed");
        apply(&mut s, "ROLLBACK");
        assert_eq!(s.search_path(), ["QuotedSchema", "other schema"]);
        apply(&mut s, "BEGIN");
        apply(&mut s, "SET LOCAL search_path TO local_schema");
        apply(&mut s, "COMMIT");
        assert_eq!(s.search_path(), ["QuotedSchema", "other schema"]);
        apply(&mut s, "BEGIN");
        apply(&mut s, "SET LOCAL search_path TO local_schema");
        apply(&mut s, "ROLLBACK");
        assert_eq!(s.search_path(), ["QuotedSchema", "other schema"]);
        apply(&mut s, "RESET search_path");
        assert_eq!(s.search_path(), ["initial"]);
        apply(&mut s, "SET search_path TO another");
        apply(&mut s, "RESET ALL");
        assert_eq!(s.search_path(), ["initial"]);
        apply(&mut s, "SET ROLE some_role");
        assert_eq!(s.role(), Some("some_role"));
        apply(&mut s, "SET ROLE NONE");
        assert_eq!(s.role(), None);
        apply(&mut s, "SET ROLE some_role");
        apply(&mut s, "BEGIN");
        apply(&mut s, "SET ROLE another_role");
        apply(&mut s, "ROLLBACK");
        assert_eq!(s.role(), Some("some_role"));
        apply(&mut s, "SET ROLE some_role");
        apply(&mut s, "RESET ROLE");
        assert_eq!(s.role(), None);
    }
}
