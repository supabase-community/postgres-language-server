//! AST matching that several lint rules share.

use pgls_query::NodeEnum;
use pgls_query::protobuf::{
    AlterDomainStmt, AlterTableCmd, AlterTableType, ColumnDef, Constraint, Node, RangeVar, TypeName,
};
use pgls_query_ext::utils::{QualifiedName, string_value};
pub(crate) use pgls_query_ext::utils::{cmd_def, detaches_concurrently};

/// The subcommands of an `ALTER TABLE` statement. Other statements have none.
pub(crate) fn alter_table_cmds(stmt: &NodeEnum) -> impl Iterator<Item = &AlterTableCmd> {
    let stmt = match stmt {
        NodeEnum::AlterTableStmt(stmt) => Some(stmt),
        _ => None,
    };
    stmt.into_iter()
        .flat_map(pgls_query_ext::utils::alter_table_cmds)
}

/// The table that an `ALTER TABLE` changes. Other statements change none.
pub(crate) fn altered_relation(stmt: &NodeEnum) -> Option<&RangeVar> {
    match stmt {
        NodeEnum::AlterTableStmt(stmt) => stmt.relation.as_ref(),
        _ => None,
    }
}

/// The columns that a statement adds or retypes: the columns of `CREATE TABLE`, and of
/// `ALTER TABLE ... ADD COLUMN` and `ALTER COLUMN ... TYPE`.
pub(crate) fn column_defs(stmt: &NodeEnum) -> impl Iterator<Item = &ColumnDef> {
    let table_elts = match stmt {
        NodeEnum::CreateStmt(stmt) => stmt.table_elts.as_slice(),
        _ => &[],
    };
    let created = table_elts.iter().filter_map(|elt| match &elt.node {
        Some(NodeEnum::ColumnDef(col_def)) => Some(col_def.as_ref()),
        _ => None,
    });
    let altered = alter_table_cmds(stmt)
        .filter(|cmd| {
            matches!(
                cmd.subtype(),
                AlterTableType::AtAddColumn | AlterTableType::AtAlterColumnType
            )
        })
        .filter_map(|cmd| match cmd_def(cmd) {
            Some(NodeEnum::ColumnDef(col_def)) => Some(col_def.as_ref()),
            _ => None,
        });
    created.chain(altered)
}

/// The constraint nodes of a list, e.g. the constraints of a `CREATE DOMAIN`.
pub(crate) fn constraints(nodes: &[Node]) -> impl Iterator<Item = &Constraint> {
    nodes.iter().filter_map(|node| match &node.node {
        Some(NodeEnum::Constraint(constraint)) => Some(constraint.as_ref()),
        _ => None,
    })
}

/// The constraints of a column definition, including `DEFAULT`, `NULL`, `NOT NULL` and
/// `GENERATED`, which are constraint nodes too.
pub(crate) fn column_constraints(col_def: &ColumnDef) -> impl Iterator<Item = &Constraint> {
    constraints(&col_def.constraints)
}

/// Whether Postgres adds the constraint without checking the existing rows: `NOT VALID`, and
/// `NOT ENFORCED`, which implies it. The parser sets `skip_validation` and makes
/// `initially_valid` its negation ([`processCASbits`]); Postgres decides on the scan with
/// `skip_validation`. Inline column constraints can't be `NOT VALID`.
///
/// [`processCASbits`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L19447
pub(crate) fn skips_validation(constraint: &Constraint) -> bool {
    constraint.skip_validation
}

/// Where an `ALTER TABLE` adds a constraint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConstraintSite {
    /// `ADD CONSTRAINT`, or `ADD` with a table constraint.
    Table,
    /// Inline on a column of `ADD COLUMN`, e.g. `ADD COLUMN a int REFERENCES b`.
    Column,
}

