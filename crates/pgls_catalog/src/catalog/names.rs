//! Helpers to read names out of the parse tree.

use pgls_query::{NodeEnum, protobuf};

/// The value of a `String` node.
pub(super) fn string_value(node: &protobuf::Node) -> Option<&str> {
    match node.node.as_ref()? {
        NodeEnum::String(value) => Some(&value.sval),
        _ => None,
    }
}

/// All `String` values of a name list, e.g. `["public", "users"]`.
pub(super) fn string_values(nodes: &[protobuf::Node]) -> Vec<String> {
    nodes
        .iter()
        .filter_map(string_value)
        .map(str::to_owned)
        .collect()
}

/// A possibly schema-qualified name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct QualifiedName {
    pub schema: Option<String>,
    pub name: String,
}

impl QualifiedName {
    pub fn schema(&self) -> Option<&str> {
        self.schema.as_deref()
    }
}

/// Reads `name`, `schema.name`, or `catalog.schema.name` from a name list.
pub(super) fn qualified_name(nodes: &[protobuf::Node]) -> Option<QualifiedName> {
    let mut names = string_values(nodes);
    let name = names.pop()?;
    Some(QualifiedName {
        schema: names.pop(),
        name,
    })
}

/// Reads the name of a `RangeVar`.
pub(super) fn range_var_name(range_var: &protobuf::RangeVar) -> QualifiedName {
    QualifiedName {
        schema: (!range_var.schemaname.is_empty()).then(|| range_var.schemaname.clone()),
        name: range_var.relname.clone(),
    }
}

/// The name of a type, e.g. `int4` for `integer`.
pub(super) fn type_name(type_name: &protobuf::TypeName) -> Option<QualifiedName> {
    qualified_name(&type_name.names)
}

/// The unqualified name of a type, as stored in [`crate::ColumnInfo::type_name`].
pub(super) fn type_label(type_name: &protobuf::TypeName) -> Option<String> {
    string_values(&type_name.names).pop()
}
