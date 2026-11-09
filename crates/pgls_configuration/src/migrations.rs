use std::str::FromStr;

use bpaf::Bpaf;
use pgls_configuration_macros::{Merge, Partial};
use serde::{Deserialize, Serialize};

/// The configuration of the filesystem
#[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
#[partial(derive(Bpaf, Clone, Eq, PartialEq, Merge))]
#[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
#[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
pub struct MigrationsConfiguration {
    /// The directory where the migration files are stored
    #[partial(bpaf(long("migrations-dir")))]
    pub migrations_dir: String,

    /// Ignore the migrations up to this version, in the version format of the migration tool: a
    /// number like `20250301120000`, or a string like `"0042"`. The version of a migration is the
    /// number its file name starts with.
    #[partial(bpaf(long("after"), argument("VERSION")))]
    pub after: MigrationVersion,

    /// The migration tool that runs the migration files: "supabase", "dbmate", "goose" or "none".
    /// The marker comments of the tool split a migration file into up and down sections, which
    /// are analysed as migrations of their own: `-- migrate:up` and `-- migrate:down` of dbmate,
    /// `-- +goose Up` and `-- +goose Down` of goose. The Supabase CLI runs every file as one
    /// migration. Default: "none", every file is one migration.
    #[partial(bpaf(long("migration-tool")))]
    pub tool: Option<MigrationTool>,

    /// Deprecated: use `tool`, which wins if both are set.
    #[partial(bpaf(long("section-markers"), hide))]
    pub section_markers: Option<MigrationTool>,

    /// If `false`, the down sections of migration files are not linted. Default: `true`.
    #[partial(bpaf(long("lint-down"), argument("true|false")))]
    pub lint_down: bool,
}

impl Default for MigrationsConfiguration {
    fn default() -> Self {
        Self {
            migrations_dir: String::new(),
            after: MigrationVersion::default(),
            tool: None,
            section_markers: None,
            lint_down: true,
        }
    }
}

impl MigrationsConfiguration {
    /// The migration tool, from `tool` or the deprecated `sectionMarkers`.
    pub fn migration_tool(&self) -> MigrationTool {
        self.tool.or(self.section_markers).unwrap_or_default()
    }
}

impl PartialMigrationsConfiguration {
    /// The migration tool, from `tool` or the deprecated `sectionMarkers`.
    pub fn migration_tool(&self) -> MigrationTool {
        self.tool.or(self.section_markers).unwrap_or_default()
    }
}

/// The migration tool that runs the migration files.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum MigrationTool {
    /// The whole file is one migration (default).
    #[default]
    None,
    /// The Supabase CLI. Every file is one migration.
    Supabase,
    /// `-- migrate:up` and `-- migrate:down` of dbmate. `transaction:false` runs the section
    /// without a transaction, but still as one query.
    Dbmate,
    /// `-- +goose Up` and `-- +goose Down` of goose. `-- +goose NO TRANSACTION` runs the file
    /// without a transaction, statement by statement.
    Goose,
}

impl FromStr for MigrationTool {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "supabase" => Ok(Self::Supabase),
            "dbmate" => Ok(Self::Dbmate),
            "goose" => Ok(Self::Goose),
            _ => Err(
                "Value not supported for the migration tool. Use 'none', 'supabase', 'dbmate' or 'goose'.",
            ),
        }
    }
}

/// The version of a migration, as the configuration writes it: a number, or a string that the
/// migration tool reads in its own format.
#[derive(Clone, Debug, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum MigrationVersion {
    Number(u64),
    Text(String),
}

impl Default for MigrationVersion {
    fn default() -> Self {
        Self::Number(0)
    }
}

impl MigrationVersion {
    /// The version as text, like a migration file writes it.
    pub fn text(&self) -> String {
        match self {
            Self::Number(number) => number.to_string(),
            Self::Text(text) => text.clone(),
        }
    }
}

impl FromStr for MigrationVersion {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse()
            .map_or_else(|_| Self::Text(s.to_string()), Self::Number))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_the_tool() {
        let config: PartialMigrationsConfiguration =
            serde_json::from_str(r#"{ "tool": "goose", "lintDown": false }"#)
                .expect("migrations configuration should deserialize");
        assert_eq!(config.migration_tool(), MigrationTool::Goose);
        assert_eq!(config.lint_down, Some(false));

        let config: PartialMigrationsConfiguration =
            serde_json::from_str(r#"{ "tool": "supabase" }"#)
                .expect("migrations configuration should deserialize");
        assert_eq!(config.migration_tool(), MigrationTool::Supabase);

        let config = MigrationsConfiguration::from(PartialMigrationsConfiguration::default());
        assert_eq!(config.migration_tool(), MigrationTool::None);
        assert!(config.lint_down);
    }

    #[test]
    fn after_is_a_number_or_a_string() {
        let after = |json: &str| {
            serde_json::from_str::<PartialMigrationsConfiguration>(json)
                .expect("migrations configuration should deserialize")
                .after
        };
        assert_eq!(
            after(r#"{ "after": 20250301120000 }"#),
            Some(MigrationVersion::Number(20250301120000))
        );
        assert_eq!(
            after(r#"{ "after": "0042" }"#),
            Some(MigrationVersion::Text("0042".into()))
        );
        assert_eq!(
            "0042".parse::<MigrationVersion>(),
            Ok(MigrationVersion::Number(42))
        );
        assert_eq!(
            "v1".parse::<MigrationVersion>(),
            Ok(MigrationVersion::Text("v1".into()))
        );
        assert!(
            serde_json::from_str::<PartialMigrationsConfiguration>(r#"{ "after": -1 }"#).is_err()
        );
    }

    #[test]
    fn section_markers_still_select_the_tool() {
        let config: PartialMigrationsConfiguration =
            serde_json::from_str(r#"{ "sectionMarkers": "dbmate" }"#)
                .expect("migrations configuration should deserialize");
        assert_eq!(config.migration_tool(), MigrationTool::Dbmate);
        assert_eq!(
            MigrationsConfiguration::from(config).migration_tool(),
            MigrationTool::Dbmate
        );

        let config: PartialMigrationsConfiguration =
            serde_json::from_str(r#"{ "tool": "goose", "sectionMarkers": "dbmate" }"#)
                .expect("migrations configuration should deserialize");
        assert_eq!(config.migration_tool(), MigrationTool::Goose);
    }

    #[test]
    fn rejects_unknown_tools() {
        for config in [
            r#"{ "tool": "flyway" }"#,
            r#"{ "sectionMarkers": "flyway" }"#,
        ] {
            assert!(serde_json::from_str::<PartialMigrationsConfiguration>(config).is_err());
        }
    }
}
