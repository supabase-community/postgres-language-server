//! Which type several inputs agree on: the common type of `UNION`, `CASE`, `COALESCE`, and
//! `VALUES` columns, and the actual types behind polymorphic arguments and results.

use crate::typing::coerce::can_coerce;
use crate::typing::types::{base_type, type_category};
use crate::typing::{CoercionContext, Decision, Selection, Type, TypeId};
use crate::{CatalogView, Lookup};

// ----- common type -----

fn text_type(c: &dyn CatalogView) -> Decision<Type> {
    match c.type_(Some("pg_catalog"), "text", &[]) {
        Lookup::Found(info) => match info.id {
            Some(id) => Decision::Known(Type::Named(id)),
            None => Decision::Unknown,
        },
        Lookup::Missing | Lookup::Unknown => Decision::Unknown,
    }
}

/// The type `UNION`, `CASE`, `COALESCE`, `VALUES` and friends resolve their inputs to. Port of
/// [`select_common_type`], including the implicit-coercibility checks of [`verify_common_type`]
/// and [`coerce_to_common_type`].
///
/// [`select_common_type`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L1345
/// [`verify_common_type`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L1609
/// [`coerce_to_common_type`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L1575
pub fn select_common_type(c: &dyn CatalogView, types: &[Type]) -> Selection<Type> {
    let Some(first) = types.first() else {
        return Selection::Unknown;
    };
    if !matches!(first, Type::UnknownLiteral) && types.iter().all(|ty| ty == first) {
        return Selection::Match(first.clone());
    }
    // Anonymous rows have no catalog identity or modeled row coercion.
    if types.iter().any(|t| matches!(t, Type::Record(_))) {
        return Selection::Unknown;
    }

    let mut selected: Option<TypeId> = None;
    let mut selected_category = None;
    let mut selected_preferred = false;
    for ty in types {
        let Type::Named(id) = ty else { continue };
        let base = match base_type(c, id) {
            Decision::Known(base) => base,
            Decision::Unknown => return Selection::Unknown,
        };
        if selected.as_ref() == Some(&base) {
            continue;
        }
        let (category, preferred) = match type_category(c, &base) {
            Decision::Known(category) => category,
            Decision::Unknown => return Selection::Unknown,
        };
        if let Some(current) = selected.as_ref() {
            if category != selected_category.unwrap_or(category) {
                return Selection::NoMatch;
            }
            if !selected_preferred {
                match (
                    can_coerce(
                        c,
                        &Type::Named(current.clone()),
                        &base,
                        CoercionContext::Implicit,
                    ),
                    can_coerce(
                        c,
                        &Type::Named(base.clone()),
                        current,
                        CoercionContext::Implicit,
                    ),
                ) {
                    (Decision::Known(true), Decision::Known(false)) => {
                        selected = Some(base);
                        selected_category = Some(category);
                        selected_preferred = preferred;
                    }
                    (Decision::Known(_), Decision::Known(_)) => {}
                    _ => return Selection::Unknown,
                }
            }
        } else {
            selected = Some(base);
            selected_category = Some(category);
            selected_preferred = preferred;
        }
    }

    let result = match selected {
        Some(id) => Type::Named(id),
        None => match text_type(c) {
            Decision::Known(ty) => ty,
            Decision::Unknown => return Selection::Unknown,
        },
    };
    let Type::Named(target) = &result else {
        return Selection::Unknown;
    };
    for ty in types {
        match can_coerce(c, ty, target, CoercionContext::Implicit) {
            Decision::Known(true) => {}
            Decision::Known(false) => return Selection::NoMatch,
            Decision::Unknown => return Selection::Unknown,
        }
    }
    Selection::Match(result)
}

// ----- polymorphic types -----

fn poly_id(c: &dyn CatalogView, name: &str) -> Option<TypeId> {
    match c.type_(Some("pg_catalog"), name, &[]) {
        Lookup::Found(info) => info.id,
        _ => None,
    }
}
fn is(id: &Option<TypeId>, candidate: &TypeId) -> bool {
    id.as_ref() == Some(candidate)
}

