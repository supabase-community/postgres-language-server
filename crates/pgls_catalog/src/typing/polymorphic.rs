use crate::typing::{Decision, Selection, Type, TypeId, select_common_type};
use crate::{CatalogView, lookup::Lookup};

fn poly_id(c: &dyn CatalogView, name: &str) -> Option<TypeId> {
    match c.type_(Some("pg_catalog"), name, &[]) {
        Lookup::Found(info) => info.id,
        _ => None,
    }
}
fn is(id: &Option<TypeId>, candidate: &TypeId) -> bool {
    id.as_ref() == Some(candidate)
}
fn base_type(c: &dyn CatalogView, id: &TypeId) -> Decision<TypeId> {
    match c.type_by_id(id) {
        Lookup::Found(info) => Decision::Known(info.base.unwrap_or_else(|| id.clone())),
        _ => Decision::Unknown,
    }
}

/// Port of `parse_coerce.c:check_generic_type_consistency`.
pub fn check_generic_type_consistency(
    c: &dyn CatalogView,
    actual: &[Type],
    declared: &[TypeId],
) -> Decision<bool> {
    if actual.len() != declared.len() {
        return Decision::Known(false);
    }
    let names = [
        "any",
        "anyelement",
        "anyarray",
        "anynonarray",
        "anyenum",
        "anycompatible",
        "anycompatiblearray",
        "anycompatiblenonarray",
        "anyrange",
        "anycompatiblerange",
        "anymultirange",
    ];
    let ids: Vec<_> = names.iter().map(|name| poly_id(c, name)).collect();
    let mut elem = None;
    let mut array = None;
    let mut compatible = Vec::new();
    let mut unknown = false;
    for (actual, declared) in actual.iter().zip(declared) {
        if is(&ids[0], declared) {
            continue;
        }
        if matches!(actual, Type::UnknownLiteral) {
            unknown = true;
            continue;
        }
        let Type::Named(input) = actual else {
            return Decision::Unknown;
        };
        if is(&ids[8], declared) || is(&ids[9], declared) || is(&ids[10], declared) {
            return Decision::Unknown;
        }
        if is(&ids[2], declared) || is(&ids[6], declared) {
            let flattened = match base_type(c, input) {
                Decision::Known(id) => id,
                Decision::Unknown => return Decision::Unknown,
            };
            let element = match c.type_by_id(&flattened) {
                Lookup::Found(info) => info.element,
                _ => return Decision::Unknown,
            };
            let Some(element) = element else {
                return Decision::Known(false);
            };
            if is(&ids[2], declared) {
                if let Some(previous) = &array {
                    if previous != &flattened {
                        return Decision::Known(false);
                    }
                }
                array = Some(flattened);
            } else {
                compatible.push(Type::Named(element));
            }
        } else if is(&ids[5], declared) || is(&ids[7], declared) {
            if is(&ids[7], declared) {
                match c.type_by_id(input) {
                    Lookup::Found(info) if info.element.is_some() => return Decision::Known(false),
                    Lookup::Found(_) => {}
                    _ => return Decision::Unknown,
                }
            }
            compatible.push(Type::Named(input.clone()));
        } else if is(&ids[1], declared) || is(&ids[3], declared) || is(&ids[4], declared) {
            if is(&ids[4], declared) {
                match c.type_by_id(input) {
                    Lookup::Found(info) if info.kind == Some(crate::typing::TypeKind::Enum) => {}
                    Lookup::Found(_) => return Decision::Known(false),
                    _ => return Decision::Unknown,
                }
            }
            if is(&ids[3], declared) {
                match c.type_by_id(input) {
                    Lookup::Found(info) if info.element.is_some() => return Decision::Known(false),
                    Lookup::Found(_) => {}
                    _ => return Decision::Unknown,
                }
            }
            if let Some(previous) = &elem {
                if previous != input {
                    return Decision::Known(false);
                }
            }
            elem = Some(input.clone());
        }
    }
    if let (Some(element), Some(array)) = (&elem, &array) {
        match c.type_by_id(array) {
            Lookup::Found(info) if info.element.as_ref() == Some(element) => {}
            Lookup::Found(_) => return Decision::Known(false),
            _ => return Decision::Unknown,
        }
    }
    if !compatible.is_empty() {
        match select_common_type(c, &compatible) {
            Selection::Match(_) => {}
            Selection::NoMatch => return Decision::Known(false),
            Selection::Ambiguous | Selection::Unknown => return Decision::Unknown,
        }
    }
    if unknown && elem.is_none() && array.is_none() && compatible.is_empty() {
        return Decision::Unknown;
    }
    Decision::Known(true)
}

