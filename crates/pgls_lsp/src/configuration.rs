use pgls_configuration::PartialConfiguration;
use pgls_workspace::configuration::SectionMask;

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
    use pgls_configuration::database::PartialDatabaseConfiguration;
    use pgls_workspace::configuration::{ConfigurationLayers, OverrideMode};

    #[test]
    fn clearing_override_materializes_sticky_sections() {
        let mut overrides = ClientOverrides::default();
        overrides.set(Some(PartialConfiguration {
            db: Some(PartialDatabaseConfiguration::default()),
            ..Default::default()
        }));
        overrides.set(None);
        let layers = ConfigurationLayers {
            environment: None,
            overrides: overrides.configuration().cloned(),
            override_mode: OverrideMode::Replace,
            default_sections: overrides.sections(),
        };
        let applied = layers.apply(PartialConfiguration::default());
        assert_eq!(applied.db, Some(PartialDatabaseConfiguration::default()));
    }
}
