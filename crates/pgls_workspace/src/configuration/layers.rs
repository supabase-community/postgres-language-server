use pgls_configuration::{
    Merge, PartialConfiguration, StringSet, database::PartialDatabaseConfiguration,
};

/// The layers laid over each configuration file, lowest precedence first:
///
/// 1. the configuration file, with the files it extends,
/// 2. the database settings of the environment ([PartialDatabaseConfiguration::from_env]),
/// 3. the overrides of the caller: the arguments of the CLI, or the configuration a client of
///    the language server sets.
///
/// The overrides deliberately outrank the environment. They are an explicit instruction for this
/// run or session, whereas the environment only describes the shell that the process happened to
/// be started from.
#[derive(Clone, Debug, Default)]
pub struct ConfigurationLayers {
    /// The database settings of the environment. They override the root configuration file, but
    /// only fill in the connection options that a nested configuration file doesn't set, so that
    /// each nested configuration keeps its own database.
    pub environment: Option<PartialDatabaseConfiguration>,
    /// The layer over everything else.
    pub overrides: Option<PartialConfiguration>,
    /// How [ConfigurationLayers::overrides] applies.
    pub override_mode: OverrideMode,
    /// The sections that are reset to their defaults if no layer sets them, see
    /// [SectionMask::materialize_defaults].
    pub default_sections: SectionMask,
}

/// How the overrides of [ConfigurationLayers] apply to the layers below them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverrideMode {
    /// Like the environment, with plain [Merge]: the sets are unioned, and a connection string
    /// below keeps winning over the individual connection options. The arguments of the CLI.
    #[default]
    Merge,
    /// With the semantics of [apply_override], so that the overrides replace what they set. The
    /// configuration a client of the language server sets.
    Replace,
}

impl ConfigurationLayers {
    /// Lays the layers over the root configuration file of a project.
    pub fn apply(&self, file: PartialConfiguration) -> PartialConfiguration {
        let mut configuration = file;
        if let Some(environment) = &self.environment {
            configuration.merge_with(PartialConfiguration {
                db: Some(environment.clone()),
                ..Default::default()
            });
        }
        self.apply_overrides(configuration)
    }

    /// Lays the layers over a nested configuration file. `file_database` is the `db` section of
    /// the file itself, without what it extends: the environment only fills in the connection
    /// options that it doesn't set (see [PartialDatabaseConfiguration::for_nested_configuration]).
    pub fn apply_nested(
        &self,
        file: PartialConfiguration,
        file_database: Option<&PartialDatabaseConfiguration>,
    ) -> PartialConfiguration {
        let mut configuration = file;
        if let Some(environment) = self
            .environment
            .as_ref()
            .and_then(|environment| environment.for_nested_configuration(file_database))
        {
            configuration.merge_with(PartialConfiguration {
                db: Some(environment),
                ..Default::default()
            });
        }
        self.apply_overrides(configuration)
    }

    fn apply_overrides(&self, mut configuration: PartialConfiguration) -> PartialConfiguration {
        if let Some(overrides) = &self.overrides {
            match self.override_mode {
                OverrideMode::Merge => configuration.merge_with(overrides.clone()),
                OverrideMode::Replace => apply_override(&mut configuration, overrides),
            }
        }
        self.default_sections
            .materialize_defaults(&mut configuration);
        configuration
    }
}

/// Merges one layer that is meant to *override* the layers below it.
///
/// Plain [`Merge`] is not enough for that, in two ways:
///
/// * `Merge for Option<T>` ignores `None`, so an override can only set values, never unset them.
///   The one case where that matters in practice is the connection string handled below.
/// * `Merge for StringSet` *unions* the two sets, so entries of a lower layer survive — and for an
///   ordered list such as `typecheck.searchPath` they even come first. Every set the override
///   specifies therefore replaces the one below it, by clearing the base set before merging.
fn apply_override(base: &mut PartialConfiguration, override_layer: &PartialConfiguration) {
    // Clears the base sets that `override_layer` specifies, so the merge below replaces them
    // instead of unioning them. Every `StringSet` of `PartialConfiguration` is listed, so that a
    // newly added one cannot silently keep union semantics.
    macro_rules! replace_string_sets {
        ($section:ident: $($field:ident),+) => {
            if let Some(section) = override_layer.$section.as_ref() {
                $(
                    if section.$field.is_some() {
                        base.$section.get_or_insert_default().$field = Some(StringSet::default());
                    }
                )+
            }
        };
    }

    if override_layer.extends.is_some() {
        base.extends = Some(StringSet::default());
    }
    replace_string_sets!(files: include, ignore);
    replace_string_sets!(linter: include, ignore);
    replace_string_sets!(format: include, ignore);
    replace_string_sets!(splinter: ignore);
    replace_string_sets!(typecheck: search_path);
    replace_string_sets!(db: allow_statement_executions_against);

    // `db.connectionString` takes precedence over the individual connection fields and even
    // derives host, port, username and database from them (see `DatabaseSettings::from`), so a
    // connection string inherited from the configuration file or the environment would silently
    // win over an override that names a concrete target. An override that names one therefore
    // unsets the inherited connection string — unless it brings its own, which simply wins.
    if let Some(db) = override_layer.db.as_ref()
        && db.connection_string.is_none()
        && (db.host.is_some()
            || db.port.is_some()
            || db.username.is_some()
            || db.password.is_some()
            || db.database.is_some())
        && let Some(base_db) = base.db.as_mut()
    {
        base_db.connection_string = None;
    }

    base.merge_with(override_layer.clone());
}

