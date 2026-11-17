use enumflags2::{BitFlags, bitflags};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum RuleCategory {
    /// This rule performs static analysis of the source code to detect
    /// invalid or error-prone patterns, and emits diagnostics along with
    /// proposed fixes
    Lint,
}

impl TryFrom<String> for RuleCategory {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl TryFrom<&str> for RuleCategory {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "lint" => Ok(Self::Lint),
            _ => Err(format!("Invalid Rule Category: {value}")),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[bitflags]
#[repr(u8)]
pub(crate) enum Categories {
    Lint = 1 << RuleCategory::Lint as u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// The categories supported by the analyser.
///
/// The default implementation of this type returns an instance with all the categories.
///
/// Use [RuleCategoriesBuilder] to generate the categories you want to query.
pub struct RuleCategories(BitFlags<Categories>);

impl RuleCategories {
    pub fn empty() -> Self {
        let empty: BitFlags<Categories> = BitFlags::empty();
        Self(empty)
    }

    pub fn all() -> Self {
        let empty: BitFlags<Categories> = BitFlags::all();
        Self(empty)
    }

    /// Checks whether the current categories contain a specific [RuleCategories]
    pub fn contains(&self, other: impl Into<RuleCategories>) -> bool {
        self.0.contains(other.into().0)
    }
}

impl Default for RuleCategories {
    fn default() -> Self {
        Self::all()
    }
}

impl From<RuleCategory> for RuleCategories {
    fn from(input: RuleCategory) -> Self {
        match input {
            RuleCategory::Lint => RuleCategories(BitFlags::from_flag(Categories::Lint)),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for RuleCategories {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut flags = Vec::new();

        if self.0.contains(Categories::Lint) {
            flags.push(RuleCategory::Lint);
        }

        serializer.collect_seq(flags)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for RuleCategories {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, SeqAccess};
        use std::fmt::{self, Formatter};

        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = RuleCategories;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "RuleCategories")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut result = RuleCategories::empty();

                while let Some(item) = seq.next_element::<String>()? {
                    match item.as_str() {
                        "Lint" => result.0 |= Categories::Lint,
                        // `Action` and `Transformation` never had rules and are no longer
                        // categories, but clients that send all three keep working.
                        "Action" | "Transformation" => {}
                        other => return Err(de::Error::unknown_variant(other, &["Lint"])),
                    }
                }

                Ok(result)
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for RuleCategories {
    fn schema_name() -> String {
        String::from("RuleCategories")
    }

    fn json_schema(r#gen: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        <Vec<RuleCategory>>::json_schema(r#gen)
    }
}

#[derive(Debug, Default)]
/// A convenient type create a [RuleCategories] type
///
/// ```
/// use pgls_analyse::{RuleCategoriesBuilder, RuleCategory};
/// let mut categories = RuleCategoriesBuilder::default().with_lint().build();
///
/// assert!(categories.contains(RuleCategory::Lint));
/// ```
pub struct RuleCategoriesBuilder {
    flags: BitFlags<Categories>,
}

impl RuleCategoriesBuilder {
    pub fn with_lint(mut self) -> Self {
        self.flags.insert(Categories::Lint);
        self
    }

    pub fn all(mut self) -> Self {
        self.flags = BitFlags::all();
        self
    }

    pub fn build(self) -> RuleCategories {
        RuleCategories(self.flags)
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::RuleCategories;

    #[test]
    fn categories_without_rules_are_still_accepted() {
        let all: RuleCategories =
            serde_json::from_str(r#"["Lint", "Action", "Transformation"]"#).unwrap();
        assert_eq!(all, RuleCategories::all());
        assert_eq!(serde_json::to_string(&all).unwrap(), r#"["Lint"]"#);
        assert!(serde_json::from_str::<RuleCategories>(r#"["Refactor"]"#).is_err());
    }
}
