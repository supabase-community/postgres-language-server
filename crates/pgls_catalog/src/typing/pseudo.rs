//! The pseudo-types that stand for the type of an argument: `"any"` and the polymorphic types.

use crate::typing::types::lookup_builtin;
use crate::typing::{Decision, TypeId};
use crate::{CatalogView, TypeInfo};

/// `"any"` and the polymorphic pseudo-types of [`IsPolymorphicType`]. `"any"` accepts every type
/// too, but Postgres doesn't count it as polymorphic: it binds nothing, so its
/// [`family`](Self::family) is `None`.
///
/// [`IsPolymorphicType`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/catalog/pg_type.h#L313
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polymorphic {
    Any,
    AnyElement,
    AnyArray,
    AnyNonArray,
    AnyEnum,
    AnyRange,
    AnyMultirange,
    AnyCompatible,
    AnyCompatibleArray,
    AnyCompatibleNonArray,
    AnyCompatibleRange,
    AnyCompatibleMultirange,
}

/// The two families of polymorphic types. The types of a family are resolved together, the
/// `anyelement` family to one element type and the `anycompatible` family to the common type of
/// its inputs ([`IsPolymorphicTypeFamily1`], [`IsPolymorphicTypeFamily2`]).
///
/// [`IsPolymorphicTypeFamily1`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/catalog/pg_type.h#L318
/// [`IsPolymorphicTypeFamily2`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/catalog/pg_type.h#L326
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolymorphicFamily {
    Element,
    Compatible,
}

impl Polymorphic {
    pub const ALL: [Self; 12] = [
        Self::Any,
        Self::AnyElement,
        Self::AnyArray,
        Self::AnyNonArray,
        Self::AnyEnum,
        Self::AnyRange,
        Self::AnyMultirange,
        Self::AnyCompatible,
        Self::AnyCompatibleArray,
        Self::AnyCompatibleNonArray,
        Self::AnyCompatibleRange,
        Self::AnyCompatibleMultirange,
    ];

    /// The name of the type in `pg_catalog`.
    pub fn name(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::AnyElement => "anyelement",
            Self::AnyArray => "anyarray",
            Self::AnyNonArray => "anynonarray",
            Self::AnyEnum => "anyenum",
            Self::AnyRange => "anyrange",
            Self::AnyMultirange => "anymultirange",
            Self::AnyCompatible => "anycompatible",
            Self::AnyCompatibleArray => "anycompatiblearray",
            Self::AnyCompatibleNonArray => "anycompatiblenonarray",
            Self::AnyCompatibleRange => "anycompatiblerange",
            Self::AnyCompatibleMultirange => "anycompatiblemultirange",
        }
    }

    /// The type with this name in `pg_catalog`.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }

    /// The type `info` describes, if it is one of these.
    pub fn from_type(info: &TypeInfo) -> Option<Self> {
        if info.schema != "pg_catalog" {
            return None;
        }
        Self::from_name(&info.name)
    }

    pub fn family(self) -> Option<PolymorphicFamily> {
        match self {
            Self::Any => None,
            Self::AnyElement
            | Self::AnyArray
            | Self::AnyNonArray
            | Self::AnyEnum
            | Self::AnyRange
            | Self::AnyMultirange => Some(PolymorphicFamily::Element),
            Self::AnyCompatible
            | Self::AnyCompatibleArray
            | Self::AnyCompatibleNonArray
            | Self::AnyCompatibleRange
            | Self::AnyCompatibleMultirange => Some(PolymorphicFamily::Compatible),
        }
    }

    /// Whether Postgres counts the type as polymorphic, which is all but `"any"`.
    pub fn is_polymorphic(self) -> bool {
        self.family().is_some()
    }

    /// Whether the type is a range or multirange.
    pub fn is_range(self) -> bool {
        matches!(
            self,
            Self::AnyRange
                | Self::AnyMultirange
                | Self::AnyCompatibleRange
                | Self::AnyCompatibleMultirange
        )
    }

    /// Whether the type binds to the element type of its family itself, rather than to an
    /// array or range of it.
    pub fn is_scalar(self) -> bool {
        matches!(
            self,
            Self::AnyElement
                | Self::AnyNonArray
                | Self::AnyEnum
                | Self::AnyCompatible
                | Self::AnyCompatibleNonArray
        )
    }

    /// The identity of the type, if it is known.
    pub fn type_id(self, c: &dyn CatalogView) -> Option<TypeId> {
        c.builtin_type(self.name())
    }
}

/// The identities of all [`Polymorphic`] types in a catalog, to tell which one a type is
/// without a lookup per type.
pub(crate) struct PolymorphicIds([Option<TypeId>; 12]);

impl PolymorphicIds {
    /// Looks up every type. A type that is missing or unknown has no identity.
    pub(crate) fn load(c: &dyn CatalogView) -> Self {
        Self(Polymorphic::ALL.map(|p| p.type_id(c)))
    }

    /// Looks up every type, and is unknown if any of the lookups is. A missing type has no
    /// identity.
    pub(crate) fn try_load(c: &dyn CatalogView) -> Decision<Self> {
        let mut ids: [Option<TypeId>; 12] = Default::default();
        for (id, p) in ids.iter_mut().zip(Polymorphic::ALL) {
            match lookup_builtin(c, p.name()) {
                Decision::Known(found) => *id = found,
                Decision::Unknown => return Decision::Unknown,
            }
        }
        Decision::Known(Self(ids))
    }

    /// Which of the types `id` is.
    pub(crate) fn of(&self, id: &TypeId) -> Option<Polymorphic> {
        self.0
            .iter()
            .zip(Polymorphic::ALL)
            .find_map(|(candidate, p)| (candidate.as_ref() == Some(id)).then_some(p))
    }
}

#[cfg(test)]
mod tests {
    use super::Polymorphic;

    #[test]
    fn names_round_trip() {
        for p in Polymorphic::ALL {
            assert_eq!(Polymorphic::from_name(p.name()), Some(p));
        }
        assert_eq!(Polymorphic::from_name("record"), None);
    }
}
