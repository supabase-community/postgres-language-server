use pgls_analyse::{GroupCategory, RegistryVisitor, RuleCategory, RuleFilter, RuleGroup, RuleMeta};
use pgls_configuration::RuleSelector;
use rustc_hash::FxHashSet;

use crate::settings::Settings;

pub(crate) struct AnalyserVisitorBuilder<'a> {
    lint: Option<LintVisitor<'a>>,
    splinter: Option<SplinterVisitor<'a>>,
    settings: &'a Settings,
}

impl<'a> AnalyserVisitorBuilder<'a> {
    pub(crate) fn new(settings: &'a Settings) -> Self {
        Self {
            settings,
            lint: None,
            splinter: None,
        }
    }
    #[must_use]
    pub(crate) fn with_linter_rules(
        mut self,
        only: &'a [RuleSelector],
        skip: &'a [RuleSelector],
    ) -> Self {
        self.lint = Some(LintVisitor::new(only, skip, self.settings));
        self
    }

    #[must_use]
    pub(crate) fn with_splinter_rules(
        mut self,
        only: &'a [RuleSelector],
        skip: &'a [RuleSelector],
    ) -> Self {
        self.splinter = Some(SplinterVisitor::new(only, skip, self.settings));
        self
    }

    #[must_use]
    pub(crate) fn finish(self) -> (Vec<RuleFilter<'static>>, Vec<RuleFilter<'static>>) {
        let mut disabled_rules = vec![];
        let mut enabled_rules = vec![];
        if let Some(mut lint) = self.lint {
            pgls_analyser::visit_registry(&mut lint);
            let (linter_enabled_rules, linter_disabled_rules) = lint.finish();
            enabled_rules.extend(linter_enabled_rules);
            disabled_rules.extend(linter_disabled_rules);
        }
        if let Some(mut splinter) = self.splinter {
            pgls_splinter::registry::visit_registry(&mut splinter);
            let (splinter_enabled_rules, splinter_disabled_rules) = splinter.finish();
            enabled_rules.extend(splinter_enabled_rules);
            disabled_rules.extend(splinter_disabled_rules);
        }

        (enabled_rules, disabled_rules)
    }
}

/// Type meant to register all the lint rules
#[derive(Debug)]
struct LintVisitor<'a> {
    pub(crate) enabled_rules: FxHashSet<RuleFilter<'static>>,
    pub(crate) disabled_rules: FxHashSet<RuleFilter<'static>>,
    only: &'a [RuleSelector],
    skip: &'a [RuleSelector],
    settings: &'a Settings,
}

impl<'a> LintVisitor<'a> {
    pub(crate) fn new(
        only: &'a [RuleSelector],
        skip: &'a [RuleSelector],
        settings: &'a Settings,
    ) -> Self {
        Self {
            enabled_rules: Default::default(),
            disabled_rules: Default::default(),
            only,
            skip,
            settings,
        }
    }

    fn finish(
        mut self,
    ) -> (
        FxHashSet<RuleFilter<'static>>,
        FxHashSet<RuleFilter<'static>>,
    ) {
        let has_only_filter = !self.only.is_empty();

        if !has_only_filter {
            let enabled_rules = self
                .settings
                .as_linter_rules()
                .map(|rules| rules.as_enabled_rules())
                .unwrap_or_default();

            self.enabled_rules.extend(enabled_rules);

            let disabled_rules = self
                .settings
                .as_linter_rules()
                .map(|rules| rules.as_disabled_rules())
                .unwrap_or_default();
            self.disabled_rules.extend(disabled_rules);
        }

        (self.enabled_rules, self.disabled_rules)
    }

    fn push_rule<R>(&mut self)
    where
        R: RuleMeta + 'static,
    {
        // Do not report unused suppression comment diagnostics if a single rule is run.
        for selector in self.only {
            let filter = RuleFilter::from(selector);
            if filter.match_rule::<R>() {
                self.enabled_rules.insert(filter);
            }
        }
        for selector in self.skip {
            let filter = RuleFilter::from(selector);
            if filter.match_rule::<R>() {
                self.disabled_rules.insert(filter);
            }
        }
    }
}

impl RegistryVisitor for LintVisitor<'_> {
    fn record_category<C: GroupCategory>(&mut self) {
        if C::CATEGORY == RuleCategory::Lint {
            C::record_groups(self)
        }
    }

    fn record_group<G: RuleGroup>(&mut self) {
        for selector in self.only {
            if RuleFilter::from(selector).match_group::<G>() {
                G::record_rules(self)
            }
        }

        for selector in self.skip {
            if RuleFilter::from(selector).match_group::<G>() {
                G::record_rules(self)
            }
        }
    }

    fn record_rule<R>(&mut self)
    where
        R: RuleMeta + 'static,
    {
        self.push_rule::<R>()
    }
}

