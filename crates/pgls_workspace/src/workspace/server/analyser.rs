use pgls_analyse::{GroupCategory, RegistryVisitor, RuleCategory, RuleFilter, RuleGroup, RuleMeta};
use pgls_configuration::RuleSelector;
use rustc_hash::FxHashSet;

use crate::settings::Settings;

pub(crate) struct AnalyserVisitorBuilder<'a> {
    lint: Option<RuleSelection<'a>>,
    #[cfg(feature = "db")]
    splinter: Option<RuleSelection<'a>>,
    settings: &'a Settings,
}

impl<'a> AnalyserVisitorBuilder<'a> {
    #[cfg(feature = "db")]
    pub(crate) fn new(settings: &'a Settings) -> Self {
        Self {
            settings,
            lint: None,
            splinter: None,
        }
    }

    #[cfg(not(feature = "db"))]
    pub(crate) fn new(settings: &'a Settings) -> Self {
        Self {
            settings,
            lint: None,
        }
    }

    #[must_use]
    pub(crate) fn with_linter_rules(
        mut self,
        only: &'a [RuleSelector],
        skip: &'a [RuleSelector],
    ) -> Self {
        self.lint = Some(RuleSelection::new(only, skip));
        self
    }

    #[cfg(feature = "db")]
    #[must_use]
    pub(crate) fn with_splinter_rules(
        mut self,
        only: &'a [RuleSelector],
        skip: &'a [RuleSelector],
    ) -> Self {
        self.splinter = Some(RuleSelection::new(only, skip));
        self
    }

    #[must_use]
    pub(crate) fn finish(self) -> (Vec<RuleFilter<'static>>, Vec<RuleFilter<'static>>) {
        let mut disabled_rules = vec![];
        let mut enabled_rules = vec![];
        if let Some(mut lint) = self.lint {
            pgls_analyser::visit_registry(&mut lint);
            let (mut linter_enabled_rules, linter_disabled_rules) = lint.finish(
                self.settings
                    .as_linter_rules()
                    .map(|rules| (rules.as_enabled_rules(), rules.as_disabled_rules())),
            );

            // `linter.enabled` switches the lint rules, and `typecheck.enabled` the typecheck
            // rules.
            let linter_enabled = self.settings.linter.enabled;
            let typecheck_enabled = self.settings.typecheck.enabled;
            linter_enabled_rules.retain(|filter| {
                let group = match filter {
                    RuleFilter::Group(group) | RuleFilter::Rule(group, _) => *group,
                };
                if group == pgls_analyser::TYPECHECK_GROUP {
                    typecheck_enabled
                } else {
                    linter_enabled
                }
            });

            enabled_rules.extend(linter_enabled_rules);
            disabled_rules.extend(linter_disabled_rules);
        }
        #[cfg(feature = "db")]
        if let Some(mut splinter) = self.splinter {
            pgls_splinter::registry::visit_registry(&mut splinter);
            let (splinter_enabled_rules, splinter_disabled_rules) = splinter.finish(
                self.settings
                    .as_splinter_rules()
                    .map(|rules| (rules.as_enabled_rules(), rules.as_disabled_rules())),
            );
            enabled_rules.extend(splinter_enabled_rules);
            disabled_rules.extend(splinter_disabled_rules);
        }

        (enabled_rules, disabled_rules)
    }
}

/// The rules of a registry that `only` and `skip` select, visiting the registry of the linter
/// or of splinter.
#[derive(Debug)]
struct RuleSelection<'a> {
    enabled_rules: FxHashSet<RuleFilter<'static>>,
    disabled_rules: FxHashSet<RuleFilter<'static>>,
    only: &'a [RuleSelector],
    skip: &'a [RuleSelector],
}

impl<'a> RuleSelection<'a> {
    fn new(only: &'a [RuleSelector], skip: &'a [RuleSelector]) -> Self {
        Self {
            enabled_rules: Default::default(),
            disabled_rules: Default::default(),
            only,
            skip,
        }
    }

    /// The enabled and the disabled rules. Without `only`, they include the rules that the
    /// configuration enables and disables, `configured`.
    fn finish(
        mut self,
        configured: Option<(
            FxHashSet<RuleFilter<'static>>,
            FxHashSet<RuleFilter<'static>>,
        )>,
    ) -> (
        FxHashSet<RuleFilter<'static>>,
        FxHashSet<RuleFilter<'static>>,
    ) {
        if self.only.is_empty() {
            let (enabled_rules, disabled_rules) = configured.unwrap_or_default();
            self.enabled_rules.extend(enabled_rules);
            self.disabled_rules.extend(disabled_rules);
        }

        (self.enabled_rules, self.disabled_rules)
    }
}

