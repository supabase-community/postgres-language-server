use crate::protobuf::{DefElem, ReindexStmt};

impl ReindexStmt {
    /// Whether the statement is `REINDEX CONCURRENTLY`, like the `concurrently` option of
    /// [`ExecReindex`]. Not for an invalid value, which Postgres rejects.
    ///
    /// [`ExecReindex`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/indexcmds.c#L2857
    pub fn is_concurrent(&self) -> bool {
        DefElem::find(&self.params, "concurrently").and_then(DefElem::bool_value) == Some(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::NodeEnum;

    fn reindex_concurrent(sql: &str) -> bool {
        match crate::parse(sql).unwrap().into_root().unwrap() {
            NodeEnum::ReindexStmt(stmt) => stmt.is_concurrent(),
            _ => panic!("Expected ReindexStmt"),
        }
    }

    #[test]
    fn reindex_concurrent_values() {
        assert!(reindex_concurrent("reindex index concurrently i"));
        assert!(reindex_concurrent("reindex (concurrently) index i"));
        assert!(!reindex_concurrent("reindex index i"));
        assert!(!reindex_concurrent("reindex (concurrently false) index i"));
        // `CONCURRENTLY` after the object kind comes after the options in the list.
        assert!(reindex_concurrent(
            "reindex (concurrently false) index concurrently i"
        ));
    }
}
