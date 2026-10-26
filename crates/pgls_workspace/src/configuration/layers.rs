use pgls_configuration::{
    Merge, PartialConfiguration, StringSet, database::PartialDatabaseConfiguration,
};

/// The layers laid over each configuration file, lowest precedence first:
///
/// 1. the configuration file, with the files it extends,
/// 2. the database settings of the environment ([PartialDatabaseConfiguration::from_env]),
/// 3. the overrides of the caller: the arguments of the CLI, or the configuration a client of
///    the language server sets. They replace what they set (see [apply_override]).
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
            apply_override(&mut configuration, overrides);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(value: serde_json::Value) -> PartialConfiguration {
        serde_json::from_value(value).unwrap()
    }

    fn layers(
        environment: Option<serde_json::Value>,
        overrides: Option<serde_json::Value>,
    ) -> ConfigurationLayers {
        ConfigurationLayers {
            environment: environment.map(|environment| parse(environment).db.unwrap()),
            overrides: overrides.map(parse),
        }
    }

    #[test]
    fn override_wins_over_environment_and_file() {
        let merged = layers(
            Some(json!({"db":{"host":"env"}})),
            Some(json!({"db":{"host":"client"}})),
        )
        .apply(parse(json!({"db":{"host":"file"}})));
        assert_eq!(merged.db.unwrap().host.as_deref(), Some("client"));
    }

    #[test]
    fn override_search_path_replaces_and_preserves_order() {
        let merged = layers(
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
        let merged = layers(
            None,
            Some(json!({"db":{"host":"localhost", "database":"db"}})),
        )
        .apply(inherited());
        assert_eq!(merged.db.unwrap().connection_string, None);
        let merged = layers(
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
        let merged = layers(None, Some(json!({"db":{"password":""}})))
            .apply(PartialConfiguration::default());
        assert_eq!(merged.db.unwrap().password.as_deref(), Some(""));
    }

    /// The environment overrides the root configuration file, but only fills in a nested one.
    #[test]
    fn environment_fills_in_nested_configurations() {
        let layers = layers(Some(json!({"db":{"host":"env", "database":"env"}})), None);
        let file = || parse(json!({"db":{"host":"file"}}));

        let root = layers.apply(file()).db.unwrap();
        assert_eq!(root.host.as_deref(), Some("env"));
        assert_eq!(root.database.as_deref(), Some("env"));

        let nested = layers.apply_nested(file(), file().db.as_ref()).db.unwrap();
        assert_eq!(nested.host.as_deref(), Some("file"));
        assert_eq!(nested.database.as_deref(), Some("env"));
    }
}