/// Type meant to register all the splinter (database lint) rules
#[derive(Debug)]
struct SplinterVisitor<'a> {
    pub(crate) enabled_rules: FxHashSet<RuleFilter<'static>>,
    pub(crate) disabled_rules: FxHashSet<RuleFilter<'static>>,
    only: &'a [RuleSelector],
    skip: &'a [RuleSelector],
    settings: &'a Settings,
}

impl<'a> SplinterVisitor<'a> {
    pub(crate) fn new(
        only: &'a [RuleSelector],
        skip: &'a [RuleSelector],
        settings: &'a Settings,
    ) -> Self {
        Self {
            enabled_rules: Default::default(),
            disabled_rules: Default::default(),
            only,
            skip,
            settings,
        }
    }

    fn finish(
        mut self,
    ) -> (
        FxHashSet<RuleFilter<'static>>,
        FxHashSet<RuleFilter<'static>>,
    ) {
        let has_only_filter = !self.only.is_empty();

        if !has_only_filter {
            let enabled_rules = self
                .settings
                .as_splinter_rules()
                .map(|rules| rules.as_enabled_rules())
                .unwrap_or_default();

            self.enabled_rules.extend(enabled_rules);

            let disabled_rules = self
                .settings
                .as_splinter_rules()
                .map(|rules| rules.as_disabled_rules())
                .unwrap_or_default();
            self.disabled_rules.extend(disabled_rules);
        }

        (self.enabled_rules, self.disabled_rules)
    }

    fn push_rule<R>(&mut self)
    where
        R: RuleMeta + 'static,
    {
        for selector in self.only {
            let filter = RuleFilter::from(selector);
            if filter.match_rule::<R>() {
                self.enabled_rules.insert(filter);
            }
        }
        for selector in self.skip {
            let filter = RuleFilter::from(selector);
            if filter.match_rule::<R>() {
                self.disabled_rules.insert(filter);
            }
        }
    }
}

impl RegistryVisitor for SplinterVisitor<'_> {
    fn record_category<C: GroupCategory>(&mut self) {
        // Splinter uses Lint as its kind in declare_category! macro
        // We always record because we're visiting the splinter registry specifically
        C::record_groups(self)
    }

    fn record_group<G: RuleGroup>(&mut self) {
        for selector in self.only {
            if RuleFilter::from(selector).match_group::<G>() {
                G::record_rules(self)
            }
        }

        for selector in self.skip {
            if RuleFilter::from(selector).match_group::<G>() {
                G::record_rules(self)
            }
        }
    }

    fn record_rule<R>(&mut self)
    where
        R: RuleMeta + 'static,
    {
        self.push_rule::<R>()
    }
}

#[cfg(test)]
mod tests {
    use pgls_analyse::RuleFilter;
    use pgls_configuration::{RuleConfiguration, Rules, linter::Safety};

    use crate::{
        settings::{LinterSettings, Settings, SplinterSettings},
        workspace::server::analyser::AnalyserVisitorBuilder,
    };

    #[test]
    fn recognizes_disabled_linter_rules() {
        let settings = Settings {
            linter: LinterSettings {
                rules: Some(Rules {
                    safety: Some(Safety {
                        ban_drop_column: Some(RuleConfiguration::Plain(
                            pgls_configuration::RulePlainConfiguration::Off,
                        )),
                        ..Default::default()
                    }),
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
            vec![RuleFilter::Rule("safety", "banDropColumn")]
        )
    }

    #[test]
    fn recognizes_disabled_splinter_rules() {
        use pgls_configuration::splinter::{Performance, Rules as SplinterRules};

        let settings = Settings {
            splinter: SplinterSettings {
                enabled: true,
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

    #[test]
    fn combines_linter_and_splinter_rules() {
        use pgls_configuration::splinter::{Performance, Rules as SplinterRules};

        let settings = Settings {
            linter: LinterSettings {
                rules: Some(Rules {
                    safety: Some(Safety {
                        ban_drop_column: Some(RuleConfiguration::Plain(
                            pgls_configuration::RulePlainConfiguration::Off,
                        )),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            splinter: SplinterSettings {
                enabled: true,
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
        assert!(disabled_rules.contains(&RuleFilter::Rule("safety", "banDropColumn")));
        assert!(disabled_rules.contains(&RuleFilter::Rule("performance", "authRlsInitplan")));
        assert_eq!(disabled_rules.len(), 2);
    }
}
