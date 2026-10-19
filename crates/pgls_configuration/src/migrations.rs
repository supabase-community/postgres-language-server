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

    /// The comments that split a migration file into up and down sections, which are analysed
    /// as migrations of their own: "dbmate" (`-- migrate:up`, `-- migrate:down`), "goose"
    /// (`-- +goose Up`, `-- +goose Down`) or "none". Default: "none".
    #[partial(bpaf(long("section-markers")))]
    pub section_markers: SectionMarkers,

    /// If `false`, the down sections of migration files are not linted. Default: `true`.
    #[partial(bpaf(long("lint-down"), argument("true|false")))]
    pub lint_down: bool,
}

impl Default for MigrationsConfiguration {
    fn default() -> Self {
        Self {
            migrations_dir: String::new(),
            after: 0,
            section_markers: SectionMarkers::default(),
            lint_down: true,
        }
    }
}

/// The marker comments of a migration tool.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum SectionMarkers {
    /// The whole file is one migration (default).
    #[default]
    None,
    /// `-- migrate:up` and `-- migrate:down` of dbmate. `transaction:false` runs the section
    /// without a transaction.
    Dbmate,
    /// `-- +goose Up` and `-- +goose Down` of goose. `-- +goose NO TRANSACTION` runs the file
    /// without a transaction.
    Goose,
}

impl FromStr for SectionMarkers {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "dbmate" => Ok(Self::Dbmate),
            "goose" => Ok(Self::Goose),
            _ => Err("Value not supported for SectionMarkers. Use 'none', 'dbmate' or 'goose'."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_section_markers() {
        let config: PartialMigrationsConfiguration =
            serde_json::from_str(r#"{ "sectionMarkers": "goose", "lintDown": false }"#)
                .expect("migrations configuration should deserialize");
        assert_eq!(config.section_markers, Some(SectionMarkers::Goose));
        assert_eq!(config.lint_down, Some(false));

        let config = MigrationsConfiguration::from(PartialMigrationsConfiguration::default());
        assert_eq!(config.section_markers, SectionMarkers::None);
        assert!(config.lint_down);
    }

    #[test]
    fn rejects_unknown_section_markers() {
        assert!(
            serde_json::from_str::<PartialMigrationsConfiguration>(
                r#"{ "sectionMarkers": "flyway" }"#
            )
            .is_err()
        );
    }
}
