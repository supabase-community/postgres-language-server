//! The names of the parse tree.
//!
//! Name lists like `funcname` or `TypeName::names` only ever hold `String` nodes. A list with
//! any other node is not a name, and [`NameParts::of`] and [`QualifiedName::of`] return `None`
//! for it.

use std::fmt;

use pgls_query::protobuf::{Node, RangeVar, TypeName};

/// The most parts a name has: `catalog.schema.table.column`.
const NAME_PARTS_MAX: usize = 4;

/// The parts of a name list, like `["public", "users"]`, borrowed from the parse tree. Unlike
/// [`Node::string_values`], reading them doesn't allocate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameParts<'a> {
    parts: [&'a str; NAME_PARTS_MAX],
    len: usize,
}

impl<'a> NameParts<'a> {
    /// The parts of a list of `String` nodes. `None` if a node is not a string, and for lists
    /// with more parts than a name can have, which Postgres rejects.
    pub fn of(nodes: &'a [Node]) -> Option<Self> {
        if nodes.len() > NAME_PARTS_MAX {
            return None;
        }
        let mut parts = [""; NAME_PARTS_MAX];
        for (part, node) in parts.iter_mut().zip(nodes) {
            *part = node.string_value()?;
        }
        Some(Self {
            parts,
            len: nodes.len(),
        })
    }

    /// The last part of a list of `String` nodes of any length, like the operator of
    /// `OPERATOR(pg_catalog.=)`. `None` if the list is empty or a node is not a string.
    pub fn last(nodes: &'a [Node]) -> Option<&'a str> {
        nodes
            .iter()
            .try_fold(None, |_, node| node.string_value().map(Some))?
    }

    pub fn as_slice(&self) -> &[&'a str] {
        self.parts.get(..self.len).unwrap_or_default()
    }
}

/// A possibly qualified name, like `name`, `schema.name` or `catalog.schema.name`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QualifiedName<'a> {
    pub catalog: Option<&'a str>,
    pub schema: Option<&'a str>,
    pub name: &'a str,
}

impl<'a> QualifiedName<'a> {
    /// Reads a possibly qualified name from a name list, like [`DeconstructQualifiedName`].
    /// `None` for an empty list, more than three parts, which Postgres rejects, and lists with
    /// nodes that are not strings.
    ///
    /// Postgres only accepts a catalog that is the current database, and then ignores it. We
    /// don't know the current database, so callers use the schema and the name.
    ///
    /// [`DeconstructQualifiedName`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/catalog/namespace.c#L3304
    pub fn of(nodes: &'a [Node]) -> Option<Self> {
        let names = NameParts::of(nodes)?;
        let (catalog, schema, name) = match names.as_slice() {
            [name] => (None, None, *name),
            [schema, name] => (None, Some(*schema), *name),
            [catalog, schema, name] => (Some(*catalog), Some(*schema), *name),
            _ => return None,
        };
        Some(Self {
            catalog,
            schema,
            name,
        })
    }

    /// The name of a `RangeVar`, like `t`, `s.t` or `db.s.t`.
    pub fn of_range_var(range_var: &'a RangeVar) -> Self {
        let part = |value: &'a str| (!value.is_empty()).then_some(value);
        Self {
            catalog: part(&range_var.catalogname),
            schema: part(&range_var.schemaname),
            name: &range_var.relname,
        }
    }

    /// The name of a type as the parser stores it, e.g. `pg_catalog.int4` for `integer`. `None`
    /// for names that [`Self::of`] doesn't read.
    pub fn of_type_name(type_name: &'a TypeName) -> Option<Self> {
        Self::of(&type_name.names)
    }
}

/// The name with its qualifiers, like `myschema.mood`, unquoted like in Postgres' messages.
impl fmt::Display for QualifiedName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for qualifier in [self.catalog, self.schema].into_iter().flatten() {
            write!(f, "{qualifier}.")?;
        }
        f.write_str(self.name)
    }
}

#[cfg(test)]
mod tests {
    use pgls_query::NodeEnum;

    use crate::utils::{NameParts, QualifiedName};

    fn create_function(sql: &str) -> pgls_query::protobuf::CreateFunctionStmt {
        match pgls_query::parse(sql).unwrap().into_root().unwrap() {
            NodeEnum::CreateFunctionStmt(stmt) => *stmt,
            _ => panic!("Expected CreateFunctionStmt"),
        }
    }

    #[test]
    fn name_parts() {
        let funcname = |sql: &str| create_function(sql).funcname;

        let names = funcname("create function db.s.f() returns int return 1");
        assert_eq!(
            NameParts::of(&names).map(|parts| parts.as_slice().to_vec()),
            Some(vec!["db", "s", "f"])
        );
        assert_eq!(NameParts::last(&names), Some("f"));
        assert_eq!(NameParts::last(&[]), None);
    }

    #[test]
    fn qualified_names() {
        let funcname = |sql: &str| create_function(sql).funcname;

        assert_eq!(
            QualifiedName::of(&funcname("create function f() returns int return 1")),
            Some(QualifiedName {
                catalog: None,
                schema: None,
                name: "f",
            })
        );
        assert_eq!(
            QualifiedName::of(&funcname("create function db.s.f() returns int return 1")),
            Some(QualifiedName {
                catalog: Some("db"),
                schema: Some("s"),
                name: "f",
            })
        );
        assert_eq!(
            QualifiedName::of_type_name(
                create_function("create function f() returns boolean return true")
                    .return_type
                    .as_ref()
                    .unwrap()
            ),
            Some(QualifiedName {
                catalog: None,
                schema: Some("pg_catalog"),
                name: "bool",
            })
        );
        // Postgres rejects more than three parts.
        assert_eq!(
            QualifiedName::of(&funcname("create function a.db.s.f() returns int return 1")),
            None
        );
        assert_eq!(QualifiedName::of(&[]), None);

        // `t.*` is not a name.
        let NodeEnum::SelectStmt(select) = pgls_query::parse("select t.* from t")
            .unwrap()
            .into_root()
            .unwrap()
        else {
            panic!("Expected SelectStmt");
        };
        let Some(NodeEnum::ResTarget(target)) = &select.target_list[0].node else {
            panic!("Expected ResTarget");
        };
        let Some(NodeEnum::ColumnRef(column)) = target.val.as_ref().unwrap().node.as_ref() else {
            panic!("Expected ColumnRef");
        };
        assert_eq!(QualifiedName::of(&column.fields), None);
    }

    #[test]
    fn range_var_names() {
        let NodeEnum::SelectStmt(select) = pgls_query::parse("select from t, s.t, db.s.t")
            .unwrap()
            .into_root()
            .unwrap()
        else {
            panic!("Expected SelectStmt");
        };
        let names: Vec<_> = select
            .from_clause
            .iter()
            .map(|item| match item.node.as_ref().unwrap() {
                NodeEnum::RangeVar(range_var) => {
                    let name = QualifiedName::of_range_var(range_var);
                    (name.catalog, name.schema, name.name, name.to_string())
                }
                _ => panic!("Expected RangeVar"),
            })
            .collect();
        assert_eq!(
            names,
            [
                (None, None, "t", "t".to_string()),
                (None, Some("s"), "t", "s.t".to_string()),
                (Some("db"), Some("s"), "t", "db.s.t".to_string())
            ]
        );
    }
}
