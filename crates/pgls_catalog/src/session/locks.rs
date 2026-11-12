//! Lock, timeout, and constraint state of the current transaction, for the safety rules.
//!
//! The state is reset at the end of each transaction, to avoid false positives across separate
//! transactions in the same file.

use pgls_query::{
    NodeEnum,
    protobuf::{self, VariableSetKind},
};
use pgls_query_ext::utils::{alter_table_cmds, cmd_def};

use super::lock_level::{LockMode, TableLock, relation_locks};
use crate::Catalog;

#[derive(Debug, Default, Clone)]
pub(super) struct Locks {
    /// Whether `SET lock_timeout` has been called in this transaction
    pub(super) lock_timeout_set: bool,
    /// Whether `SET statement_timeout` has been called in this transaction
    pub(super) statement_timeout_set: bool,
    /// Whether `SET idle_in_transaction_session_timeout` has been called in this transaction
    pub(super) idle_in_transaction_timeout_set: bool,
    /// Objects (schema, name) created in this transaction, in the schema Postgres created them in
    created_objects: Vec<(String, String)>,
    /// Whether an ACCESS EXCLUSIVE lock is currently being held, set by an `ALTER TABLE` that
    /// takes one on an existing table
    pub(super) holding_access_exclusive: bool,
    /// Constraints added with NOT VALID that can be validated later: (schema, table, constraint_name)
    not_valid_constraints: Vec<(String, String, String)>,
    /// Tables holding ACCESS EXCLUSIVE locks in this transaction (for wide lock window detection)
    pub(super) access_exclusive_tables: Vec<(String, String)>,
}

// The parser folds unquoted names to lower case, so names are compared exactly: `"Users"` and
// `users` are different tables.
impl Locks {
    pub(super) fn has_not_valid_constraint(
        &self,
        table: &protobuf::RangeVar,
        name: &str,
        catalog: &Catalog,
        search_path: &[String],
    ) -> bool {
        let (schema, table) = catalog.relation_name_of(table, search_path);
        self.not_valid_constraints
            .iter()
            .any(|(s, t, n)| *s == schema && *t == table && n == name)
    }

    pub(super) fn has_created_relation(
        &self,
        relation: &protobuf::RangeVar,
        catalog: &Catalog,
        search_path: &[String],
    ) -> bool {
        let (schema, name) = catalog.relation_name_of(relation, search_path);
        self.has_created_object(&schema, &name)
    }

    pub(super) fn has_created_object(&self, schema: &str, name: &str) -> bool {
        self.created_objects
            .iter()
            .any(|(s, n)| s == schema && n == name)
    }

