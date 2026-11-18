use crate::protobuf::{IntoClause, SelectStmt, SetOperation};

/// How many arms of a chain of set operations, like `select 1 union select 2 union ...`, are
/// followed to the left-most select. Postgres transforms the chain recursively, and fails with
/// its stack depth limit long before a hundred thousand arms.
const SET_OPERATION_ARMS_MAX: usize = 100_000;

impl SelectStmt {
    /// The left-most select of a chain of set operations, whose columns name those of the chain
    /// and which holds its `INTO`, or the select itself. `None` after [`SET_OPERATION_ARMS_MAX`]
    /// arms.
    pub fn leftmost(&self) -> Option<&SelectStmt> {
        let mut select = self;
        for _ in 0..SET_OPERATION_ARMS_MAX {
            if select.op() == SetOperation::SetopNone {
                return Some(select);
            }
            select = select.larg.as_deref()?;
        }
        tracing::warn!(
            "stopped looking for the left-most select after {SET_OPERATION_ARMS_MAX} set operations"
        );
        None
    }

    /// The `INTO` of `SELECT ... INTO`, which makes the statement create a table. In a set
    /// operation like `select 1 into t union select 2`, it is on the left-most select
    /// ([`transformOptionalSelectInto`]), not in the [`SelectStmt::into_clause`] of the statement.
    ///
    /// [`transformOptionalSelectInto`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/analyze.c#L273
    pub fn into_clause_of_query(&self) -> Option<&IntoClause> {
        self.leftmost()?.into_clause.as_deref()
    }
}
