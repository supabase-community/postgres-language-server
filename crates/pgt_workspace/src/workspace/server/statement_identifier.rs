use std::sync::Arc;

use serde::{Deserialize, Serialize};

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
pub enum StatementId {
    Root {
        content: Arc<str>,
    },
    Child {
        content: Arc<str>,        // child's actual content
        parent_content: Arc<str>, // parent's content for lookups
    },
}

// this is only here for strum to work on the code actions enum
impl Default for StatementId {
    fn default() -> Self {
        StatementId::Root { content: "".into() }
    }
}

impl StatementId {
    pub fn new(statement: &str) -> Self {
        StatementId::Root {
            content: statement.into(),
        }
    }

    /// Use this if you need to create a matching `StatementId::Child` for `Root`.
    /// You cannot create a `Child` of a `Child`.
    /// Note: This method requires the child content to be provided.
    pub fn create_child(&self, child_content: &str) -> StatementId {
        match self {
            StatementId::Root { content } => StatementId::Child {
                content: child_content.into(),
                parent_content: content.clone(),
            },
            StatementId::Child { .. } => panic!("Cannot create child from a child statement id"),
        }
    }

    pub fn content(&self) -> &str {
        match self {
            StatementId::Root { content } => content,
            StatementId::Child { content, .. } => content,
        }
    }

    /// Returns the parent content if this is a child statement
    pub fn parent_content(&self) -> Option<&str> {
        match self {
            StatementId::Root { .. } => None,
            StatementId::Child { parent_content, .. } => Some(parent_content),
        }
    }

    pub fn is_root(&self) -> bool {
        matches!(self, StatementId::Root { .. })
    }

    pub fn is_child(&self) -> bool {
        matches!(self, StatementId::Child { .. })
    }

    pub fn is_child_of(&self, maybe_parent: &StatementId) -> bool {
        match self {
            StatementId::Root { .. } => false,
            StatementId::Child { parent_content, .. } => match maybe_parent {
                StatementId::Root { content } => parent_content == content,
                StatementId::Child { .. } => false,
            },
        }
    }

    pub fn parent(&self) -> Option<StatementId> {
        match self {
            StatementId::Root { .. } => None,
            StatementId::Child { parent_content, .. } => Some(StatementId::Root {
                content: parent_content.clone(),
            }),
        }
    }
}
