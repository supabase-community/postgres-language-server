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

    /// Ignore any migrations before this timestamp
    #[partial(bpaf(long("after")))]
    pub after: u64,

    /// The migration tool that runs the migration files: "dbmate", "goose" or "none". The marker
    /// comments of the tool split a migration file into up and down sections, which are analysed
    /// as migrations of their own: `-- migrate:up` and `-- migrate:down` of dbmate, `-- +goose Up`
    /// and `-- +goose Down` of goose. Default: "none", every file is one migration.
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
            after: 0,
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

/// A migration tool that puts the up and the down migration in the same file.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum MigrationTool {
    /// The whole file is one migration (default).
    #[default]
    None,
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
            "dbmate" => Ok(Self::Dbmate),
            "goose" => Ok(Self::Goose),
            _ => {
                Err("Value not supported for the migration tool. Use 'none', 'dbmate' or 'goose'.")
            }
        }
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

        let config = MigrationsConfiguration::from(PartialMigrationsConfiguration::default());
        assert_eq!(config.migration_tool(), MigrationTool::None);
        assert!(config.lint_down);
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
