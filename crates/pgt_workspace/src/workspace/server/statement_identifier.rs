use std::ops::Deref;
use serde::{Deserialize, Serialize};

pub type RootId = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum StatementId {
    Root(RootId),
    // StatementId is the same as the root id since we can only have a single sql function body per Root
    Child(RootId),
}

impl Default for StatementId {
    fn default() -> Self {
        StatementId::Root(0)
    }
}

/// Helper struct to generate unique statement ids
pub struct IdGenerator {
    next_id: RootId,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn next(&mut self) -> StatementId {
        let id = self.next_id;
        self.next_id += 1;
        StatementId::Root(id)
    }
}

impl StatementId {
    pub fn as_child(&self) -> Option<StatementId> {
        match self {
            StatementId::Root(id) => Some(StatementId::Child(*id)),
            StatementId::Child(_) => None,
        }
    }

    pub fn create_child(&self) -> StatementId {
        match self {
            StatementId::Root(id) => StatementId::Child(*id),
            StatementId::Child(_) => panic!("Cannot create child from a child statement id"),
        }
    }
}

impl Deref for StatementId {
    type Target = RootId;

    fn deref(&self) -> &Self::Target {
        match self {
            StatementId::Root(id) => id,
            StatementId::Child(id) => id,
        }
    }
}