impl RegistryVisitor for RuleSelection<'_> {
    fn record_category<C: GroupCategory>(&mut self) {
        // The splinter rules are of the lint kind too.
        if C::CATEGORY == RuleCategory::Lint {
            C::record_groups(self)
        }
    }

    fn record_group<G: RuleGroup>(&mut self) {
        if self
            .only
            .iter()
            .flat_map(|selector| selector.filters())
            .any(|filter| filter.match_group::<G>())
        {
            G::record_rules(self)
        }

        if self
            .skip
            .iter()
            .flat_map(|selector| selector.filters())
            .any(|filter| filter.match_group::<G>())
        {
            G::record_rules(self)
        }
    }

    fn record_rule<R>(&mut self)
    where
        R: RuleMeta + 'static,
    {
        // Do not report unused suppression comment diagnostics if a single rule is run.
        for filter in self.only.iter().flat_map(|selector| selector.filters()) {
            if filter.match_rule::<R>() {
                self.enabled_rules.insert(filter);
            }
        }
        for filter in self.skip.iter().flat_map(|selector| selector.filters()) {
            if filter.match_rule::<R>() {
                self.disabled_rules.insert(filter);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pgls_analyse::RuleFilter;
    use pgls_configuration::{RuleConfiguration, Rules};

    use crate::{
        settings::{LinterSettings, Settings},
        workspace::server::analyser::AnalyserVisitorBuilder,
    };

    #[test]
    fn recognizes_disabled_linter_rules() {
        let settings = Settings {
            linter: LinterSettings {
                rules: Some(Rules {
                    ban_drop_column: Some(RuleConfiguration::Plain(
                        pgls_configuration::RulePlainConfiguration::Off,
                    )),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        let (_, disabled_rules) = AnalyserVisitorBuilder::new(&settings)
            .with_linter_rules(&[], &[])
            .finish();

        assert_eq!(
            disabled_rules,
            vec![RuleFilter::Rule("destructive", "banDropColumn")]
        )
    }

    #[cfg(feature = "db")]
    #[test]
    fn recognizes_disabled_splinter_rules() {
        use pgls_configuration::splinter::{Performance, Rules as SplinterRules};

        use crate::settings::SplinterSettings;

        let settings = Settings {
            splinter: SplinterSettings {
                enabled: true,
                ignore: Default::default(),
                rules: Some(SplinterRules {
                    performance: Some(Performance {
                        auth_rls_initplan: Some(RuleConfiguration::Plain(
                            pgls_configuration::RulePlainConfiguration::Off,
                        )),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            },
            ..Default::default()
        };

        let (_, disabled_rules) = AnalyserVisitorBuilder::new(&settings)
            .with_splinter_rules(&[], &[])
            .finish();

        assert_eq!(
            disabled_rules,
            vec![RuleFilter::Rule("performance", "authRlsInitplan")]
        )
    }

    #[cfg(feature = "db")]
    #[test]
    fn combines_linter_and_splinter_rules() {
        use pgls_configuration::splinter::{Performance, Rules as SplinterRules};

        use crate::settings::SplinterSettings;

        let settings = Settings {
            linter: LinterSettings {
                rules: Some(Rules {
                    ban_drop_column: Some(RuleConfiguration::Plain(
                        pgls_configuration::RulePlainConfiguration::Off,
                    )),
                    ..Default::default()
                }),
                ..Default::default()
            },
            splinter: SplinterSettings {
                enabled: true,
                ignore: Default::default(),
                rules: Some(SplinterRules {
                    performance: Some(Performance {
                        auth_rls_initplan: Some(RuleConfiguration::Plain(
                            pgls_configuration::RulePlainConfiguration::Off,
                        )),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            },
            ..Default::default()
        };

        let (_, disabled_rules) = AnalyserVisitorBuilder::new(&settings)
            .with_linter_rules(&[], &[])
            .with_splinter_rules(&[], &[])
            .finish();

        // Should contain disabled rules from both linter and splinter
        assert!(disabled_rules.contains(&RuleFilter::Rule("destructive", "banDropColumn")));
        assert!(disabled_rules.contains(&RuleFilter::Rule("performance", "authRlsInitplan")));
        assert_eq!(disabled_rules.len(), 2);
    }
}
