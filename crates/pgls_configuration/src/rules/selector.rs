use pgls_analyse::RuleFilter;

use std::str::FromStr;

/// Represents a rule group from any analyzer (linter or splinter)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum AnalyzerGroup {
    Linter(crate::linter::RuleGroup),
    Splinter(crate::splinter::RuleGroup),
}

impl AnalyzerGroup {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Linter(group) => group.as_str(),
            Self::Splinter(group) => group.as_str(),
        }
    }

    pub const fn category_prefix(&self) -> &'static str {
        match self {
            Self::Linter(_) => "lint",
            Self::Splinter(_) => "splinter",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum RuleSelector {
    Group(AnalyzerGroup),
    Rule(AnalyzerGroup, &'static str),
}

impl From<RuleSelector> for RuleFilter<'static> {
    fn from(value: RuleSelector) -> Self {
        match value {
            RuleSelector::Group(group) => RuleFilter::Group(group.as_str()),
            RuleSelector::Rule(group, name) => RuleFilter::Rule(group.as_str(), name),
        }
    }
}

impl<'a> From<&'a RuleSelector> for RuleFilter<'static> {
    fn from(value: &'a RuleSelector) -> Self {
        match value {
            RuleSelector::Group(group) => RuleFilter::Group(group.as_str()),
            RuleSelector::Rule(group, name) => RuleFilter::Rule(group.as_str(), name),
        }
    }
}

impl FromStr for RuleSelector {
    type Err = &'static str;
    fn from_str(selector: &str) -> Result<Self, Self::Err> {
        // Try to detect the analyzer from the prefix
        let (analyzer_type, rest) = if let Some(rest) = selector.strip_prefix("lint/") {
            ("lint", rest)
        } else if let Some(rest) = selector.strip_prefix("splinter/") {
            ("splinter", rest)
        } else {
            // Default to lint for backward compatibility
            ("lint", selector)
        };

        if let Some((group_name, rule_name)) = rest.split_once('/') {
            // Parse as <group>/<rule>
            match analyzer_type {
                "lint" => {
                    let group = crate::linter::RuleGroup::from_str(group_name)?;
                    if let Some(rule_name) = crate::linter::Rules::has_rule(group, rule_name) {
                        Ok(RuleSelector::Rule(AnalyzerGroup::Linter(group), rule_name))
                    } else {
                        Err("This rule doesn't exist.")
                    }
                }
                "splinter" => {
                    let group = crate::splinter::RuleGroup::from_str(group_name)?;
                    if let Some(rule_name) = crate::splinter::Rules::has_rule(group, rule_name) {
                        Ok(RuleSelector::Rule(
                            AnalyzerGroup::Splinter(group),
                            rule_name,
                        ))
                    } else {
                        Err("This rule doesn't exist.")
                    }
                }
                _ => Err("Unknown analyzer type."),
            }
        } else {
            // Parse as just <group>
            match analyzer_type {
                "lint" => match crate::linter::RuleGroup::from_str(rest) {
                    Ok(group) => Ok(RuleSelector::Group(AnalyzerGroup::Linter(group))),
                    Err(_) => Err(
                        "This group doesn't exist. Use the syntax `<group>/<rule>` to specify a rule.",
                    ),
                },
                "splinter" => match crate::splinter::RuleGroup::from_str(rest) {
                    Ok(group) => Ok(RuleSelector::Group(AnalyzerGroup::Splinter(group))),
                    Err(_) => Err(
                        "This group doesn't exist. Use the syntax `<group>/<rule>` to specify a rule.",
                    ),
                },
                _ => Err("Unknown analyzer type."),
            }
        }
    }
}

impl serde::Serialize for RuleSelector {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            RuleSelector::Group(group) => {
                let prefix = group.category_prefix();
                let group_name = group.as_str();
                serializer.serialize_str(&format!("{prefix}/{group_name}"))
            }
            RuleSelector::Rule(group, rule_name) => {
                let prefix = group.category_prefix();
                let group_name = group.as_str();
                serializer.serialize_str(&format!("{prefix}/{group_name}/{rule_name}"))
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for RuleSelector {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = RuleSelector;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("<group>/<rule_name>")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match RuleSelector::from_str(v) {
                    Ok(result) => Ok(result),
                    Err(error) => Err(serde::de::Error::custom(error)),
                }
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for RuleSelector {
    fn schema_name() -> String {
        "RuleCode".to_string()
    }
    fn json_schema(r#gen: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        String::json_schema(r#gen)
    }
}
