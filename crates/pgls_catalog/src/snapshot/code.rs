//! The one-letter codes of catalog columns, like `pg_cast.castmethod`, parsed when a snapshot
//! row is read.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::ArgumentMode;
use crate::{CastMethod, OperatorKind, TypeKind, typing::CoercionContext};

/// A value of a catalog column with a fixed set of codes.
pub trait CatalogCode: Copy + PartialEq + 'static {
    /// Each value with the code the snapshot stores for it. A value may have several codes;
    /// the first one is the one written back.
    const CODES: &'static [(Self, &'static str)];

    fn from_code(code: &str) -> Option<Self> {
        Self::CODES
            .iter()
            .find(|(_, candidate)| *candidate == code)
            .map(|(value, _)| *value)
    }

    fn code(self) -> &'static str {
        Self::CODES
            .iter()
            .find(|(value, _)| *value == self)
            .map_or("", |(_, code)| code)
    }
}

/// A code of a snapshot row. `None` when the code is one we don't know, e.g. from a newer
/// Postgres, or when a legacy snapshot leaves it out. Readers treat that as unknown, never as
/// missing.
///
/// In JSON it is the code as a string, `""` when there is none, so a `SchemaCache` stays the
/// same.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Code<T>(Option<T>);

impl<T: CatalogCode> Code<T> {
    pub fn get(self) -> Option<T> {
        self.0
    }
}

impl<T> Default for Code<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> From<Option<T>> for Code<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T: CatalogCode> From<T> for Code<T> {
    fn from(value: T) -> Self {
        Self(Some(value))
    }
}

impl<T: CatalogCode> From<&str> for Code<T> {
    fn from(code: &str) -> Self {
        Self(T::from_code(code))
    }
}

// What the snapshot queries return.
impl<T: CatalogCode> From<String> for Code<T> {
    fn from(code: String) -> Self {
        Self::from(code.as_str())
    }
}

impl<T: CatalogCode> fmt::Debug for Code<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(value) => write!(f, "{:?}", value.code()),
            None => write!(f, "unknown"),
        }
    }
}

impl<T: CatalogCode> Serialize for Code<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0.map_or("", T::code))
    }
}

impl<'de, T: CatalogCode> Deserialize<'de> for Code<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let code = std::borrow::Cow::<'de, str>::deserialize(deserializer)?;
        Ok(Self::from(code.as_ref()))
    }
}

#[cfg(feature = "schema")]
impl<T> schemars::JsonSchema for Code<T> {
    fn schema_name() -> String {
        String::schema_name()
    }

    fn json_schema(generator: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        String::json_schema(generator)
    }

    fn is_referenceable() -> bool {
        false
    }
}

/// [`TYPTYPE_BASE`] and the ones after it.
///
/// [`TYPTYPE_BASE`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/catalog/pg_type.h#L276
impl CatalogCode for TypeKind {
    const CODES: &'static [(Self, &'static str)] = &[
        (Self::Base, "b"),
        (Self::Composite, "c"),
        (Self::Domain, "d"),
        (Self::Enum, "e"),
        (Self::Pseudo, "p"),
        (Self::Range, "r"),
        (Self::Multirange, "m"),
    ];
}

/// [`CoercionCodes`].
///
/// [`CoercionCodes`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/catalog/pg_cast.h#L75
impl CatalogCode for CoercionContext {
    const CODES: &'static [(Self, &'static str)] = &[
        (Self::Implicit, "i"),
        (Self::Assignment, "a"),
        (Self::Explicit, "e"),
    ];
}

/// [`CoercionMethod`].
///
/// [`CoercionMethod`]: https://github.com/postgres/postgres/blob/REL_18_6/src/include/catalog/pg_cast.h#L87
impl CatalogCode for CastMethod {
    const CODES: &'static [(Self, &'static str)] = &[
        (Self::Function, "f"),
        (Self::InputOutput, "i"),
        (Self::Binary, "b"),
    ];
}

/// The `kind` that `operators.sql` gives for `pg_operator.oprkind`.
impl CatalogCode for OperatorKind {
    const CODES: &'static [(Self, &'static str)] =
        &[(Self::Prefix, "prefix"), (Self::Infix, "infix")];
}

/// The `mode` that `functions.sql` gives for `pg_proc.proargmodes`.
impl CatalogCode for ArgumentMode {
    const CODES: &'static [(Self, &'static str)] = &[
        (Self::In, "in"),
        (Self::Out, "out"),
        (Self::InOut, "inout"),
        (Self::Variadic, "variadic"),
        (Self::Table, "table"),
    ];
}

#[cfg(test)]
mod tests {
    use super::{CatalogCode, Code};
    use crate::snapshot::ArgumentMode;
    use crate::{CastMethod, OperatorKind, TypeKind, typing::CoercionContext};

    fn round_trip<T: CatalogCode>(json: &str) -> (Option<T>, String) {
        let code: Code<T> = serde_json::from_str(json).unwrap();
        (code.get(), serde_json::to_string(&code).unwrap())
    }

    #[test]
    fn known_codes_stay_the_same() {
        assert_eq!(
            round_trip(r#""d""#),
            (Some(TypeKind::Domain), r#""d""#.into())
        );
        assert_eq!(
            round_trip(r#""a""#),
            (Some(CoercionContext::Assignment), r#""a""#.into())
        );
        assert_eq!(
            round_trip(r#""b""#),
            (Some(CastMethod::Binary), r#""b""#.into())
        );
        assert_eq!(
            round_trip(r#""prefix""#),
            (Some(OperatorKind::Prefix), r#""prefix""#.into())
        );
        assert_eq!(
            round_trip(r#""table""#),
            (Some(ArgumentMode::Table), r#""table""#.into())
        );
    }

    #[test]
    fn unknown_codes_are_unknown() {
        // A typtype that a newer Postgres could add, a postfix operator (Postgres 13 and older),
        // and a legacy snapshot without the column.
        assert_eq!(round_trip::<TypeKind>(r#""x""#), (None, r#""""#.into()));
        assert_eq!(
            round_trip::<OperatorKind>(r#""postfix""#),
            (None, r#""""#.into())
        );
        assert_eq!(round_trip::<TypeKind>(r#""""#), (None, r#""""#.into()));
    }
}
