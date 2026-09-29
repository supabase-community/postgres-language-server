use pgls_analyse::RuleFilter;

use std::str::FromStr;

/// Represents a rule group from any analyzer (linter, splinter, or pglinter)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum AnalyzerGroup {
    Linter(&'static str),
    Splinter(crate::splinter::RuleGroup),
    PgLinter(crate::pglinter::RuleGroup),
}

impl AnalyzerGroup {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Linter(group) => group,
            Self::Splinter(group) => group.as_str(),
            Self::PgLinter(group) => group.as_str(),
        }
    }

    pub const fn category_prefix(&self) -> &'static str {
        match self {
            Self::Linter(_) => "lint",
            Self::Splinter(_) => "splinter",
            Self::PgLinter(_) => "pglinter",
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
        if let Some(rest) = selector.strip_prefix("splinter/") {
            return match rest.split_once('/') {
                Some((group_name, rule_name)) => {
                    let group = crate::splinter::RuleGroup::from_str(group_name)?;
                    crate::splinter::Rules::has_rule(group, rule_name)
                        .map(|rule| RuleSelector::Rule(AnalyzerGroup::Splinter(group), rule))
                        .ok_or("This rule doesn't exist.")
                }
                None => crate::splinter::RuleGroup::from_str(rest)
                    .map(|group| RuleSelector::Group(AnalyzerGroup::Splinter(group)))
                    .map_err(|_| {
                        "This group doesn't exist. Use the syntax `<group>/<rule>` to specify a rule."
                    }),
            };
        }

        if let Some(rest) = selector.strip_prefix("pglinter/") {
            return match rest.split_once('/') {
                Some((group_name, rule_name)) => {
                    let group = crate::pglinter::RuleGroup::from_str(group_name)?;
                    crate::pglinter::Rules::has_rule(group, rule_name)
                        .map(|rule| RuleSelector::Rule(AnalyzerGroup::PgLinter(group), rule))
                        .ok_or("This rule doesn't exist.")
                }
                None => crate::pglinter::RuleGroup::from_str(rest)
                    .map(|group| RuleSelector::Group(AnalyzerGroup::PgLinter(group)))
                    .map_err(|_| {
                        "This group doesn't exist. Use the syntax `<group>/<rule>` to specify a rule."
                    }),
            };
        }

        // Linter rules have flat IDs: `lint/<rule>` or `<rule>`. The former `<group>/<rule>`
        // is still accepted, even if the rule moved to another group since.
        let rest = selector.strip_prefix("lint/").unwrap_or(selector);
        let name = match rest.split_once('/') {
            Some((_, rule)) => rule,
            None => rest,
        };
        if let Some(rule) = crate::linter::LINTER_RULES
            .iter()
            .find(|rule| rule.name == name)
        {
            return Ok(RuleSelector::Rule(
                AnalyzerGroup::Linter(rule.group),
                rule.name,
            ));
        }
        if rest.contains('/') {
            return Err("This rule doesn't exist.");
        }
        crate::linter::LINTER_GROUPS
            .iter()
            .find(|group| **group == rest)
            .map(|group| RuleSelector::Group(AnalyzerGroup::Linter(group)))
            .ok_or("This rule or group doesn't exist.")
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
                if matches!(group, AnalyzerGroup::Linter(_)) {
                    serializer.serialize_str(rule_name)
                } else {
                    let prefix = group.category_prefix();
                    let group_name = group.as_str();
                    serializer.serialize_str(&format!("{prefix}/{group_name}/{rule_name}"))
                }
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
