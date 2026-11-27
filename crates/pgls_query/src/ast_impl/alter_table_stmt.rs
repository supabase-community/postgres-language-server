use crate::{
    NodeEnum,
    protobuf::{AlterTableCmd, AlterTableStmt},
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
