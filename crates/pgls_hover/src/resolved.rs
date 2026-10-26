//! Hover for names that the resolver resolved to exactly one object.

use pgls_catalog::resolve::{ColumnSource, Reference};
use pgls_catalog::{Function, FunctionInfo, Snapshot, TypeId};
use pgls_text_size::{TextRange, TextSize};

use crate::{hoverables::Hoverable, hovered_node::HoveredNode};

/// The object that the hovered name resolves to, if one of `references` is of its kind and the
/// object is in the snapshot. `position` is the cursor in `sql`, the statement.
pub(crate) fn resolved_hoverable<'a>(
    node: &HoveredNode,
    references: &[Reference],
    snapshot: &'a Snapshot,
    sql: &str,
    position: TextSize,
) -> Option<Hoverable<'a>> {
    references
        .iter()
        .find_map(|reference| match (node, reference) {
            (HoveredNode::Function(_), Reference::Function { function, .. }) => {
                find_function(snapshot, function).map(Hoverable::Function)
            }
            // The span of a column reference includes its qualifiers, like `u` in `u.name`.
            (
                HoveredNode::Column(_),
                Reference::Column {
                    source,
                    column,
                    span,
                    ..
                },
            ) if span.is_some_and(|span| is_in_last_part(sql, span, position)) => match source {
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

/// Whether `position` is in the last part of the qualified name at `span`, like `name` in
/// `u.name`, rather than in a qualifier.
fn is_in_last_part(sql: &str, span: TextRange, position: TextSize) -> bool {
    let Some(text) = sql.get(std::ops::Range::<usize>::from(span)) else {
        return false;
    };
    let mut quoted = false;
    let mut last_dot = None;
    for (offset, byte) in text.bytes().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b'.' if !quoted => last_dot = Some(offset),
            _ => {}
        }
    }
    let offset = usize::from(position.checked_sub(span.start()).unwrap_or_default());
    last_dot.is_none_or(|dot| offset > dot)
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
