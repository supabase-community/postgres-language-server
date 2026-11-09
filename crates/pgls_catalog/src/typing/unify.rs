//! Which type several inputs agree on: the common type of `UNION`, `CASE`, `COALESCE`, and
//! `VALUES` columns, and the actual types behind polymorphic arguments and results.

use crate::typing::coerce::can_coerce;
use crate::typing::pseudo::{Polymorphic, PolymorphicFamily, PolymorphicIds};
use crate::typing::types::{array_element, base_type, type_category};
use crate::typing::{CoercionContext, Decision, Selection, Type, TypeId};
use crate::{CatalogView, Lookup};

// ----- common type -----

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

    let Some(target) = selected.or_else(|| c.builtin_type("text")) else {
        return Selection::Unknown;
    };
    for ty in types {
        match can_coerce(c, ty, &target, CoercionContext::Implicit) {
            Decision::Known(true) => {}
            Decision::Known(false) => return Selection::NoMatch,
            Decision::Unknown => return Selection::Unknown,
        }
    }
    Selection::Match(Type::Named(target))
}

// ----- polymorphic types -----

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
    let ids = PolymorphicIds::load(c);
    let mut elem = None;
    let mut array = None;
    let mut compatible = Vec::new();
    for (actual, declared) in actual.iter().zip(declared) {
        let declared = ids.of(declared);
        if declared == Some(Polymorphic::Any) {
            continue;
        }
        if matches!(actual, Type::UnknownLiteral) {
            continue;
        }
        let Type::Named(input) = actual else {
            return Decision::Unknown;
        };
        let Some(declared) = declared else {
            continue;
        };
        match declared {
            Polymorphic::AnyRange
            | Polymorphic::AnyMultirange
            | Polymorphic::AnyCompatibleRange
            | Polymorphic::AnyCompatibleMultirange => {
                // Range bindings aren't modelled, but an input that isn't a range (or multirange)
                // can't match at all.
                let expected = match declared {
                    Polymorphic::AnyMultirange | Polymorphic::AnyCompatibleMultirange => {
                        crate::TypeKind::Multirange
                    }
                    _ => crate::TypeKind::Range,
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
            Polymorphic::AnyArray | Polymorphic::AnyCompatibleArray => {
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
                if declared == Polymorphic::AnyArray {
                    if let Some(previous) = &array
                        && previous != &flattened
                    {
                        return Decision::Known(false);
                    }
                    array = Some(flattened);
                } else {
                    compatible.push(Type::Named(element));
                }
            }
            Polymorphic::AnyCompatible | Polymorphic::AnyCompatibleNonArray => {
                if declared == Polymorphic::AnyCompatibleNonArray {
                    match c.type_by_id(input) {
                        Lookup::Found(info) if info.element.is_some() => {
                            return Decision::Known(false);
                        }
                        Lookup::Found(_) => {}
                        _ => return Decision::Unknown,
                    }
                }
                compatible.push(Type::Named(input.clone()));
            }
            Polymorphic::AnyElement | Polymorphic::AnyNonArray | Polymorphic::AnyEnum => {
                if declared == Polymorphic::AnyEnum {
                    match c.type_by_id(input) {
                        Lookup::Found(info) if info.kind == Some(crate::TypeKind::Enum) => {}
                        Lookup::Found(_) => return Decision::Known(false),
                        _ => return Decision::Unknown,
                    }
                }
                if declared == Polymorphic::AnyNonArray {
                    match c.type_by_id(input) {
                        Lookup::Found(info) if info.element.is_some() => {
                            return Decision::Known(false);
                        }
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
            // Skipped above.
            Polymorphic::Any => {}
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
    let ids = PolymorphicIds::load(c);
    let result = match ids.of(result) {
        Some(Polymorphic::Any) => return Decision::Known(None),
        Some(p) if p.is_range() => return Decision::Unknown,
        Some(p) => p,
        None => return Decision::Known(Some(Type::Named(result.clone()))),
    };
    let is_element =
        |p: Polymorphic| p.is_scalar() && p.family() == Some(PolymorphicFamily::Element);
    let is_compatible =
        |p: Polymorphic| p.is_scalar() && p.family() == Some(PolymorphicFamily::Compatible);

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
        let Some(declared) = ids.of(declared) else {
            continue;
        };
        if declared.is_range() {
            return Decision::Unknown;
        }
        if declared == Polymorphic::Any {
            continue;
        }
        has_compatible |= declared.family() == Some(PolymorphicFamily::Compatible);
        let input = match actual {
            Type::UnknownLiteral => continue,
            Type::Named(input) => input,
            Type::Record(_) => return Decision::Unknown,
        };
        if is_element(declared) {
            if element.as_ref().is_some_and(|element| element != input) {
                return Decision::Unknown;
            }
            element = Some(input.clone());
        } else if declared == Polymorphic::AnyArray {
            let Decision::Known(input) = base_type(c, input) else {
                return Decision::Unknown;
            };
            if array.as_ref().is_some_and(|array| array != &input) {
                return Decision::Unknown;
            }
            array = Some(input);
        } else if is_compatible(declared) {
            compatible.push(actual.clone());
        } else {
            // `anycompatiblearray`
            let Decision::Known(input) = base_type(c, input) else {
                return Decision::Unknown;
            };
            match array_element(c, &input) {
                Decision::Known(Some(element)) => compatible.push(Type::Named(element)),
                _ => return Decision::Unknown,
            }
        }
    }
    // The element type follows from the array type.
    if let Some(array) = &array {
        match array_element(c, array) {
            Decision::Known(Some(array_element)) => match &element {
                Some(element) if element != &array_element => return Decision::Unknown,
                Some(_) => {}
                None => element = Some(array_element),
            },
            _ => return Decision::Unknown,
        }
    }

    if is_element(result) {
        return match element {
            Some(element) => Decision::Known(Some(Type::Named(element))),
            None => Decision::Unknown,
        };
    }
    if result == Polymorphic::AnyArray {
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
        match c.builtin_type("text") {
            Some(text) => text,
            None => return Decision::Unknown,
        }
    } else {
        match select_common_type(c, &compatible) {
            Selection::Match(Type::Named(common)) => common,
            _ => return Decision::Unknown,
        }
    };
    if result == Polymorphic::AnyCompatibleArray {
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
    let Some(result) = polymorphic(c, result) else {
        return Decision::Unknown;
    };
    let Some(inputs) = inputs
        .iter()
        .map(|input| polymorphic(c, input.as_ref()?))
        .collect::<Option<Vec<_>>>()
    else {
        return Decision::Unknown;
    };
    let Some(result) = result else {
        return Decision::Known(true);
    };
    // Any type of the family determines the result, but only a range or multirange determines
    // a range or multirange result.
    let accepts = |input: Polymorphic| {
        input.family() == result.family() && (!result.is_range() || input.is_range())
    };
    Decision::Known(inputs.into_iter().flatten().any(accepts))
}

/// Which polymorphic type a type is, `Some(None)` if it is not polymorphic, and `None` if the
/// type is unknown.
fn polymorphic(c: &dyn CatalogView, id: &TypeId) -> Option<Option<Polymorphic>> {
    let info = c.type_by_id(id).found()?;
    Some(Polymorphic::from_type(&info).filter(|p| p.is_polymorphic()))
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
