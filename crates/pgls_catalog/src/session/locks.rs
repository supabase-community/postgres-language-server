//! Lock, timeout, and constraint state of the current transaction, for the safety rules.
//!
//! The state is reset at the end of each transaction, to avoid false positives across separate
//! transactions in the same file.

use pgls_query::{
    NodeEnum,
    protobuf::{self, VariableSetKind},
};

#[derive(Debug, Default, Clone)]
pub(super) struct Locks {
    /// Whether `SET lock_timeout` has been called in this transaction
    pub(super) lock_timeout_set: bool,
    /// Whether `SET statement_timeout` has been called in this transaction
    pub(super) statement_timeout_set: bool,
    /// Whether `SET idle_in_transaction_session_timeout` has been called in this transaction
    pub(super) idle_in_transaction_timeout_set: bool,
    /// Objects (schema, name) created in this transaction
    created_objects: Vec<(String, String)>,
    /// Whether an ACCESS EXCLUSIVE lock is currently being held
    /// This is set when an ALTER TABLE is executed on an existing table
    pub(super) holding_access_exclusive: bool,
    /// Constraints added with NOT VALID: (schema, table, constraint_name)
    not_valid_constraints: Vec<(String, String, String)>,
    /// Tables holding ACCESS EXCLUSIVE locks in this transaction (for wide lock window detection)
    pub(super) access_exclusive_tables: Vec<(String, String)>,
}

impl Locks {
    pub(super) fn has_not_valid_constraint(
        &self,
        schema: &str,
        table: &str,
        name: &str,
        search_path: &[String],
    ) -> bool {
        self.not_valid_constraints.iter().any(|(s, t, n)| {
            normalized_schema(schema, search_path).eq_ignore_ascii_case(s)
                && table.eq_ignore_ascii_case(t)
                && name.eq_ignore_ascii_case(n)
        })
    }

    pub(super) fn has_created_object(
        &self,
        schema: &str,
        name: &str,
        search_path: &[String],
    ) -> bool {
        self.created_objects.iter().any(|(s, n)| {
            normalized_schema(schema, search_path).eq_ignore_ascii_case(s)
                && name.eq_ignore_ascii_case(n)
        })
    }

    pub(super) fn is_dangerous_lock_stmt(&self, stmt: &NodeEnum, search_path: &[String]) -> bool {
        let is_new = |relation: &protobuf::RangeVar| {
            self.has_created_object(&relation.schemaname, &relation.relname, search_path)
        };
        match stmt {
            NodeEnum::AlterTableStmt(s) => s.relation.as_ref().is_none_or(|r| !is_new(r)),
            NodeEnum::IndexStmt(s) => {
                !s.concurrent && s.relation.as_ref().is_none_or(|r| !is_new(r))
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

    pub(super) fn access_exclusive_table_for_alter(
        &self,
        stmt: &protobuf::AlterTableStmt,
        search_path: &[String],
    ) -> Option<(String, String)> {
        let relation = stmt.relation.as_ref()?;
        // Some subcommands take lighter locks.
        let has_access_exclusive_cmd = stmt.cmds.iter().any(|cmd| {
            matches!(&cmd.node, Some(NodeEnum::AlterTableCmd(c))
                if c.subtype() != protobuf::AlterTableType::AtValidateConstraint)
        });
        if !has_access_exclusive_cmd {
            return None;
        }
        let schema = normalized_schema(&relation.schemaname, search_path).to_string();
        let name = relation.relname.clone();
        if self.has_created_object(&schema, &name, search_path) {
            return None;
        }
        Some((schema, name))
    }

    /// Forgets the state of the transaction that ended.
    pub(super) fn reset(&mut self) {
        *self = Self::default();
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

    pub(super) fn apply(&mut self, stmt: &NodeEnum, search_path: &[String]) {
        match stmt {
            NodeEnum::CreateStmt(s) => {
                if let Some(r) = &s.relation {
                    self.add_created_object(&r.schemaname, &r.relname, search_path);
                }
            }
            NodeEnum::IndexStmt(s) => {
                if !s.idxname.is_empty() {
                    let schema = s
                        .relation
                        .as_ref()
                        .map(|r| r.schemaname.as_str())
                        .unwrap_or_default();
                    self.add_created_object(schema, &s.idxname, search_path);
                }
            }
            NodeEnum::CreateTableAsStmt(s) => {
                if let Some(r) = s.into.as_ref().and_then(|into| into.rel.as_ref()) {
                    self.add_created_object(&r.schemaname, &r.relname, search_path);
                }
            }
            NodeEnum::AlterTableStmt(s) => {
                self.add_not_valid_constraints(s, search_path);
                if let Some((schema, name)) = self.access_exclusive_table_for_alter(s, search_path)
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
            _ => {}
        }
    }

    fn add_created_object(&mut self, schema: &str, name: &str, search_path: &[String]) {
        self.created_objects.push((
            normalized_schema(schema, search_path).to_string(),
            name.to_owned(),
        ));
    }

    fn add_not_valid_constraints(
        &mut self,
        stmt: &protobuf::AlterTableStmt,
        search_path: &[String],
    ) {
        let (schema, table) = stmt
            .relation
            .as_ref()
            .map(|r| {
                (
                    normalized_schema(&r.schemaname, search_path).to_string(),
                    r.relname.clone(),
                )
            })
            .unwrap_or_default();
        for item in &stmt.cmds {
            if let Some(NodeEnum::AlterTableCmd(c)) = &item.node
                && c.subtype() == protobuf::AlterTableType::AtAddConstraint
                && let Some(NodeEnum::Constraint(c)) = c.def.as_ref().and_then(|d| d.node.as_ref())
                && c.skip_validation
                && !c.conname.is_empty()
            {
                self.not_valid_constraints
                    .push((schema.clone(), table.clone(), c.conname.clone()));
            }
        }
    }
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
            _ => true,
        },
        None => true,
    }
}
