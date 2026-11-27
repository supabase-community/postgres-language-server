use crate::protobuf::{DefElem, VacuumStmt};

impl VacuumStmt {
    /// Whether the statement is `VACUUM FULL`, like the `full` option of [`ExecVacuum`]. Not for
    /// an invalid value, which Postgres rejects.
    ///
    /// [`ExecVacuum`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/commands/vacuum.c#L243
    pub fn is_full(&self) -> bool {
        DefElem::find(&self.options, "full").and_then(DefElem::bool_value) == Some(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::NodeEnum;

    fn vacuum_full(sql: &str) -> bool {
        match crate::parse(sql).unwrap().into_root().unwrap() {
            NodeEnum::VacuumStmt(stmt) => stmt.is_full(),
            _ => panic!("Expected VacuumStmt"),
        }
    }

    #[test]
    fn vacuum_full_values() {
        assert!(vacuum_full("vacuum full t"));
        assert!(vacuum_full("vacuum (full) t"));
        assert!(vacuum_full("vacuum (FULL true) t"));
        assert!(vacuum_full("vacuum (full 1) t"));
        assert!(vacuum_full("vacuum (full 'ON') t"));
        assert!(!vacuum_full("vacuum t"));
        assert!(!vacuum_full("vacuum (full off) t"));
        assert!(!vacuum_full("vacuum (full 0) t"));
        // Postgres rejects these.
        assert!(!vacuum_full("vacuum (full 2) t"));
        assert!(!vacuum_full("vacuum (full '0') t"));
        assert!(!vacuum_full("vacuum (full 1.0) t"));
        assert!(!vacuum_full("vacuum (\"FULL\") t"));
        // The last one wins.
        assert!(!vacuum_full("vacuum (full, full false) t"));
        assert!(vacuum_full("vacuum (full false, full) t"));
    }
}
