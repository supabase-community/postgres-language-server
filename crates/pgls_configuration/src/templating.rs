use std::{fmt, str::FromStr};

use bpaf::Bpaf;
use pgls_configuration_macros::{Merge, Partial};
use serde::{
    Deserialize, Serialize,
    de::{SeqAccess, Visitor},
};

/// SQL placeholders supplied by host languages or PostgreSQL bind parameters.
#[derive(Clone, Debug, Default, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
pub struct TemplatingConfiguration {
    /// Enabled syntaxes. CLI values are comma-separated; an empty array disables preprocessing.
    #[partial(bpaf(long("template-syntax"), argument("SYNTAXES"), optional))]
    pub syntax: TemplateSyntaxList,
}

/// The syntax of host-language placeholders in SQL.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum TemplateSyntax {
    GoPrintf,
    #[serde(rename = "rust-fmt", alias = "rust-format")]
    RustFormat,
    DollarValues,
}

impl FromStr for TemplateSyntax {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "go-printf" => Ok(Self::GoPrintf),
            "rust-fmt" | "rust-format" => Ok(Self::RustFormat),
            "dollar-values" => Ok(Self::DollarValues),
            _ => Err(format!(
                "Unsupported template syntax: {value}. Expected go-printf, rust-fmt or dollar-values."
            )),
        }
    }
}

// There are three independent parsers. Reject longer lists rather than allocate from config input.
const SYNTAX_ENTRIES_MAX: usize = 3;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TemplateSyntaxList(Vec<TemplateSyntax>);

#[cfg(feature = "schema")]
impl schemars::JsonSchema for TemplateSyntaxList {
    fn schema_name() -> String {
        "TemplateSyntaxList".to_string()
    }

    fn is_referenceable() -> bool {
        false
    }

    fn json_schema(generator: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = Vec::<TemplateSyntax>::json_schema(generator).into_object();
        let array = schema.array.get_or_insert_default();
        array.max_items = Some(SYNTAX_ENTRIES_MAX as u32);
        array.unique_items = Some(true);
        schema.into()
    }
}

impl pgls_configuration::Merge for TemplateSyntaxList {
    fn merge_with(&mut self, other: Self) {
        // An override replaces the list, so [] can clear the enabled syntaxes.
        *self = other;
    }
}

impl<'de> Deserialize<'de> for TemplateSyntaxList {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SyntaxVisitor;
        impl<'de> Visitor<'de> for SyntaxVisitor {
            type Value = TemplateSyntaxList;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an array of up to three distinct template syntaxes")
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = sequence.next_element::<TemplateSyntax>()? {
                    if entries.len() >= SYNTAX_ENTRIES_MAX {
                        tracing::warn!("Template syntax list exceeds {SYNTAX_ENTRIES_MAX} entries");
                        return Err(serde::de::Error::custom(
                            "At most three template syntaxes may be enabled",
                        ));
                    }
                    if entries.contains(&entry) {
                        return Err(serde::de::Error::custom("Duplicate template syntax"));
                    }
                    entries.push(entry);
                }
                Ok(TemplateSyntaxList(entries))
            }
        }
        deserializer.deserialize_seq(SyntaxVisitor)
    }
}

impl FromStr for TemplateSyntaxList {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() || value == "none" {
            return Ok(Self::default());
        }
        let mut entries = Vec::new();
        for name in value.split(',') {
            if entries.len() >= SYNTAX_ENTRIES_MAX {
                tracing::warn!("Template syntax list exceeds {SYNTAX_ENTRIES_MAX} entries");
                return Err("At most three template syntaxes may be enabled".to_string());
            }
            let entry = name.trim().parse()?;
            if entries.contains(&entry) {
                return Err(format!("Duplicate template syntax: {name}"));
            }
            entries.push(entry);
        }
        Ok(Self(entries))
    }
}

impl From<TemplateSyntaxList> for pgls_lexer::TemplateSyntax {
    fn from(value: TemplateSyntaxList) -> Self {
        let mut options = Self::default();
        for syntax in value.0 {
            match syntax {
                TemplateSyntax::GoPrintf => options.go_printf = true,
                TemplateSyntax::RustFormat => options.rust_format = true,
                TemplateSyntax::DollarValues => options.dollar_values = true,
            }
        }
        options
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Merge as _;

    #[test]
    fn template_syntax_array_and_cli_enable_the_same_parsers() {
        let from_json: TemplateSyntaxList =
            serde_json::from_str(r#"["rust-fmt","go-printf","dollar-values"]"#).unwrap();
        let from_cli: TemplateSyntaxList = "go-printf,dollar-values,rust-format".parse().unwrap();
        let json_options: pgls_lexer::TemplateSyntax = from_json.into();
        let cli_options: pgls_lexer::TemplateSyntax = from_cli.into();
        assert_eq!(json_options, cli_options);
        assert!(json_options.go_printf && json_options.rust_format && json_options.dollar_values);
    }

    #[test]
    fn template_syntax_arrays_reject_scalars_unknowns_and_duplicates() {
        for invalid in [
            r#""go-printf""#,
            r#"["whatever"]"#,
            r#"["rust-fmt","rust-format"]"#,
            r#"["go-printf","rust-fmt","dollar-values","go-printf"]"#,
        ] {
            assert!(
                serde_json::from_str::<TemplateSyntaxList>(invalid).is_err(),
                "{invalid}"
            );
        }
        for invalid in [
            "whatever",
            "go-printf,go-printf",
            "go-printf,rust-fmt,dollar-values,go-printf",
        ] {
            assert!(invalid.parse::<TemplateSyntaxList>().is_err(), "{invalid}");
        }
    }

    #[test]
    fn template_syntax_override_replaces_and_can_clear_the_list() {
        let mut original: TemplateSyntaxList = "go-printf".parse().unwrap();
        original.merge_with("dollar-values".parse().unwrap());
        assert_eq!(
            serde_json::to_string(&original).unwrap(),
            r#"["dollar-values"]"#
        );
        original.merge_with(serde_json::from_str("[]").unwrap());
        assert_eq!(
            pgls_lexer::TemplateSyntax::from(original),
            Default::default()
        );
    }
}
