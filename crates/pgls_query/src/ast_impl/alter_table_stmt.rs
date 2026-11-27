use crate::{
    NodeEnum,
    protobuf::{AlterTableCmd, AlterTableStmt, AlterTableType, ColumnDef, Constraint, RangeVar},
};

impl AlterTableStmt {
    /// The subcommands of the statement, e.g. `ADD COLUMN` and `DROP CONSTRAINT`.
    pub fn commands(&self) -> impl Iterator<Item = &AlterTableCmd> {
        self.cmds.iter().filter_map(|cmd| match cmd.node.as_ref()? {
            NodeEnum::AlterTableCmd(cmd) => Some(cmd.as_ref()),
            _ => None,
        })
    }
}

impl AlterTableCmd {
    /// The definition of the subcommand, e.g. the column of `ADD COLUMN` or the constraint of
    /// `ADD CONSTRAINT`.
    pub fn definition(&self) -> Option<&NodeEnum> {
        self.def.as_ref()?.node.as_ref()
    }

    /// Whether the subcommand is `DETACH PARTITION ... CONCURRENTLY`, the only subcommand that
    /// Postgres can run concurrently.
    pub fn detaches_concurrently(&self) -> bool {
        matches!(self.definition(), Some(NodeEnum::PartitionCmd(partition)) if partition.concurrent)
    }
}

/// Where an `ALTER TABLE` adds a constraint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstraintSite {
    /// `ADD CONSTRAINT`, or `ADD` with a table constraint.
    Table,
    /// Inline on a column of `ADD COLUMN`, e.g. `ADD COLUMN a int REFERENCES b`.
    Column,
}

impl NodeEnum {
    /// The subcommands of an `ALTER TABLE` statement. Other statements have none.
    pub fn alter_table_commands(&self) -> impl Iterator<Item = &AlterTableCmd> {
        let stmt = match self {
            NodeEnum::AlterTableStmt(stmt) => Some(stmt),
            _ => None,
        };
        stmt.into_iter().flat_map(|stmt| stmt.commands())
    }

    /// The table that an `ALTER TABLE` changes. Other statements change none.
    pub fn altered_relation(&self) -> Option<&RangeVar> {
        match self {
            NodeEnum::AlterTableStmt(stmt) => stmt.relation.as_ref(),
            _ => None,
        }
    }

    /// The columns that a statement adds or retypes: the columns of `CREATE TABLE`, and of
    /// `ALTER TABLE ... ADD COLUMN` and `ALTER COLUMN ... TYPE`.
    pub fn column_defs(&self) -> impl Iterator<Item = &ColumnDef> {
        let table_elts = match self {
            NodeEnum::CreateStmt(stmt) => stmt.table_elts.as_slice(),
            _ => &[],
        };
        let created = table_elts.iter().filter_map(|elt| match &elt.node {
            Some(NodeEnum::ColumnDef(col_def)) => Some(col_def.as_ref()),
            _ => None,
        });
        let altered = self
            .alter_table_commands()
            .filter(|cmd| {
                matches!(
                    cmd.subtype(),
                    AlterTableType::AtAddColumn | AlterTableType::AtAlterColumnType
                )
            })
            .filter_map(|cmd| match cmd.definition() {
                Some(NodeEnum::ColumnDef(col_def)) => Some(col_def.as_ref()),
                _ => None,
            });
        created.chain(altered)
    }

    /// The constraints that an `ALTER TABLE` adds, in the order of the subcommands: those of
    /// `ADD CONSTRAINT`, and the inline constraints of `ADD COLUMN`. Other statements add none.
    ///
    /// Postgres adds a constraint without checking the existing rows when it is `NOT VALID` or
    /// `NOT ENFORCED`, which implies it: the parser sets `skip_validation` for both
    /// ([`processCASbits`]). Inline column constraints can't be `NOT VALID`.
    ///
    /// [`processCASbits`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L19447
    pub fn added_constraints(&self) -> impl Iterator<Item = (&Constraint, ConstraintSite)> {
        self.alter_table_commands().flat_map(|cmd| {
            let (table, column) = match (cmd.subtype(), cmd.definition()) {
                (AlterTableType::AtAddConstraint, Some(NodeEnum::Constraint(constraint))) => {
                    (Some(constraint.as_ref()), None)
                }
                (AlterTableType::AtAddColumn, Some(NodeEnum::ColumnDef(col_def))) => {
                    (None, Some(col_def.as_ref()))
                }
                _ => (None, None),
            };
            let table = table.map(|constraint| (constraint, ConstraintSite::Table));
            let column = column
                .into_iter()
                .flat_map(|col_def| Constraint::in_list(&col_def.constraints))
                .map(|constraint| (constraint, ConstraintSite::Column));
            table.into_iter().chain(column)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ConstraintSite;
    use crate::{NodeEnum, protobuf::ConstrType};

    fn parse(sql: &str) -> NodeEnum {
        crate::parse(sql).unwrap().into_root().unwrap()
    }

    #[test]
    fn not_valid_and_not_enforced_constraints_skip_validation() {
        let skips = |sql: &str| {
            let stmt = parse(sql);
            let (constraint, _) = stmt.added_constraints().next().unwrap();
            assert_eq!(constraint.initially_valid, !constraint.skip_validation);
            constraint.skip_validation
        };
        assert!(skips(
            "alter table t add constraint c foreign key (a) references b not valid"
        ));
        assert!(skips(
            "alter table t add constraint c check (a > 0) not valid"
        ));
        assert!(skips(
            "alter table t add constraint c foreign key (a) references b not enforced"
        ));
        assert!(!skips(
            "alter table t add constraint c foreign key (a) references b"
        ));
        assert!(!skips("alter table t add column a int references b"));
    }

    #[test]
    fn added_constraints_are_those_of_add_constraint_and_add_column() {
        let stmt = parse(
            "alter table t add column a int references b not null, add constraint c check (a > 0), alter column d set not null, add unique (e)",
        );
        let added: Vec<_> = stmt
            .added_constraints()
            .map(|(constraint, site)| (constraint.contype(), site))
            .collect();
        assert_eq!(
            added,
            [
                (ConstrType::ConstrForeign, ConstraintSite::Column),
                (ConstrType::ConstrNotnull, ConstraintSite::Column),
                (ConstrType::ConstrCheck, ConstraintSite::Table),
                (ConstrType::ConstrUnique, ConstraintSite::Table),
            ]
        );
        assert_eq!(
            parse("create table t (a int unique)")
                .added_constraints()
                .count(),
            0
        );
    }
}
