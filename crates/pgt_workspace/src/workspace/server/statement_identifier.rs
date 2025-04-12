use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RootId {
    inner: usize,
}

#[cfg(test)]
impl From<RootId> for usize {
    fn from(val: RootId) -> Self {
        val.inner
    }
}

#[cfg(test)]
impl From<usize> for RootId {
    fn from(inner: usize) -> Self {
        RootId { inner }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
/// `StatementId` can represent IDs for nested statements.
///
/// For example, an SQL function really consist of two statements; the function creation
/// and the body:
///
/// ```sql
/// create or replace function get_product_name(product_id INT) -- the root statement
/// returns varchar as $$
///   select * from … -- the child statement
/// $$ LANGUAGE plpgsql;
/// ```
///
/// For now, we only support SQL functions – no complex, nested statements.
///
/// An SQL function only ever has ONE child, that's why the inner `RootId` of a `Root`
/// is the same as the one of its `Child`.
pub enum StatementId {
    Root(RootId),
    // StatementId is the same as the root id since we can only have a single sql function body per Root
    Child(RootId),
}

impl Default for StatementId {
    fn default() -> Self {
        StatementId::Root(RootId { inner: 0 })
    }
}

impl StatementId {
    pub fn raw(&self) -> usize {
        match self {
            StatementId::Root(s) => s.inner,
            StatementId::Child(s) => s.inner,
        }
    }
}

/// Helper struct to generate unique statement ids
pub struct StatementIdGenerator {
    next_id: usize,
}

impl StatementIdGenerator {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn next(&mut self) -> StatementId {
        let id = self.next_id;
        self.next_id += 1;
        StatementId::Root(RootId { inner: id })
    }
}

impl StatementId {
    /// Use this to get the matching `StatementId::Child` for
    /// a `StatementId::Root`.
    /// If the `StatementId` was already a `Child`, this will return `None`.
    /// It is not guaranteed that the `Root` actually has a `Child` statement in the workspace.
    pub fn get_child_id(&self) -> Option<StatementId> {
        match self {
            StatementId::Root(id) => Some(StatementId::Child(RootId { inner: id.inner })),
            StatementId::Child(_) => None,
        }
    }

    /// Use this if you need to create a matching `StatementId::Child` for `Root`.
    /// You cannot create a `Child` of a `Child`.
    pub fn create_child(&self) -> StatementId {
        match self {
            StatementId::Root(id) => StatementId::Child(RootId { inner: id.inner }),
            StatementId::Child(_) => panic!("Cannot create child from a child statement id"),
        }
    }
}