/// Port of `parse_coerce.c:enforce_generic_type_consistency` for supported families.
pub fn resolve_polymorphic_result(
    c: &dyn CatalogView,
    actual: &[Type],
    declared: &[TypeId],
    result: &TypeId,
) -> Decision<Option<Type>> {
    if matches!(
        check_generic_type_consistency(c, actual, declared),
        Decision::Unknown
    ) {
        return Decision::Unknown;
    }
    let names = [
        "any",
        "anyelement",
        "anyarray",
        "anynonarray",
        "anyenum",
        "anycompatible",
        "anycompatiblearray",
        "anycompatiblenonarray",
        "anyrange",
        "anycompatiblerange",
        "anymultirange",
    ];
    let ids: Vec<_> = names.iter().map(|name| poly_id(c, name)).collect();
    if ids.iter().skip(8).any(|id| id.as_ref() == Some(result)) {
        return Decision::Unknown;
    }
    if ids[0].as_ref() == Some(result) {
        return Decision::Known(None);
    }
    if !ids.iter().any(|id| id.as_ref() == Some(result)) {
        return Decision::Known(Some(Type::Named(result.clone())));
    }
    let mut elem = None;
    let mut compatible = Vec::new();
    for (actual, declared) in actual.iter().zip(declared) {
        if is(&ids[0], declared) || matches!(actual, Type::UnknownLiteral) {
            continue;
        }
        let Type::Named(id) = actual else {
            return Decision::Unknown;
        };
        if is(&ids[2], declared) || is(&ids[6], declared) {
            let id = match base_type(c, id) {
                Decision::Known(id) => id,
                Decision::Unknown => return Decision::Unknown,
            };
            if is(&ids[2], result) {
                return Decision::Known(Some(Type::Named(id)));
            }
            let element = match c.type_by_id(&id) {
                Lookup::Found(info) => info.element,
                _ => return Decision::Unknown,
            };
            let Some(element) = element else {
                return Decision::Unknown;
            };
            if is(&ids[6], declared) {
                compatible.push(Type::Named(element));
            } else {
                elem.get_or_insert(element);
            }
        } else if is(&ids[1], declared)
            || is(&ids[3], declared)
            || is(&ids[4], declared)
            || is(&ids[5], declared)
            || is(&ids[7], declared)
        {
            if is(&ids[5], declared) || is(&ids[7], declared) {
                compatible.push(Type::Named(id.clone()));
            } else {
                elem.get_or_insert(id.clone());
            }
        }
    }
    if is(&ids[5], result) || is(&ids[6], result) || is(&ids[7], result) {
        let common = match select_common_type(c, &compatible) {
            Selection::Match(Type::Named(id)) => id,
            _ => return Decision::Unknown,
        };
        if is(&ids[6], result) {
            return match c.type_by_id(&common) {
                Lookup::Found(info) => info
                    .array
                    .map(|id| Decision::Known(Some(Type::Named(id))))
                    .unwrap_or(Decision::Unknown),
                _ => Decision::Unknown,
            };
        }
        return Decision::Known(Some(Type::Named(common)));
    }
    match elem {
        Some(id) => Decision::Known(Some(Type::Named(id))),
        None => Decision::Unknown,
    }
}

/// Compatibility shim for callers of the initial typing API.
pub fn resolve_polymorphic(result: &TypeId, actual: &[Type]) -> Decision<Type> {
    if actual.len() == 1 {
        Decision::Known(actual[0].clone())
    } else {
        let _ = result;
        Decision::Unknown
    }
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use super::{check_generic_type_consistency, resolve_polymorphic_result};
    use crate::lookup::CatalogView;
    use crate::typing::{Decision, Type};
    use crate::{Catalog, CatalogBase, Snapshot};
    use sqlx::PgPool;
    use std::sync::Arc;

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn resolves_builtin_polymorphic_families(test_db: PgPool) {
        let snapshot = Snapshot::load(&test_db).await.expect("snapshot");
        let catalog = Catalog::new(Some(Arc::new(CatalogBase::new(Arc::new(snapshot)))));
        let path = vec!["public".to_owned()];
        let id = |name: &str| {
            catalog
                .type_(Some("pg_catalog"), name, &path)
                .found()
                .unwrap()
                .id
                .unwrap()
        };
        let anyelement = id("anyelement");
        let int4 = id("int4");
        let int8 = id("int8");
        assert_eq!(
            check_generic_type_consistency(
                &catalog,
                &[Type::Named(int4.clone())],
                &[anyelement.clone()]
            ),
            Decision::Known(true)
        );
        assert_eq!(
            resolve_polymorphic_result(
                &catalog,
                &[Type::Named(int4.clone())],
                &[anyelement.clone()],
                &anyelement
            ),
            Decision::Known(Some(Type::Named(int4)))
        );
        let _ = int8;
    }
}
