use crate::typing::coerce::base_type;
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
        "anycompatiblemultirange",
    ];
    let ids: Vec<_> = names.iter().map(|name| poly_id(c, name)).collect();
    let mut elem = None;
    let mut array = None;
    let mut compatible = Vec::new();
    for (actual, declared) in actual.iter().zip(declared) {
        if is(&ids[0], declared) {
            continue;
        }
        if matches!(actual, Type::UnknownLiteral) {
            continue;
        }
        let Type::Named(input) = actual else {
            return Decision::Unknown;
        };
        if is(&ids[8], declared)
            || is(&ids[9], declared)
            || is(&ids[10], declared)
            || is(&ids[11], declared)
        {
            // Range bindings aren't modelled, but an input that isn't a range (or multirange)
            // can't match at all.
            let expected = if is(&ids[10], declared) || is(&ids[11], declared) {
                crate::typing::TypeKind::Multirange
            } else {
                crate::typing::TypeKind::Range
            };
            let base = match base_type(c, input) {
                Decision::Known(base) => base,
                Decision::Unknown => return Decision::Unknown,
            };
            return match c.type_by_id(&base) {
                Lookup::Found(info) if info.kind == Some(expected) => Decision::Unknown,
                Lookup::Found(info) if info.kind.is_some() => Decision::Known(false),
                _ => Decision::Unknown,
            };
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
    Decision::Known(true)
}

/// The result type of a call with polymorphic arguments or result: `Known(None)` for an
/// `"any"` result. Port of `parse_coerce.c: enforce_generic_type_consistency`, without ranges.
pub fn resolve_polymorphic_result(
    c: &dyn CatalogView,
    actual: &[Type],
    declared: &[TypeId],
    result: &TypeId,
) -> Decision<Option<Type>> {
    if matches!(
        check_generic_type_consistency(c, actual, declared),
        Decision::Unknown | Decision::Known(false)
    ) {
        return Decision::Unknown;
    }
    let id = |name: &str| poly_id(c, name);
    let [
        any,
        anyelement,
        anyarray,
        anynonarray,
        anyenum,
        anycompatible,
        anycompatiblearray,
        anycompatiblenonarray,
    ] = [
        "any",
        "anyelement",
        "anyarray",
        "anynonarray",
        "anyenum",
        "anycompatible",
        "anycompatiblearray",
        "anycompatiblenonarray",
    ]
    .map(id);
    let ranges = [
        "anyrange",
        "anymultirange",
        "anycompatiblerange",
        "anycompatiblemultirange",
    ]
    .map(id);
    let element_family = [&anyelement, &anynonarray, &anyenum];
    let compatible_family = [&anycompatible, &anycompatiblenonarray];
    if is(&any, result) {
        return Decision::Known(None);
    }
    if ranges.iter().any(|range| is(range, result)) {
        return Decision::Unknown;
    }
    let polymorphic_result = element_family
        .iter()
        .chain(&compatible_family)
        .any(|family| is(family, result))
        || is(&anyarray, result)
        || is(&anycompatiblearray, result);
    if !polymorphic_result {
        return Decision::Known(Some(Type::Named(result.clone())));
    }

    let element_of = |id: &TypeId| match c.type_by_id(id) {
        Lookup::Found(info) => Decision::Known(info.element),
        _ => Decision::Unknown,
    };
    let array_of = |id: &TypeId| match c.type_by_id(id) {
        Lookup::Found(info) => match info.array {
            Some(array) => Decision::Known(Some(Type::Named(array))),
            None => Decision::Unknown,
        },
        _ => Decision::Unknown,
    };

    let mut element: Option<TypeId> = None;
    let mut array: Option<TypeId> = None;
    let mut compatible: Vec<Type> = Vec::new();
    let mut has_compatible = false;
    for (actual, declared) in actual.iter().zip(declared) {
        if ranges.iter().any(|range| is(range, declared)) {
            return Decision::Unknown;
        }
        let is_element = element_family.iter().any(|family| is(family, declared));
        let is_array = is(&anyarray, declared);
        let is_compatible = compatible_family.iter().any(|family| is(family, declared));
        let is_compatible_array = is(&anycompatiblearray, declared);
        if !(is_element || is_array || is_compatible || is_compatible_array) {
            continue;
        }
        has_compatible |= is_compatible || is_compatible_array;
        let input = match actual {
            Type::UnknownLiteral => continue,
            Type::Named(input) => input,
            Type::Record(_) => return Decision::Unknown,
        };
        if is_element {
            if element.as_ref().is_some_and(|element| element != input) {
                return Decision::Unknown;
            }
            element = Some(input.clone());
        } else if is_array {
            let Decision::Known(input) = base_type(c, input) else {
                return Decision::Unknown;
            };
            if array.as_ref().is_some_and(|array| array != &input) {
                return Decision::Unknown;
            }
            array = Some(input);
        } else if is_compatible {
            compatible.push(actual.clone());
        } else {
            let Decision::Known(input) = base_type(c, input) else {
                return Decision::Unknown;
            };
            match element_of(&input) {
                Decision::Known(Some(element)) => compatible.push(Type::Named(element)),
                _ => return Decision::Unknown,
            }
        }
    }
    // The element type follows from the array type.
    if let Some(array) = &array {
        match element_of(array) {
            Decision::Known(Some(array_element)) => match &element {
                Some(element) if element != &array_element => return Decision::Unknown,
                Some(_) => {}
                None => element = Some(array_element),
            },
            _ => return Decision::Unknown,
        }
    }

    if element_family.iter().any(|family| is(family, result)) {
        return match element {
            Some(element) => Decision::Known(Some(Type::Named(element))),
            None => Decision::Unknown,
        };
    }
    if is(&anyarray, result) {
        return match (array, element) {
            (Some(array), _) => Decision::Known(Some(Type::Named(array))),
            (None, Some(element)) => array_of(&element),
            (None, None) => Decision::Unknown,
        };
    }
    // The anycompatible family: the common type, or text if all inputs are unknown.
    let common = if compatible.is_empty() {
        if !has_compatible {
            return Decision::Unknown;
        }
        match c.type_(Some("pg_catalog"), "text", &[]) {
            Lookup::Found(info) => match info.id {
                Some(text) => text,
                None => return Decision::Unknown,
            },
            _ => return Decision::Unknown,
        }
    } else {
        match select_common_type(c, &compatible) {
            Selection::Match(Type::Named(common)) => common,
            _ => return Decision::Unknown,
        }
    };
    if is(&anycompatiblearray, result) {
        array_of(&common)
    } else {
        Decision::Known(Some(Type::Named(common)))
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
