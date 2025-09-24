use pgls_analyse::{GroupCategory, RegistryVisitor, Rule, RuleCategory, RuleFilter, RuleGroup};
use pgls_configuration::RuleSelector;
use rustc_hash::FxHashSet;

use crate::settings::Settings;

pub(crate) struct AnalyserVisitorBuilder<'a, 'b> {
    lint: Option<LintVisitor<'a, 'b>>,
    settings: &'b Settings,
}

impl<'a, 'b> AnalyserVisitorBuilder<'a, 'b> {
    pub(crate) fn new(settings: &'b Settings) -> Self {
        Self {
            settings,
            lint: None,
        }
    }
    #[must_use]
    pub(crate) fn with_linter_rules(
        mut self,
        only: &'b [RuleSelector],
        skip: &'b [RuleSelector],
    ) -> Self {
        self.lint = Some(LintVisitor::new(only, skip, self.settings));
        self
    }

    #[must_use]
    pub(crate) fn finish(self) -> (Vec<RuleFilter<'a>>, Vec<RuleFilter<'a>>) {
        let mut disabled_rules = vec![];
        let mut enabled_rules = vec![];
        if let Some(mut lint) = self.lint {
            pgls_analyser::visit_registry(&mut lint);
            let (linter_enabled_rules, linter_disabled_rules) = lint.finish();
            enabled_rules.extend(linter_enabled_rules);
            disabled_rules.extend(linter_disabled_rules);
        }

        (enabled_rules, disabled_rules)
    }
}

/// Type meant to register all the lint rules
#[derive(Debug)]
struct LintVisitor<'a, 'b> {
    pub(crate) enabled_rules: FxHashSet<RuleFilter<'a>>,
    pub(crate) disabled_rules: FxHashSet<RuleFilter<'a>>,
    only: &'b [RuleSelector],
    skip: &'b [RuleSelector],
    settings: &'b Settings,
}

impl<'a, 'b> LintVisitor<'a, 'b> {
    pub(crate) fn new(
        only: &'b [RuleSelector],
        skip: &'b [RuleSelector],
        settings: &'b Settings,
    ) -> Self {
        Self {
            enabled_rules: Default::default(),
            disabled_rules: Default::default(),
            only,
            skip,
            settings,
        }
    }

    fn finish(mut self) -> (FxHashSet<RuleFilter<'a>>, FxHashSet<RuleFilter<'a>>) {
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
        R: Rule<Options: Default> + 'static,
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

impl RegistryVisitor for LintVisitor<'_, '_> {
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
        R: Rule<Options: Default> + 'static,
    {
        self.push_rule::<R>()
    }
}

#[cfg(test)]
mod tests {
    use pgls_analyse::RuleFilter;
    use pgls_configuration::{RuleConfiguration, Rules, analyser::Safety};

    use crate::{
        settings::{LinterSettings, Settings},
        workspace::server::analyser::AnalyserVisitorBuilder,
    };

    #[test]
    fn recognizes_disabled_rules() {
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
}
