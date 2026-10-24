//! Hover for names that the resolver resolved to exactly one object.

use pgls_catalog::resolve::{ColumnSource, Reference};
use pgls_catalog::{Function, FunctionInfo, Snapshot, TypeId};

use crate::{hoverables::Hoverable, hovered_node::HoveredNode};

/// The object that the hovered name resolves to, if one of `references` is of its kind and the
/// object is in the snapshot.
pub(crate) fn resolved_hoverable<'a>(
    node: &HoveredNode,
    references: &[Reference],
    snapshot: &'a Snapshot,
) -> Option<Hoverable<'a>> {
    references
        .iter()
        .find_map(|reference| match (node, reference) {
            (HoveredNode::Function(_), Reference::Function { function, .. }) => {
                find_function(snapshot, function).map(Hoverable::Function)
            }
            (HoveredNode::Column(_), Reference::Column { source, column, .. }) => match source {
                ColumnSource::Relation { schema, name } => snapshot
                    .columns
                    .iter()
                    .find(|candidate| {
                        candidate.schema_name == *schema
                            && candidate.table_name == *name
                            && candidate.name == column.name
                    })
                    .map(Hoverable::Column),
                ColumnSource::Cte { .. } | ColumnSource::Subquery { .. } => {
                    Some(Hoverable::DerivedColumn(crate::hoverables::DerivedColumn {
                        source: source.clone(),
                        name: column.name.clone(),
                        type_name: column.type_name.clone(),
                    }))
                }
            },
            (HoveredNode::Table(_), Reference::Relation { relation, .. }) => snapshot
                .tables
                .iter()
                .find(|table| table.schema == relation.schema && table.name == relation.name)
                .map(Hoverable::Table),
            _ => None,
        })
}

/// The function that a call resolves to, for statements whose names the editor grammar doesn't
/// classify, like `CALL`.
pub(crate) fn resolved_function<'a>(
    references: &[Reference],
    snapshot: &'a Snapshot,
) -> Option<Hoverable<'a>> {
    references.iter().find_map(|reference| match reference {
        Reference::Function { function, .. } => {
            find_function(snapshot, function).map(Hoverable::Function)
        }
        _ => None,
    })
}

/// The snapshot function with the schema, name, and input argument types of `function`, if
/// exactly one matches.
fn find_function<'a>(snapshot: &'a Snapshot, function: &FunctionInfo) -> Option<&'a Function> {
    let signature = function.signature.as_ref()?;
    let mut candidates = snapshot.functions.iter().filter(|candidate| {
        if candidate.schema != function.schema || candidate.name != function.name {
            return false;
        }
        let inputs: Vec<_> = candidate
            .args
            .args
            .iter()
            .filter(|arg| matches!(arg.mode.as_str(), "in" | "inout" | "variadic"))
            .collect();
        inputs.len() == signature.arguments.len()
            && inputs
                .iter()
                .zip(&signature.arguments)
                .all(|(arg, parameter)| match &parameter.ty {
                    Some(TypeId::Snapshot(id)) => arg.type_id == *id,
                    // Types created in the file get negative ids in the snapshot.
                    Some(TypeId::File(_)) => arg.type_id < 0,
                    None => true,
                })
    });
    let found = candidates.next()?;
    candidates.next().is_none().then_some(found)
}