    pub(super) fn is_dangerous_lock_stmt(
        &self,
        stmt: &NodeEnum,
        catalog: &Catalog,
        search_path: &[String],
    ) -> bool {
        match stmt {
            NodeEnum::AlterTableStmt(_) | NodeEnum::IndexStmt(_) => self
                .table_locks(stmt, catalog, search_path)
                .iter()
                .any(|lock| lock.mode.blocks_writes()),
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

    /// The locks that the statement takes on tables that this transaction didn't create, see
    /// [`relation_locks`].
    pub(super) fn table_locks(
        &self,
        stmt: &NodeEnum,
        catalog: &Catalog,
        search_path: &[String],
    ) -> Vec<TableLock> {
        relation_locks(stmt)
            .into_iter()
            .filter_map(|(mode, relation)| {
                let (schema, name) = catalog.relation_name_of(relation, search_path);
                (!self.has_created_object(&schema, &name)).then_some(TableLock {
                    mode,
                    schema,
                    name,
                })
            })
            .collect()
    }

    /// Forgets the state of the transaction that ended.
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    /// Forgets the locks and the unvalidated constraints of a statement that committed on its
    /// own. Session-level timeouts stay set, and tables created before are still new.
    pub(super) fn release(&mut self) {
        self.holding_access_exclusive = false;
        self.access_exclusive_tables.clear();
        self.not_valid_constraints.clear();
    }

    pub(super) fn apply_variable_set(&mut self, stmt: &protobuf::VariableSetStmt) {
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

    /// Applies a statement. `catalog` is the catalog after the statement, which knows where
    /// the statement created its objects, and whether `IF NOT EXISTS` skipped them.
    pub(super) fn apply(&mut self, stmt: &NodeEnum, catalog: &Catalog, search_path: &[String]) {
        let name_of =
            |relation: &protobuf::RangeVar| catalog.relation_name_of(relation, search_path);
        let created = |relation: &protobuf::RangeVar| {
            if catalog.skipped_existing() {
                return None;
            }
            catalog.relation_creation_name_of(relation, search_path)
        };
        match stmt {
            NodeEnum::CreateStmt(s) => {
                if let Some(key) = s.relation.as_ref().and_then(created) {
                    self.created_objects.push(key);
                }
            }
            // An index is in the schema of its table.
            NodeEnum::IndexStmt(s) => {
                if !s.idxname.is_empty()
                    && !catalog.skipped_existing()
                    && let Some(r) = &s.relation
                {
                    let (schema, _) = name_of(r);
                    self.created_objects.push((schema, s.idxname.clone()));
                }
            }
            NodeEnum::CreateTableAsStmt(s) => {
                if let Some(key) = s
                    .into
                    .as_ref()
                    .and_then(|into| into.rel.as_ref())
                    .and_then(created)
                {
                    self.created_objects.push(key);
                }
            }
            NodeEnum::AlterTableStmt(s) => {
                if let Some(r) = &s.relation {
                    let (schema, table) = name_of(r);
                    self.add_not_valid_constraints(s, &schema, &table);
                }
                for lock in self.table_locks(stmt, catalog, search_path) {
                    if lock.mode != LockMode::AccessExclusive {
                        continue;
                    }
                    self.holding_access_exclusive = true;
                    if !self
                        .access_exclusive_tables
                        .iter()
                        .any(|(schema, name)| *schema == lock.schema && *name == lock.name)
                    {
                        self.access_exclusive_tables.push((lock.schema, lock.name));
                    }
                }
            }
            _ => {}
        }
    }

    fn add_not_valid_constraints(
        &mut self,
        stmt: &protobuf::AlterTableStmt,
        schema: &str,
        table: &str,
    ) {
        for cmd in alter_table_cmds(stmt) {
            if cmd.subtype() == protobuf::AlterTableType::AtAddConstraint
                && let Some(NodeEnum::Constraint(c)) = cmd_def(cmd)
                && c.skip_validation
                // A NOT ENFORCED constraint (Postgres 18) is NOT VALID too, but can't be validated.
                // Only CHECK and FOREIGN KEY constraints can be NOT ENFORCED; the parser leaves
                // `is_enforced` false for the others, e.g. NOT NULL ... NOT VALID.
                && (c.is_enforced
                    || !matches!(
                        c.contype(),
                        protobuf::ConstrType::ConstrCheck | protobuf::ConstrType::ConstrForeign
                    ))
                && !c.conname.is_empty()
            {
                self.not_valid_constraints.push((
                    schema.to_owned(),
                    table.to_owned(),
                    c.conname.clone(),
                ));
            }
        }
    }
}

pub fn is_vacuum_full(stmt: &protobuf::VacuumStmt) -> bool {
    stmt.options.iter().any(
        |opt| matches!(&opt.node, Some(NodeEnum::DefElem(def)) if def_elem_is_enabled(def, "full")),
    )
}

pub fn is_reindex_concurrent(stmt: &protobuf::ReindexStmt) -> bool {
    stmt.params.iter().any(|param| {
        matches!(&param.node, Some(NodeEnum::DefElem(def)) if def_elem_is_enabled(def, "concurrently"))
    })
}

fn def_elem_is_enabled(def: &protobuf::DefElem, name: &str) -> bool {
    if !def.defname.eq_ignore_ascii_case(name) {
        return false;
    }
    match &def.arg {
        Some(arg) => match &arg.node {
            Some(NodeEnum::Integer(i)) => i.ival != 0,
            Some(NodeEnum::Boolean(b)) => b.boolval,
            // `(CONCURRENTLY false)`, like [`defGetBoolean`](https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/define.c#L94).
            Some(NodeEnum::String(s)) => !["false", "off"]
                .iter()
                .any(|value| s.sval.eq_ignore_ascii_case(value)),
            _ => true,
        },
        None => true,
    }
}
