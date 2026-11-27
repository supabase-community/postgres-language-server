use crate::protobuf::AlterDomainStmt;

/// What an `ALTER DOMAIN` does. The parser stores it as a letter in `subtype`, see
/// [`AlterDomainStmt`].
///
/// [`AlterDomainStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L11630
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlterDomainKind {
    /// `SET DEFAULT` (`T` with a default).
    SetDefault,
    /// `DROP DEFAULT` (`T` without a default).
    DropDefault,
    /// `DROP NOT NULL` (`N`).
    DropNotNull,
    /// `SET NOT NULL` (`O`).
    SetNotNull,
    /// `ADD CONSTRAINT` (`C`).
    AddConstraint,
    /// `DROP CONSTRAINT` (`X`).
    DropConstraint,
    /// `VALIDATE CONSTRAINT` (`V`).
    ValidateConstraint,
}

impl AlterDomainStmt {
    /// What the statement does. `None` for a `subtype` the parser doesn't produce.
    pub fn kind(&self) -> Option<AlterDomainKind> {
        Some(match self.subtype.as_str() {
            "T" if self.def.is_some() => AlterDomainKind::SetDefault,
            "T" => AlterDomainKind::DropDefault,
            "N" => AlterDomainKind::DropNotNull,
            "O" => AlterDomainKind::SetNotNull,
            "C" => AlterDomainKind::AddConstraint,
            "X" => AlterDomainKind::DropConstraint,
            "V" => AlterDomainKind::ValidateConstraint,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::AlterDomainKind;
    use crate::NodeEnum;

    #[test]
    fn alter_domain_kinds() {
        let kind = |sql: &str| match crate::parse(sql).unwrap().into_root().unwrap() {
            NodeEnum::AlterDomainStmt(stmt) => stmt.kind(),
            _ => None,
        };
        assert_eq!(
            kind("alter domain d set default 1"),
            Some(AlterDomainKind::SetDefault)
        );
        assert_eq!(
            kind("alter domain d drop default"),
            Some(AlterDomainKind::DropDefault)
        );
        assert_eq!(
            kind("alter domain d drop not null"),
            Some(AlterDomainKind::DropNotNull)
        );
        assert_eq!(
            kind("alter domain d set not null"),
            Some(AlterDomainKind::SetNotNull)
        );
        assert_eq!(
            kind("alter domain d add check (value > 0)"),
            Some(AlterDomainKind::AddConstraint)
        );
        assert_eq!(
            kind("alter domain d drop constraint c"),
            Some(AlterDomainKind::DropConstraint)
        );
        assert_eq!(
            kind("alter domain d validate constraint c"),
            Some(AlterDomainKind::ValidateConstraint)
        );
    }
}
