//! AST matching that several lint rules share.

use pgls_query::NodeEnum;
use pgls_query::protobuf::{AlterTableCmd, AlterTableType, ColumnDef, Node, TypeName};

/// The subcommands of an `ALTER TABLE` statement. Other statements have none.
pub(crate) fn alter_table_cmds(stmt: &NodeEnum) -> impl Iterator<Item = &AlterTableCmd> {
    let cmds = match stmt {
        NodeEnum::AlterTableStmt(stmt) => stmt.cmds.as_slice(),
        _ => &[],
    };
    cmds.iter().filter_map(|cmd| match &cmd.node {
        Some(NodeEnum::AlterTableCmd(cmd)) => Some(cmd.as_ref()),
        _ => None,
    })
}

/// The definition of an `ALTER TABLE` subcommand, e.g. the column of `ADD COLUMN` or the
/// constraint of `ADD CONSTRAINT`.
pub(crate) fn cmd_def(cmd: &AlterTableCmd) -> Option<&NodeEnum> {
    cmd.def.as_ref()?.node.as_ref()
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

/// A list of name nodes as written in a diagnostic, e.g. `myschema.mood`.
pub(crate) fn dotted_name(names: &[Node]) -> String {
    names
        .iter()
        .filter_map(|name| match &name.node {
            Some(NodeEnum::String(name)) => Some(name.sval.as_str()),
            _ => None,
        })
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
/// `pg_catalog` comes first on the search path.
pub(crate) fn builtin_type_name(type_name: &TypeName) -> Option<&str> {
    let names: Vec<&str> = type_name
        .names
        .iter()
        .map(|name| match &name.node {
            Some(NodeEnum::String(name)) => Some(name.sval.as_str()),
            _ => None,
        })
        .collect::<Option<_>>()?;
    match names.as_slice() {
        [name] | ["pg_catalog", name] => Some(name),
        _ => None,
    }
}

/// The object type as a noun with its article, e.g. "a view", for diagnostic messages.
pub(crate) fn object_type_noun(object_type: pgls_query::protobuf::ObjectType) -> &'static str {
    use pgls_query::protobuf::ObjectType;
    match object_type {
        ObjectType::ObjectAggregate => "an aggregate",
        ObjectType::ObjectAttribute => "an attribute",
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
        ObjectType::ObjectTrigger => "a trigger",
        ObjectType::ObjectTsconfiguration => "a text search configuration",
        ObjectType::ObjectTsdictionary => "a text search dictionary",
        ObjectType::ObjectTsparser => "a text search parser",
        ObjectType::ObjectTstemplate => "a text search template",
        ObjectType::ObjectType => "a type",
        ObjectType::ObjectView => "a view",
        _ => "an object",
    }
}
