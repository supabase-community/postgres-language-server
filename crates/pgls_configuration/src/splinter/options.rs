#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Shared options for all splinter rules.
///
/// These options allow configuring per-rule filtering of database objects.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SplinterRuleOptions {
    /// A list of glob patterns for database objects to ignore.
    ///
    /// Patterns use Unix-style globs where:
    /// - `*` matches any sequence of characters
    /// - `?` matches any single character
    ///
    /// Each pattern should be in the format `schema.object_name`, for example:
    /// - `"public.my_table"` - ignores a specific table
    /// - `"audit.*"` - ignores all objects in the audit schema
    /// - `"*.audit_*"` - ignores objects with audit_ prefix in any schema
    #[serde(default)]
    pub ignore: Vec<String>,
}
