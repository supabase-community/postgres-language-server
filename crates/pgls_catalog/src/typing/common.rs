use crate::typing::coerce::{base_type, can_coerce, type_category};
use crate::typing::{CoercionContext, Decision, Selection, Type, TypeId};
use crate::{CatalogView, Lookup};

fn text_type(c: &dyn CatalogView) -> Decision<Type> {
    match c.type_(Some("pg_catalog"), "text", &[]) {
        Lookup::Found(info) => match info.id {
            Some(id) => Decision::Known(Type::Named(id)),
            None => Decision::Unknown,
        },
        Lookup::Missing | Lookup::Unknown => Decision::Unknown,
    }
}

/// Port of PostgreSQL `select_common_type`, including `verify_common_type` and
/// `coerce_to_common_type`'s implicit-coercibility validation.
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

#[cfg(all(test, feature = "db"))]
mod tests {
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
