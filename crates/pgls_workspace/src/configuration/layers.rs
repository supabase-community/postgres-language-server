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
///   The one case where that matters in practice is the connection string handled below, which
///   wins over the connection options that an override sets.
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
    // replaces that part of the inherited connection string, like libpq does for the options
    // that follow a connection string, and keeps the rest of it — unless it brings its own
    // connection string, which simply wins.
    if let Some(db) = override_layer.db.as_ref()
        && db.connection_string.is_none()
        && (db.host.is_some()
            || db.port.is_some()
            || db.username.is_some()
            || db.password.is_some()
            || db.database.is_some())
        && let Some(base_db) = base.db.as_mut()
    {
        base_db.connection_string = base_db
            .connection_string
            .take()
            .and_then(|connection_string| override_connection_string(&connection_string, db));
    }

    base.merge_with(override_layer.clone());
}

/// `connection_string` with the connection options that `db` sets instead of its own. They are
/// set as query parameters, which take precedence over the rest of the URL, so that a host can
/// also be a socket folder or an IPv6 address. `None` if the connection string isn't a URL, so
/// that only the options of `db` apply.
#[cfg(feature = "db")]
fn override_connection_string(
    connection_string: &str,
    db: &PartialDatabaseConfiguration,
) -> Option<String> {
    let mut url = url::Url::parse(connection_string.trim()).ok()?;
    // The query parameters of each option, the first of which is set.
    let options: [(&[&str], Option<String>); 5] = [
        (&["host", "hostaddr"], db.host.clone()),
        (&["port"], db.port.map(|port| port.to_string())),
        (&["user"], db.username.clone()),
        (&["password"], db.password.clone()),
        (&["dbname"], db.database.clone()),
    ];
    let overridden: Vec<&str> = options
        .iter()
        .filter(|(_, value)| value.is_some())
        .flat_map(|(keys, _)| keys.iter().copied())
        .collect();
    let kept: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(key, _)| !overridden.contains(&key.as_ref()))
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    if db.password.is_some() {
        // An empty password counts as none, so it can't be left to the URL.
        url.set_password(None).ok()?;
    }
    url.set_query(None);
    {
        let mut query = url.query_pairs_mut();
        query.extend_pairs(&kept);
        for (keys, value) in &options {
            if let Some(value) = value.as_ref().filter(|value| !value.is_empty()) {
                query.append_pair(keys[0], value);
            }
        }
    }
    if url.query() == Some("") {
        url.set_query(None);
    }
    Some(url.to_string())
}

#[cfg(not(feature = "db"))]
fn override_connection_string(_: &str, _: &PartialDatabaseConfiguration) -> Option<String> {
    None
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

    /// An override of a connection option replaces it in an inherited connection string and
    /// keeps the others, so that `--database app_test` still connects to the host of the file.
    #[cfg(feature = "db")]
    #[test]
    fn individual_database_fields_replace_their_part_of_the_inherited_connection_string() {
        use crate::settings::DatabaseSettings;

        let inherited = || {
            parse(json!({"db":{
                "connectionString":"postgres://alice:secret@db.internal:6543/app?sslmode=require&dbname=app&application_name=pgls"
            }}))
        };
        let settings = |overrides: serde_json::Value| {
            let db = layers(None, Some(overrides)).apply(inherited()).db.unwrap();
            assert!(db.connection_string.is_some());
            DatabaseSettings::from(db)
        };

        let database = settings(json!({"db":{"database":"app_test"}}));
        assert!(database.enable_connection);
        assert_eq!(
            (
                database.host.as_str(),
                database.port,
                database.username.as_str(),
                database.database.as_str()
            ),
            ("db.internal", 6543, "alice", "app_test")
        );
        let url = database.connection_string.unwrap();
        assert!(url.contains("sslmode=require"), "{url}");
        assert!(url.contains("application_name=pgls"), "{url}");
        assert!(url.contains("secret"), "{url}");

        let all = settings(json!({"db":{
            "host":"::1",
            "port":5432,
            "username":"bob",
            "password":"",
            "database":"other"
        }}));
        assert_eq!(
            (
                all.host.as_str(),
                all.port,
                all.username.as_str(),
                all.database.as_str()
            ),
            ("::1", 5432, "bob", "other")
        );
        let url = all.connection_string.unwrap();
        assert!(!url.contains("secret"), "{url}");
        assert!(url.contains("sslmode=require"), "{url}");
    }

    #[test]
    fn own_connection_string_wins_over_the_inherited_one() {
        let merged = layers(
            None,
            Some(json!({"db":{"connectionString":"postgres://own", "host":"localhost"}})),
        )
        .apply(parse(
            json!({"db":{"connectionString":"postgres://inherited"}}),
        ));
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