macro_rules! define_section_mask {
    ($($section:ident),+ $(,)?) => {
        /// A set of the sections of a [PartialConfiguration].
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct SectionMask { $( $section: bool, )+ }
        impl SectionMask {
            /// The sections that `configuration` sets.
            pub fn of(configuration: &PartialConfiguration) -> Self {
                Self { $( $section: configuration.$section.is_some(), )+ }
            }
            pub fn union(self, other: Self) -> Self {
                Self { $( $section: self.$section || other.$section, )+ }
            }
            /// Materialises every section in this mask that `configuration` omits, using that
            /// section's defaults.
            ///
            /// `Settings::merge_with_configuration` leaves an existing section untouched when the
            /// incoming configuration omits it, and re-registering a project preserves its
            /// settings. Without this step, clearing an override would not restore the baseline:
            /// with an empty configuration file and no environment configuration, `db` would keep
            /// pointing at the overridden database. An all-`None` `Partial*` section maps to that
            /// section's defaults, so materialising it resets the section.
            pub fn materialize_defaults(self, configuration: &mut PartialConfiguration) {
                $( if self.$section && configuration.$section.is_none() {
                    configuration.$section = Some(Default::default());
                } )+
            }
        }
    };
}
define_section_mask!(
    vcs,
    files,
    migrations,
    linter,
    splinter,
    format,
    pglinter,
    typecheck,
    plpgsql_check,
    db,
    inlay_hints,
    semantic_tokens
);

#[cfg(test)]
mod tests {
    use super::*;
    use pgls_configuration::{
        PartialTypecheckConfiguration, database::PartialDatabaseConfiguration,
    };
    use serde_json::json;

    fn parse(value: serde_json::Value) -> PartialConfiguration {
        serde_json::from_value(value).unwrap()
    }

    /// The layers of the language server: the client overrides replace what they set.
    fn client_layers(
        environment: Option<serde_json::Value>,
        overrides: Option<serde_json::Value>,
    ) -> ConfigurationLayers {
        ConfigurationLayers {
            environment: environment.map(|environment| parse(environment).db.unwrap()),
            overrides: overrides.map(parse),
            override_mode: OverrideMode::Replace,
            default_sections: SectionMask::default(),
        }
    }

    #[test]
    fn client_layer_wins_over_environment_and_file() {
        let merged = client_layers(
            Some(json!({"db":{"host":"env"}})),
            Some(json!({"db":{"host":"client"}})),
        )
        .apply(parse(json!({"db":{"host":"file"}})));
        assert_eq!(merged.db.unwrap().host.as_deref(), Some("client"));
    }

    #[test]
    fn override_search_path_replaces_and_preserves_order() {
        let merged = client_layers(
            None,
            Some(json!({"typecheck":{"searchPath":["client_b", "client_a"]}})),
        )
        .apply(parse(
            json!({"typecheck":{"searchPath":["file", "common"]}}),
        ));
        let actual: Vec<_> = merged
            .typecheck
            .unwrap()
            .search_path
            .unwrap()
            .into_iter()
            .collect();
        assert_eq!(actual, ["client_b", "client_a"]);
    }

    #[test]
    fn individual_database_fields_clear_inherited_connection_string() {
        let inherited = || parse(json!({"db":{"connectionString":"postgres://inherited"}}));
        let merged = client_layers(
            None,
            Some(json!({"db":{"host":"localhost", "database":"db"}})),
        )
        .apply(inherited());
        assert_eq!(merged.db.unwrap().connection_string, None);
        let merged = client_layers(
            None,
            Some(json!({"db":{"connectionString":"postgres://own"}})),
        )
        .apply(inherited());
        assert_eq!(
            merged.db.unwrap().connection_string.as_deref(),
            Some("postgres://own")
        );
    }

    #[test]
    fn empty_database_password_is_preserved() {
        let merged = client_layers(None, Some(json!({"db":{"password":""}})))
            .apply(PartialConfiguration::default());
        assert_eq!(merged.db.unwrap().password.as_deref(), Some(""));
    }

    /// The arguments of the CLI are merged like the environment.
    #[test]
    fn merged_overrides_union_sets() {
        let layers = ConfigurationLayers {
            overrides: Some(parse(json!({"typecheck":{"searchPath":["argument"]}}))),
            ..Default::default()
        };
        let merged = layers.apply(parse(json!({"typecheck":{"searchPath":["file"]}})));
        let actual: Vec<_> = merged
            .typecheck
            .unwrap()
            .search_path
            .unwrap()
            .into_iter()
            .collect();
        assert_eq!(actual, ["file", "argument"]);
    }

    #[test]
    fn default_sections_are_materialized() {
        let layers = ConfigurationLayers {
            default_sections: SectionMask::of(&PartialConfiguration {
                db: Some(PartialDatabaseConfiguration::default()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let applied = layers.apply(PartialConfiguration::default());
        assert_eq!(applied.db, Some(PartialDatabaseConfiguration::default()));
    }

    /// The environment overrides the root configuration file, but only fills in a nested one.
    #[test]
    fn environment_fills_in_nested_configurations() {
        let layers = client_layers(Some(json!({"db":{"host":"env", "database":"env"}})), None);
        let file = || parse(json!({"db":{"host":"file"}}));

        let root = layers.apply(file()).db.unwrap();
        assert_eq!(root.host.as_deref(), Some("env"));
        assert_eq!(root.database.as_deref(), Some("env"));

        let nested = layers.apply_nested(file(), file().db.as_ref()).db.unwrap();
        assert_eq!(nested.host.as_deref(), Some("file"));
        assert_eq!(nested.database.as_deref(), Some("env"));
    }

    #[test]
    fn mask_tracks_sections() {
        let mask = SectionMask::of(&PartialConfiguration {
            typecheck: Some(PartialTypecheckConfiguration::default()),
            ..Default::default()
        });
        assert!(mask.typecheck);
    }
}
