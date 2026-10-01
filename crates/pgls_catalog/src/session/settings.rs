//! The settings that affect name resolution: `search_path` and `role`.

use pgls_query::{
    NodeEnum,
    protobuf::{self, VariableSetKind},
};

#[derive(Debug, Default, Clone)]
pub(super) struct Settings {
    /// Search path the session started with.
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

impl Settings {
    pub(super) fn new(search_path: Vec<String>) -> Self {
        Self {
            initial_search_path: search_path.clone(),
            search_path,
            ..Self::default()
        }
    }

    pub(super) fn search_path(&self) -> &[String] {
        &self.search_path
    }

    pub(super) fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }

    /// Remembers the settings at the start of the outermost transaction.
    pub(super) fn begin(&mut self) {
        self.transaction_search_path = Some(self.search_path.clone());
        self.transaction_role = Some(self.role.clone());
    }

    /// Keeps the settings of the transaction, except those set with `SET LOCAL`.
    pub(super) fn commit(&mut self) {
        if let Some(path) = self.local_search_path.take() {
            self.search_path = path;
        }
        if let Some(role) = self.local_role.take() {
            self.role = role;
        }
        self.transaction_search_path = None;
        self.transaction_role = None;
    }

    /// Restores the settings of the start of the transaction.
    pub(super) fn rollback(&mut self) {
        if let Some(path) = self.transaction_search_path.take() {
            self.search_path = path;
        }
        if let Some(role) = self.transaction_role.take() {
            self.role = role;
        }
        self.local_search_path = None;
        self.local_role = None;
    }

    pub(super) fn apply_variable_set(&mut self, stmt: &protobuf::VariableSetStmt) {
        let kind = VariableSetKind::try_from(stmt.kind).unwrap_or(VariableSetKind::Undefined);
        if kind == VariableSetKind::VarResetAll {
            self.search_path = self.initial_search_path.clone();
            self.role = None;
            return;
        }
        let resets = matches!(
            kind,
            VariableSetKind::VarReset | VariableSetKind::VarSetDefault
        );

        if stmt.name.eq_ignore_ascii_case("search_path") {
            if stmt.is_local && self.local_search_path.is_none() {
                self.local_search_path = Some(self.search_path.clone());
            }
            if resets {
                self.search_path = self.initial_search_path.clone();
            } else {
                let path: Vec<_> = stmt
                    .args
                    .iter()
                    .filter_map(|a| a.node.as_ref().and_then(variable_value))
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
                .and_then(|a| a.node.as_ref().and_then(variable_value));
            let is_null = stmt
                .args
                .iter()
                .any(|a| matches!(a.node.as_ref(), Some(NodeEnum::AConst(c)) if c.isnull));
            let is_none = role
                .as_deref()
                .is_some_and(|r| r.eq_ignore_ascii_case("none"));
            self.role = if resets || is_null || is_none {
                None
            } else {
                role
            };
        }
    }
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
