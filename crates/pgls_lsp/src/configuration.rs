use pgls_configuration::{Merge, PartialConfiguration, StringSet};

/// Merges the configuration layers of a session, lowest precedence first:
///
/// 1. the built-in defaults,
/// 2. the configuration file on disk,
/// 3. the environment configuration (`DATABASE_URL`, `PGHOST`, ...) captured once at server start,
/// 4. the sticky client override layer.
///
/// The client layer deliberately outranks the environment. The comment this replaces read "Env
/// vars take highest priority — merge last so they override everything", and that stays true of
/// the configuration file: the environment still overrides it. But a client override is an
/// explicit, live instruction from the editor for this session, whereas the environment only
/// describes the shell the server happened to be started from, so the client wins over both.
///
/// Only the client layer gets the override semantics of [`apply_override`]; the environment keeps
/// the plain [`Merge`] behaviour it always had.
pub(crate) fn merge_layers(
    file: PartialConfiguration,
    env: Option<&PartialConfiguration>,
    client: Option<&PartialConfiguration>,
) -> PartialConfiguration {
    let mut configuration = file;
    if let Some(env) = env {
        configuration.merge_with(env.clone());
    }
    if let Some(client) = client {
        apply_override(&mut configuration, client);
    }
    configuration
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
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub(crate) struct SectionMask { $( $section: bool, )+ }
        impl SectionMask {
            fn of(configuration: &PartialConfiguration) -> Self {
                Self { $( $section: configuration.$section.is_some(), )+ }
            }
            fn union(self, other: Self) -> Self {
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
            /// section's defaults in `pgls_workspace`, so materialising it resets the section.
            pub(crate) fn materialize_defaults(self, configuration: &mut PartialConfiguration) {
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
    db
);

/// The sticky layer of configuration a client supplies at runtime.
#[derive(Debug, Default)]
pub(crate) struct ClientOverrides {
    configuration: Option<PartialConfiguration>,
    /// This mask only ever grows: a cleared section remains in the applied stack so it can be
    /// materialised as defaults instead of leaving stale workspace settings active.
    sections: SectionMask,
}

impl ClientOverrides {
    /// Replaces the layer wholesale; it does not accumulate. `None` clears it.
    pub(crate) fn set(&mut self, configuration: Option<PartialConfiguration>) {
        if let Some(configuration) = &configuration {
            self.sections = self.sections.union(SectionMask::of(configuration));
        }
        self.configuration = configuration;
    }
    pub(crate) fn configuration(&self) -> Option<&PartialConfiguration> {
        self.configuration.as_ref()
    }
    pub(crate) fn sections(&self) -> SectionMask {
        self.sections
    }
}

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

    #[test]
    fn client_layer_wins_over_environment_and_file() {
        let merged = merge_layers(
            parse(json!({"db":{"host":"file"}})),
            Some(&parse(json!({"db":{"host":"env"}}))),
            Some(&parse(json!({"db":{"host":"client"}}))),
        );
        assert_eq!(merged.db.unwrap().host.as_deref(), Some("client"));
    }

    #[test]
    fn override_search_path_replaces_and_preserves_order() {
        let merged = merge_layers(
            parse(json!({"typecheck":{"searchPath":["file", "common"]}})),
            None,
            Some(&parse(
                json!({"typecheck":{"searchPath":["client_b", "client_a"]}}),
            )),
        );
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
        let inherited = parse(json!({"db":{"connectionString":"postgres://inherited"}}));
        let merged = merge_layers(
            inherited,
            None,
            Some(&parse(json!({"db":{"host":"localhost", "database":"db"}}))),
        );
        assert_eq!(merged.db.unwrap().connection_string, None);
        let merged = merge_layers(
            parse(json!({"db":{"connectionString":"postgres://inherited"}})),
            None,
            Some(&parse(json!({"db":{"connectionString":"postgres://own"}}))),
        );
        assert_eq!(
            merged.db.unwrap().connection_string.as_deref(),
            Some("postgres://own")
        );
    }

    #[test]
    fn empty_database_password_is_preserved() {
        let merged = merge_layers(
            PartialConfiguration::default(),
            None,
            Some(&parse(json!({"db":{"password":""}}))),
        );
        assert_eq!(merged.db.unwrap().password.as_deref(), Some(""));
    }

    #[test]
    fn clearing_override_materializes_sticky_sections() {
        let mut overrides = ClientOverrides::default();
        overrides.set(Some(PartialConfiguration {
            db: Some(PartialDatabaseConfiguration::default()),
            ..Default::default()
        }));
        overrides.set(None);
        let mut applied = merge_layers(
            PartialConfiguration::default(),
            None,
            overrides.configuration(),
        );
        overrides.sections().materialize_defaults(&mut applied);
        assert_eq!(applied.db, Some(PartialDatabaseConfiguration::default()));
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
