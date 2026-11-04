//! Helpers to read names out of the parse tree.

use pgls_query::protobuf;

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

/// Reads `name`, `schema.name`, or `catalog.schema.name` from a name list, like
/// [`pgls_query_ext::utils::qualified_name`].
pub(super) fn qualified_name(nodes: &[protobuf::Node]) -> Option<QualifiedName> {
    let name = pgls_query_ext::utils::qualified_name(nodes)?;
    Some(QualifiedName {
        schema: name.schema.map(str::to_owned),
        name: name.name.to_owned(),
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
    qualified_name(&type_name.names).map(|name| name.name)
}