/// The constraints that an `ALTER TABLE` adds, in the order of the subcommands: those of
/// `ADD CONSTRAINT`, and the inline constraints of `ADD COLUMN`. Other statements add none.
pub(crate) fn added_constraints(
    stmt: &NodeEnum,
) -> impl Iterator<Item = (&Constraint, ConstraintSite)> {
    alter_table_cmds(stmt).flat_map(|cmd| {
        let (table, column) = match (cmd.subtype(), cmd_def(cmd)) {
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
            .flat_map(column_constraints)
            .map(|constraint| (constraint, ConstraintSite::Column));
        table.into_iter().chain(column)
    })
}

/// What an `ALTER DOMAIN` does. The parser stores it as a letter in `subtype`, see
/// [`AlterDomainStmt`].
///
/// [`AlterDomainStmt`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/gram.y#L11630
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AlterDomainKind {
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

impl AlterDomainKind {
    pub(crate) fn of(stmt: &AlterDomainStmt) -> Option<Self> {
        Some(match stmt.subtype.as_str() {
            "T" if stmt.def.is_some() => Self::SetDefault,
            "T" => Self::DropDefault,
            "N" => Self::DropNotNull,
            "O" => Self::SetNotNull,
            "C" => Self::AddConstraint,
            "X" => Self::DropConstraint,
            "V" => Self::ValidateConstraint,
            _ => return None,
        })
    }
}

/// A list of name nodes as written in a diagnostic, e.g. `myschema.mood`.
pub(crate) fn dotted_name(names: &[Node]) -> String {
    names
        .iter()
        .filter_map(string_value)
        .collect::<Vec<_>>()
        .join(".")
}

/// A name with its schema, if it has one, e.g. `myschema.mood`.
pub(crate) fn qualified_name(schema: Option<&str>, name: &str) -> String {
    match schema {
        Some(schema) => format!("{schema}.{name}"),
        None => name.to_owned(),
    }
}

/// The name of a built-in type, e.g. `varchar` for `varchar(10)` and `pg_catalog.varchar`, but
/// not for `myschema.varchar`. Unqualified names are built-in types if they exist, since
/// `pg_catalog` comes first on the search path. Postgres only accepts the current database as
/// the catalog of a three-part name like `mydb.pg_catalog.varchar`, and then ignores it, so we
/// ignore it too.
pub(crate) fn builtin_type_name(type_name: &TypeName) -> Option<&str> {
    match pgls_query_ext::utils::qualified_name(&type_name.names)? {
        QualifiedName {
            schema: None | Some("pg_catalog"),
            name,
            ..
        } => Some(name),
        _ => None,
    }
}

/// The object type as a noun with its article, e.g. "a view", for diagnostic messages.
pub(crate) fn object_type_noun(object_type: pgls_query::protobuf::ObjectType) -> &'static str {
    use pgls_query::protobuf::ObjectType;
    match object_type {
        ObjectType::ObjectAccessMethod => "an access method",
        ObjectType::ObjectAggregate => "an aggregate",
        ObjectType::ObjectAttribute => "an attribute",
        ObjectType::ObjectCast => "a cast",
        ObjectType::ObjectCollation => "a collation",
        ObjectType::ObjectColumn => "a column",
        ObjectType::ObjectConversion => "a conversion",
        ObjectType::ObjectDatabase => "a database",
        ObjectType::ObjectDomain => "a domain",
        ObjectType::ObjectDomconstraint | ObjectType::ObjectTabconstraint => "a constraint",
        ObjectType::ObjectEventTrigger => "an event trigger",
        ObjectType::ObjectExtension => "an extension",
        ObjectType::ObjectFdw => "a foreign-data wrapper",
        ObjectType::ObjectForeignServer => "a foreign server",
        ObjectType::ObjectForeignTable => "a foreign table",
        ObjectType::ObjectFunction => "a function",
        ObjectType::ObjectIndex => "an index",
        ObjectType::ObjectLanguage => "a language",
        ObjectType::ObjectMatview => "a materialized view",
        ObjectType::ObjectOpclass => "an operator class",
        ObjectType::ObjectOperator => "an operator",
        ObjectType::ObjectOpfamily => "an operator family",
        ObjectType::ObjectPolicy => "a policy",
        ObjectType::ObjectProcedure => "a procedure",
        ObjectType::ObjectPublication => "a publication",
        ObjectType::ObjectRole => "a role",
        ObjectType::ObjectRoutine => "a routine",
        ObjectType::ObjectRule => "a rule",
        ObjectType::ObjectSchema => "a schema",
        ObjectType::ObjectSequence => "a sequence",
        ObjectType::ObjectStatisticExt => "a statistics object",
        ObjectType::ObjectSubscription => "a subscription",
        ObjectType::ObjectTable => "a table",
        ObjectType::ObjectTablespace => "a tablespace",
        ObjectType::ObjectTransform => "a transform",
        ObjectType::ObjectTrigger => "a trigger",
        ObjectType::ObjectTsconfiguration => "a text search configuration",
        ObjectType::ObjectTsdictionary => "a text search dictionary",
        ObjectType::ObjectTsparser => "a text search parser",
        ObjectType::ObjectTstemplate => "a text search template",
        ObjectType::ObjectType => "a type",
        ObjectType::ObjectUserMapping => "a user mapping",
        ObjectType::ObjectView => "a view",
        _ => "an object",
    }
}

#[cfg(test)]
mod tests {
    use pgls_query::NodeEnum;
    use pgls_query::protobuf::ConstrType;

    use super::{AlterDomainKind, ConstraintSite, added_constraints, skips_validation};

    fn parse(sql: &str) -> NodeEnum {
        pgls_query::parse(sql).unwrap().into_root().unwrap()
    }

    #[test]
    fn not_valid_and_not_enforced_constraints_skip_validation() {
        let skips = |sql: &str| {
            let stmt = parse(sql);
            let (constraint, _) = added_constraints(&stmt).next().unwrap();
            assert_eq!(constraint.initially_valid, !constraint.skip_validation);
            skips_validation(constraint)
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
        let added: Vec<_> = added_constraints(&stmt)
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
            added_constraints(&parse("create table t (a int unique)")).count(),
            0
        );
    }

    #[test]
    fn alter_domain_kinds() {
        let kind = |sql: &str| match parse(sql) {
            NodeEnum::AlterDomainStmt(stmt) => AlterDomainKind::of(&stmt),
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
