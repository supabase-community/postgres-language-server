//! Whether a value of one type can become another: Postgres' coercion rules.

use crate::typing::types::{array_element, base_type, info, is_array, type_category};
use crate::typing::{CoercionContext, Decision, Type, TypeId};
use crate::{CastContext, CastMethod};
use crate::{CatalogView, Lookup, TypeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoercionPathway {
    RelabelType,
    Func,
    ArrayCoerce,
    CoerceViaIo,
}

fn cast_context(context: CastContext) -> u8 {
    match context {
        CastContext::Implicit => 0,
        CastContext::Assignment => 1,
        CastContext::Explicit => 2,
    }
}
fn coercion_context(context: CoercionContext) -> u8 {
    match context {
        CoercionContext::Implicit => 0,
        CoercionContext::Assignment => 1,
        CoercionContext::Explicit => 2,
    }
}

/// Port of PostgreSQL `find_coercion_pathway`.
pub fn find_coercion_pathway(
    c: &dyn CatalogView,
    source: &TypeId,
    target: &TypeId,
    context: CoercionContext,
) -> Decision<Option<CoercionPathway>> {
    let source = match base_type(c, source) {
        Decision::Known(id) => id,
        Decision::Unknown => return Decision::Unknown,
    };
    let target = match base_type(c, target) {
        Decision::Known(id) => id,
        Decision::Unknown => return Decision::Unknown,
    };
    if source == target {
        return Decision::Known(Some(CoercionPathway::RelabelType));
    }
    match c.cast(&source, &target) {
        Lookup::Found(cast) => {
            if coercion_context(context) < cast_context(cast.context) {
                return Decision::Known(None);
            }
            return Decision::Known(Some(match cast.method {
                CastMethod::Function => CoercionPathway::Func,
                CastMethod::InputOutput => CoercionPathway::CoerceViaIo,
                CastMethod::Binary => CoercionPathway::RelabelType,
            }));
        }
        Lookup::Unknown => return Decision::Unknown,
        Lookup::Missing => {}
    }
    // Derive array coercions only for true arrays, identified by `element` metadata. Postgres
    // never derives them for `oidvector` and `int2vector` targets.
    let vector_target = match (
        is_named(c, &target, "oidvector"),
        is_named(c, &target, "int2vector"),
    ) {
        (Decision::Known(oidvector), Decision::Known(int2vector)) => oidvector || int2vector,
        _ => return Decision::Unknown,
    };
    match (array_element(c, &source), array_element(c, &target)) {
        _ if vector_target => {}
        (Decision::Known(Some(se)), Decision::Known(Some(te))) => {
            match find_coercion_pathway(c, &se, &te, context) {
                Decision::Known(Some(_)) => {
                    return Decision::Known(Some(CoercionPathway::ArrayCoerce));
                }
                Decision::Unknown => return Decision::Unknown,
                Decision::Known(None) => {}
            }
        }
        (Decision::Unknown, _) | (_, Decision::Unknown) => return Decision::Unknown,
        _ => {}
    }
    let sc = type_category(c, &source);
    let tc = type_category(c, &target);
    match (sc, tc) {
        (Decision::Known(_), Decision::Known(('S', _))) if context != CoercionContext::Implicit => {
            Decision::Known(Some(CoercionPathway::CoerceViaIo))
        }
        (Decision::Known(('S', _)), Decision::Known(_)) if context == CoercionContext::Explicit => {
            Decision::Known(Some(CoercionPathway::CoerceViaIo))
        }
        (Decision::Known(_), Decision::Known(_)) => Decision::Known(None),
        _ => Decision::Unknown,
    }
}

fn lookup_named(c: &dyn CatalogView, name: &str) -> Decision<Option<TypeId>> {
    match c.type_(Some("pg_catalog"), name, &[]) {
        Lookup::Found(t) => Decision::Known(t.id),
        Lookup::Missing => Decision::Known(None),
        Lookup::Unknown => Decision::Unknown,
    }
}
fn is_named(c: &dyn CatalogView, id: &TypeId, name: &str) -> Decision<bool> {
    match lookup_named(c, name) {
        Decision::Known(Some(expected)) => Decision::Known(id == &expected),
        Decision::Known(None) => Decision::Known(false),
        Decision::Unknown => Decision::Unknown,
    }
}
fn target_pseudo(c: &dyn CatalogView, id: &TypeId) -> Decision<Option<String>> {
    let t = match info(c, id) {
        Decision::Known(t) => t,
        Decision::Unknown => return Decision::Unknown,
    };
    match t.kind {
        Some(TypeKind::Pseudo) => {}
        Some(_) => return Decision::Known(None),
        None => return Decision::Unknown,
    }
    const NAMES: &[&str] = &[
        "any",
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
    for name in NAMES {
        match is_named(c, id, name) {
            Decision::Known(true) => return Decision::Known(Some((*name).into())),
            Decision::Known(false) => {}
            Decision::Unknown => return Decision::Unknown,
        }
    }
    // Other pseudotypes require semantics outside the scalar coercion model.
    Decision::Unknown
}

/// Port of PostgreSQL `can_coerce_type` for a single input; polymorphic resolution is deferred.
pub fn can_coerce(
    c: &dyn CatalogView,
    from: &Type,
    to: &TypeId,
    context: CoercionContext,
) -> Decision<bool> {
    if matches!(from, Type::UnknownLiteral) {
        return match is_named(c, to, "internal") {
            Decision::Known(true) => Decision::Known(false),
            Decision::Known(false) => Decision::Known(true),
            Decision::Unknown => Decision::Unknown,
        };
    }
    // Anonymous rows convert to composite types at runtime, and to string types through I/O
    // conversion. Neither is modelled.
    if let Type::Record(_) = from {
        return match is_named(c, to, "record") {
            Decision::Known(true) => Decision::Known(true),
            _ => Decision::Unknown,
        };
    }
    let Type::Named(source) = from else {
        unreachable!()
    };
    if source == to {
        return Decision::Known(true);
    }
    match (is_named(c, source, "internal"), is_named(c, to, "internal")) {
        (Decision::Known(true), _) | (_, Decision::Known(true)) => return Decision::Known(false),
        (Decision::Unknown, _) | (_, Decision::Unknown) => return Decision::Unknown,
        _ => {}
    }
    // `ISCOMPLEX` also holds for domains over composite types, which are not followed here.
    // No cast targets `record`, so any other type can't be coerced to it.
    match is_named(c, to, "record") {
        Decision::Known(true) => {
            return match info(c, source) {
                Decision::Known(t) => match t.kind {
                    Some(TypeKind::Composite) => Decision::Known(true),
                    Some(TypeKind::Domain) | Some(TypeKind::Pseudo) | None => Decision::Unknown,
                    Some(_) => Decision::Known(false),
                },
                Decision::Unknown => Decision::Unknown,
            };
        }
        Decision::Unknown => return Decision::Unknown,
        Decision::Known(false) => {}
    }
    match target_pseudo(c, to) {
        Decision::Unknown => return Decision::Unknown,
        Decision::Known(Some(name)) => {
            return if name == "any" {
                Decision::Known(true)
            } else {
                Decision::Unknown
            };
        }
        Decision::Known(None) => {}
    }
    match find_coercion_pathway(c, source, to, context) {
        Decision::Known(Some(_)) => Decision::Known(true),
        Decision::Known(None) => row_coercion(c, source, to),
        Decision::Unknown => Decision::Unknown,
    }
}

/// `can_coerce_type` also accepts a `record` for a composite type, which is checked at
/// runtime, and a composite type for a composite type it inherits from. Neither is modelled,
/// so both are unknown.
fn row_coercion(c: &dyn CatalogView, source: &TypeId, target: &TypeId) -> Decision<bool> {
    let (source, target) = match (info(c, source), info(c, target)) {
        (Decision::Known(source), Decision::Known(target)) => (source, target),
        _ => return Decision::Unknown,
    };
    match (source.kind, target.kind) {
        (Some(TypeKind::Pseudo), _) | (_, Some(TypeKind::Pseudo)) => Decision::Unknown,
        (Some(TypeKind::Composite), Some(TypeKind::Composite)) => Decision::Unknown,
        (Some(_), Some(_)) => Decision::Known(false),
        _ => Decision::Unknown,
    }
}

/// Port of PostgreSQL `IsBinaryCoercible`.
pub fn is_binary_coercible(
    c: &dyn CatalogView,
    source: &TypeId,
    target: &TypeId,
) -> Decision<bool> {
    if source == target {
        return Decision::Known(true);
    }
    let source = match base_type(c, source) {
        Decision::Known(id) => id,
        Decision::Unknown => return Decision::Unknown,
    };
    if source == *target {
        return Decision::Known(true);
    }
    match target_pseudo(c, target) {
        Decision::Unknown => return Decision::Unknown,
        Decision::Known(Some(name)) => {
            return match name.as_str() {
                "any" | "anyelement" | "anycompatible" => Decision::Known(true),
                "anyarray" | "anycompatiblearray" => is_array(c, &source),
                "anynonarray" | "anycompatiblenonarray" => match is_array(c, &source) {
                    Decision::Known(array) => Decision::Known(!array),
                    Decision::Unknown => Decision::Unknown,
                },
                "anyenum" => match info(c, &source) {
                    Decision::Known(t) => Decision::Known(t.kind == Some(TypeKind::Enum)),
                    Decision::Unknown => Decision::Unknown,
                },
                "anyrange" | "anycompatiblerange" => match info(c, &source) {
                    Decision::Known(t) => Decision::Known(t.kind == Some(TypeKind::Range)),
                    Decision::Unknown => Decision::Unknown,
                },
                "anymultirange" | "anycompatiblemultirange" => match info(c, &source) {
                    Decision::Known(t) => Decision::Known(t.kind == Some(TypeKind::Multirange)),
                    Decision::Unknown => Decision::Unknown,
                },
                _ => Decision::Known(false),
            };
        }
        Decision::Known(None) => {}
    }
    match is_named(c, target, "record") {
        Decision::Known(true) => match info(c, &source) {
            Decision::Known(t) => return Decision::Known(t.kind == Some(TypeKind::Composite)),
            Decision::Unknown => return Decision::Unknown,
        },
        Decision::Unknown => return Decision::Unknown,
        Decision::Known(false) => {}
    }
    match c.cast(&source, target) {
        Lookup::Found(cast) => Decision::Known(
            cast.method == CastMethod::Binary && cast.context == CastContext::Implicit,
        ),
        Lookup::Missing => Decision::Known(false),
        Lookup::Unknown => Decision::Unknown,
    }
}

#[cfg(all(test, feature = "db"))]
mod tests {
    use super::*;
    use crate::{Catalog, CatalogBase, Snapshot};
    use sqlx::PgPool;
    use std::sync::Arc;

    #[sqlx::test(migrator = "pgls_test_utils::MIGRATIONS")]
    async fn postgres_cast_pathways(test_db: PgPool) {
        let snapshot = Arc::new(Snapshot::load(&test_db).await.unwrap());
        let catalog = Catalog::new(Some(Arc::new(CatalogBase::new(snapshot))));
        let int4 = TypeId::Snapshot(23);
        let int8 = TypeId::Snapshot(20);
        let text = TypeId::Snapshot(25);
        assert_eq!(
            find_coercion_pathway(&catalog, &int4, &int8, CoercionContext::Implicit),
            Decision::Known(Some(CoercionPathway::Func))
        );
        assert_eq!(
            can_coerce(
                &catalog,
                &Type::Named(text.clone()),
                &int4,
                CoercionContext::Assignment
            ),
            Decision::Known(false)
        );
        assert_eq!(
            can_coerce(
                &catalog,
                &Type::Named(text.clone()),
                &int4,
                CoercionContext::Explicit
            ),
            Decision::Known(true)
        );
        assert_eq!(
            can_coerce(
                &catalog,
                &Type::UnknownLiteral,
                &int4,
                CoercionContext::Implicit
            ),
            Decision::Known(true)
        );
        assert_eq!(
            is_binary_coercible(&catalog, &int4, &int4),
            Decision::Known(true)
        );
        assert_eq!(
            find_coercion_pathway(
                &catalog,
                &TypeId::Snapshot(1007),
                &TypeId::Snapshot(1016),
                CoercionContext::Implicit
            ),
            Decision::Known(Some(CoercionPathway::ArrayCoerce))
        );
        assert_eq!(
            find_coercion_pathway(
                &catalog,
                &text,
                &TypeId::Snapshot(1043),
                CoercionContext::Assignment
            ),
            Decision::Known(Some(CoercionPathway::RelabelType))
        );
        assert_eq!(
            can_coerce(
                &catalog,
                &Type::Named(TypeId::Snapshot(114)),
                &TypeId::Snapshot(3802),
                CoercionContext::Explicit
            ),
            Decision::Known(true)
        );
        assert_eq!(
            can_coerce(
                &catalog,
                &Type::Named(int4),
                &TypeId::Snapshot(16),
                CoercionContext::Explicit
            ),
            Decision::Known(true)
        );
    }
}