/// Whether the actual argument types fit the polymorphic declared types of a candidate. Port
/// of [`check_generic_type_consistency`].
///
/// [`check_generic_type_consistency`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L1740
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
                crate::TypeKind::Multirange
            } else {
                crate::TypeKind::Range
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
                if let Some(previous) = &array
                    && previous != &flattened
                {
                    return Decision::Known(false);
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
                    Lookup::Found(info) if info.kind == Some(crate::TypeKind::Enum) => {}
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
            if let Some(previous) = &elem
                && previous != input
            {
                return Decision::Known(false);
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
/// `"any"` result. Port of [`enforce_generic_type_consistency`], without range types.
///
/// [`enforce_generic_type_consistency`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L2134
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

/// Whether the inputs of a function determine its polymorphic result type. Port of
/// [`check_valid_polymorphic_signature`]. Unknown when a type is unknown.
///
/// [`check_valid_polymorphic_signature`]: https://github.com/postgres/postgres/blob/REL_18_6/src/backend/parser/parse_coerce.c#L2878
pub fn is_valid_polymorphic_signature(
    c: &dyn CatalogView,
    result: &TypeId,
    inputs: &[Option<TypeId>],
) -> Decision<bool> {
    let Some(result) = polymorphic_name(c, result) else {
        return Decision::Unknown;
    };
    let Some(inputs) = inputs
        .iter()
        .map(|input| polymorphic_name(c, input.as_ref()?))
        .collect::<Option<Vec<_>>>()
    else {
        return Decision::Unknown;
    };
    let Some(result) = result else {
        return Decision::Known(true);
    };
    let family1 = |name: &str| {
        matches!(
            name,
            "anyelement" | "anyarray" | "anynonarray" | "anyenum" | "anyrange" | "anymultirange"
        )
    };
    let accepts: &dyn Fn(&str) -> bool = match result {
        "anyrange" | "anymultirange" => &|name| matches!(name, "anyrange" | "anymultirange"),
        "anycompatiblerange" | "anycompatiblemultirange" => {
            &|name| matches!(name, "anycompatiblerange" | "anycompatiblemultirange")
        }
        name if family1(name) => &family1,
        _ => &|name: &str| name.starts_with("anycompatible"),
    };
    Decision::Known(inputs.iter().flatten().any(|input| accepts(input)))
}

/// The name of a type if it is polymorphic, `Some(None)` if it is not, and `None` if the type
/// is unknown.
fn polymorphic_name(c: &dyn CatalogView, id: &TypeId) -> Option<Option<&'static str>> {
    const POLYMORPHIC: &[&str] = &[
        "anyelement",
        "anyarray",
        "anynonarray",
        "anyenum",
        "anyrange",
        "anymultirange",
        "anycompatible",
        "anycompatiblearray",
        "anycompatiblenonarray",
        "anycompatiblerange",
        "anycompatiblemultirange",
    ];
    let info = c.type_by_id(id).found()?;
    if info.schema != "pg_catalog" {
        return Some(None);
    }
    Some(POLYMORPHIC.iter().copied().find(|name| *name == info.name))
}

#[cfg(all(test, feature = "db"))]
mod common_type_tests {
    use super::*;
    use crate::{Catalog, CatalogBase, CatalogView, Lookup, Snapshot};
    use sqlx::{Executor, PgPool};
    use std::sync::Arc;

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn postgres_common_types(test_db: PgPool) {
        test_db
            .execute("CREATE DOMAIN public.typing_int_domain AS int4")
            .await
            .unwrap();
        let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
        let catalog = Catalog::new(Some(Arc::new(CatalogBase::new(snapshot))));
        assert_eq!(
            select_common_type(
                &catalog,
                &[
                    Type::Named(TypeId::Snapshot(23)),
                    Type::Named(TypeId::Snapshot(20))
                ]
            ),
            Selection::Match(Type::Named(TypeId::Snapshot(20)))
        );
        assert_eq!(
            select_common_type(&catalog, &[Type::UnknownLiteral, Type::UnknownLiteral]),
            Selection::Match(Type::Named(TypeId::Snapshot(25)))
        );
        assert_eq!(
            select_common_type(
                &catalog,
                &[
                    Type::Named(TypeId::Snapshot(23)),
                    Type::Named(TypeId::Snapshot(1700))
                ]
            ),
            Selection::Match(Type::Named(TypeId::Snapshot(1700)))
        );
        assert_eq!(
            select_common_type(
                &catalog,
                &[Type::Named(TypeId::Snapshot(23)), Type::UnknownLiteral]
            ),
            Selection::Match(Type::Named(TypeId::Snapshot(23)))
        );
        assert_eq!(
            select_common_type(
                &catalog,
                &[
                    Type::Named(TypeId::Snapshot(23)),
                    Type::Named(TypeId::Snapshot(25))
                ]
            ),
            Selection::NoMatch
        );
        assert_eq!(
            select_common_type(
                &catalog,
                &[
                    Type::Named(TypeId::Snapshot(1700)),
                    Type::Named(TypeId::Snapshot(23))
                ]
            ),
            Selection::Match(Type::Named(TypeId::Snapshot(1700)))
        );
        let domain = match catalog.type_(Some("public"), "typing_int_domain", &[]) {
            Lookup::Found(info) => info.id.unwrap(),
            other => panic!("expected test domain in snapshot, got {other:?}"),
        };
        assert_eq!(
            select_common_type(
                &catalog,
                &[Type::Named(domain), Type::Named(TypeId::Snapshot(23))]
            ),
            Selection::Match(Type::Named(TypeId::Snapshot(23)))
        );
        assert_eq!(
            select_common_type(
                &catalog,
                &[
                    Type::Named(TypeId::Snapshot(25)),
                    Type::Named(TypeId::Snapshot(1043))
                ]
            ),
            Selection::Match(Type::Named(TypeId::Snapshot(25)))
        );
    }
}

#[cfg(all(test, feature = "db"))]
mod polymorphic_tests {
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
                std::slice::from_ref(&anyelement)
            ),
            Decision::Known(true)
        );
        assert_eq!(
            resolve_polymorphic_result(
                &catalog,
                &[Type::Named(int4.clone())],
                std::slice::from_ref(&anyelement),
                &anyelement
            ),
            Decision::Known(Some(Type::Named(int4)))
        );
        let _ = int8;
    }
}
