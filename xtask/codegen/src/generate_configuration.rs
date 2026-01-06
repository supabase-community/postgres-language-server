use crate::{to_capitalized, update};
use biome_string_case::Case;
use pgls_analyse::{
    GroupCategory, RegistryVisitor, RuleCategory, RuleGroup, RuleMeta, RuleMetadata,
};
use pgls_diagnostics::Severity;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use quote::quote;
use std::collections::BTreeMap;
use xtask::*;

/// Configuration for a tool that produces rules
struct ToolConfig {
    name: &'static str,
    category: RuleCategory,
}

impl ToolConfig {
    const fn new(name: &'static str, category: RuleCategory) -> Self {
        Self { name, category }
    }

    /// Derived: Directory name under pgls_configuration/src/
    fn config_dir(&self) -> &str {
        self.name
    }

    /// Derived: Crate name that contains the rules
    #[allow(dead_code)]
    fn crate_name(&self) -> String {
        format!("pgls_{}", self.name)
    }

    /// Derived: The main struct name (Rules, Actions, or Transformations)
    #[allow(dead_code)]
    fn struct_name(&self) -> &str {
        match self.category {
            RuleCategory::Lint => "Rules",
            RuleCategory::Action => "Actions",
            RuleCategory::Transformation => "Transformations",
        }
    }

    /// Derived: The generated file name (rules.rs, actions.rs, or transformations.rs)
    fn generated_file(&self) -> &str {
        match self.category {
            RuleCategory::Lint => "rules.rs",
            RuleCategory::Action => "actions.rs",
            RuleCategory::Transformation => "transformations.rs",
        }
    }

    /// Derived: Configuration struct name (LinterConfiguration, AssistsConfiguration, etc.)
    fn config_struct_name(&self) -> String {
        format!("{}Configuration", to_capitalized(self.name))
    }

    /// Derived: Partial configuration struct name
    fn partial_config_struct_name(&self) -> String {
        format!("Partial{}", self.config_struct_name())
    }

    /// Derived: Category prefix used in diagnostics (e.g., "lint" for linter, "splinter" for splinter)
    fn category_prefix(&self) -> &'static str {
        match self.name {
            "linter" => "lint",
            _ => self.name,
        }
    }
}

/// All supported tools
const TOOLS: &[ToolConfig] = &[
    ToolConfig::new("linter", RuleCategory::Lint),
    ToolConfig::new("assists", RuleCategory::Action),
    ToolConfig::new("splinter", RuleCategory::Lint),
    ToolConfig::new("pglinter", RuleCategory::Lint),
];

/// Visitor that collects rules for a specific category
struct CategoryRulesVisitor {
    category: RuleCategory,
    groups: BTreeMap<&'static str, BTreeMap<&'static str, RuleMetadata>>,
}

impl CategoryRulesVisitor {
    fn new(category: RuleCategory) -> Self {
        Self {
            category,
            groups: BTreeMap::new(),
        }
    }
}

impl RegistryVisitor for CategoryRulesVisitor {
    fn record_category<C: GroupCategory>(&mut self) {
        if C::CATEGORY == self.category {
            C::record_groups(self);
        }
    }

    fn record_rule<R>(&mut self)
    where
        R: RuleMeta + 'static,
    {
        self.groups
            .entry(<R::Group as RuleGroup>::NAME)
            .or_default()
            .insert(R::METADATA.name, R::METADATA);
    }
}

/// Generate all rule configurations
pub fn generate_rules_configuration(mode: Mode) -> Result<()> {
    generate_tool_configuration(mode, "linter")?;
    generate_tool_configuration(mode, "splinter")?;
    Ok(())
}

/// Main entry point for generating tool configuration
pub fn generate_tool_configuration(mode: Mode, tool_name: &str) -> Result<()> {
    let tool = TOOLS
        .iter()
        .find(|t| t.name == tool_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown tool: {}", tool_name))?;

    let config_root = project_root().join("crates/pgls_configuration/src");
    let tool_dir = config_root.join(tool.config_dir());

    // Collect rules from the tool's crate
    let mut visitor = CategoryRulesVisitor::new(tool.category);

    match tool.name {
        "linter" => pgls_analyser::visit_registry(&mut visitor),
        "splinter" => pgls_splinter::registry::visit_registry(&mut visitor),
        "assists" => unimplemented!("Assists rules not yet implemented"),
        "pglinter" => unimplemented!("PGLinter rules not yet implemented"),
        _ => unreachable!(),
    }

    // Generate configuration files based on category
    let (mod_content, rules_content) = match tool.category {
        RuleCategory::Lint => generate_lint_config(tool, visitor.groups)?,
        RuleCategory::Action => generate_action_config(tool, visitor.groups)?,
        RuleCategory::Transformation => {
            unimplemented!("Transformation category generation not yet implemented")
        }
    };

    // Write generated files
    update(&tool_dir.join("mod.rs"), &mod_content, &mode)?;
    update(&tool_dir.join(tool.generated_file()), &rules_content, &mode)?;

    Ok(())
}

/// Generate configuration files for Lint category tools
fn generate_lint_config(
    tool: &ToolConfig,
    groups: BTreeMap<&'static str, BTreeMap<&'static str, RuleMetadata>>,
) -> Result<(String, String)> {
    let mod_file = generate_lint_mod_file(tool);
    let rules_file = generate_lint_rules_file(tool, groups)?;
    Ok((mod_file, rules_file))
}

/// Generate the mod.rs file for a Lint tool
fn generate_lint_mod_file(tool: &ToolConfig) -> String {
    let config_struct = Ident::new(&tool.config_struct_name(), Span::call_site());
    let partial_config_struct = Ident::new(&tool.partial_config_struct_name(), Span::call_site());
    let generated_file = tool.generated_file().trim_end_matches(".rs");
    let generated_file_ident = Ident::new(generated_file, Span::call_site());

    // For splinter, we need to include the options module
    let options_module = if tool.name == "splinter" {
        quote! {
            mod options;
            pub use options::SplinterRuleOptions;
        }
    } else {
        quote! {}
    };

    let content = quote! {
        //! Generated file, do not edit by hand, see `xtask/codegen`

        #options_module

        mod #generated_file_ident;

        use biome_deserialize::StringSet;
        use biome_deserialize_macros::{Merge, Partial};
        use bpaf::Bpaf;
        pub use #generated_file_ident::*;
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
        #[partial(derive(Bpaf, Clone, Eq, Merge, PartialEq))]
        #[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
        #[partial(serde(rename_all = "camelCase", default, deny_unknown_fields))]
        pub struct #config_struct {
            /// if `false`, it disables the feature and the linter won't be executed. `true` by default
            #[partial(bpaf(hide))]
            pub enabled: bool,

            /// List of rules
            #[partial(bpaf(pure(Default::default()), optional, hide))]
            pub rules: Rules,

            /// A list of Unix shell style patterns. The linter will ignore files/folders that will match these patterns.
            #[partial(bpaf(hide))]
            pub ignore: StringSet,

            /// A list of Unix shell style patterns. The linter will include files/folders that will match these patterns.
            #[partial(bpaf(hide))]
            pub include: StringSet,
        }

        impl #config_struct {
            pub const fn is_disabled(&self) -> bool {
                !self.enabled
            }
        }

        impl Default for #config_struct {
            fn default() -> Self {
                Self {
                    enabled: true,
                    rules: Default::default(),
                    ignore: Default::default(),
                    include: Default::default(),
                }
            }
        }

        impl #partial_config_struct {
            pub const fn is_disabled(&self) -> bool {
                matches!(self.enabled, Some(false))
            }

            pub fn get_rules(&self) -> Rules {
                self.rules.clone().unwrap_or_default()
            }
        }
    };

    xtask::reformat(content.to_string()).unwrap()
}

/// Generate the rules.rs file for a Lint tool
fn generate_lint_rules_file(
    tool: &ToolConfig,
    groups: BTreeMap<&'static str, BTreeMap<&'static str, RuleMetadata>>,
) -> Result<String> {
    let mut struct_groups = Vec::with_capacity(groups.len());
    let mut group_pascal_idents = Vec::with_capacity(groups.len());
    let mut group_idents = Vec::with_capacity(groups.len());
    let mut group_strings = Vec::with_capacity(groups.len());
    let mut group_as_default_rules = Vec::with_capacity(groups.len());
    let mut group_as_disabled_rules = Vec::with_capacity(groups.len());

    for (group, rules) in groups {
        let group_pascal_ident = quote::format_ident!("{}", &Case::Pascal.convert(group));
        let group_ident = quote::format_ident!("{}", group);

        let (global_all, global_recommended) = {
            (
                quote! { self.is_all_true() },
                quote! { !self.is_recommended_false() },
            )
        };

        group_as_default_rules.push(quote! {
            if let Some(group) = self.#group_ident.as_ref() {
                group.collect_preset_rules(
                    #global_all,
                    #global_recommended,
                    &mut enabled_rules,
                );
                enabled_rules.extend(&group.get_enabled_rules());
                disabled_rules.extend(&group.get_disabled_rules());
            } else if #global_all {
                enabled_rules.extend(#group_pascal_ident::all_rules_as_filters());
            } else if #global_recommended {
                enabled_rules.extend(#group_pascal_ident::recommended_rules_as_filters());
            }
        });

        group_as_disabled_rules.push(quote! {
            if let Some(group) = self.#group_ident.as_ref() {
                disabled_rules.extend(&group.get_disabled_rules());
            }
        });

        group_pascal_idents.push(group_pascal_ident);
        group_idents.push(group_ident);
        group_strings.push(Literal::string(group));
        struct_groups.push(generate_lint_group_struct(tool.name, group, &rules));
    }

    let category_prefix = tool.category_prefix();

    // Generate get_ignore_matchers() method for splinter only
    // We need to generate this method separately in each group struct
    // because we need direct access to the rule configurations
    let get_ignore_matchers_method = if tool.name == "splinter" {
        // Generate code to call each group's get_ignore_matchers method
        let mut group_matcher_code = Vec::new();
        for group_ident in group_idents.iter() {
            group_matcher_code.push(quote! {
                if let Some(group) = &self.#group_ident {
                    matchers.extend(group.get_ignore_matchers());
                }
            });
        }

        quote! {
            /// Build matchers for all rules that have ignore patterns configured.
            /// Returns a map from rule name (camelCase) to the matcher.
            pub fn get_ignore_matchers(&self) -> rustc_hash::FxHashMap<&'static str, pgls_matcher::Matcher> {
                let mut matchers = rustc_hash::FxHashMap::default();
                #( #group_matcher_code )*
                matchers
            }
        }
    } else {
        quote! {}
    };

    let rules_struct_content = quote! {
        //! Generated file, do not edit by hand, see `xtask/codegen`

        use crate::rules::{RuleConfiguration, RulePlainConfiguration};
        use biome_deserialize_macros::Merge;
        use pgls_analyse::RuleFilter;
        use pgls_analyser::RuleOptions;
        use pgls_diagnostics::{Category, Severity};
        use rustc_hash::FxHashSet;
        #[cfg(feature = "schema")]
        use schemars::JsonSchema;
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Copy, Debug, Eq, Hash, Merge, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
        #[cfg_attr(feature = "schema", derive(JsonSchema))]
        #[serde(rename_all = "camelCase")]
        pub enum RuleGroup {
            #( #group_pascal_idents ),*
        }

        impl RuleGroup {
            pub const fn as_str(self) -> &'static str {
                match self {
                    #( Self::#group_pascal_idents => #group_pascal_idents::GROUP_NAME, )*
                }
            }
        }

        impl std::str::FromStr for RuleGroup {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #( #group_pascal_idents::GROUP_NAME => Ok(Self::#group_pascal_idents), )*
                    _ => Err("This rule group doesn't exist.")
                }
            }
        }

        #[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
        #[cfg_attr(feature = "schema", derive(JsonSchema))]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct Rules {
            /// It enables the lint rules recommended by Postgres Language Server. `true` by default.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub recommended: Option<bool>,

            /// It enables ALL rules. The rules that belong to `nursery` won't be enabled.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub all: Option<bool>,

            #(
                #[serde(skip_serializing_if = "Option::is_none")]
                pub #group_idents: Option<#group_pascal_idents>,
            )*
        }

        impl Rules {
            /// Checks if the code coming from [pgls_diagnostics::Diagnostic] corresponds to a rule.
            /// Usually the code is built like {group}/{rule_name}
            pub fn has_rule(
                group: RuleGroup,
                rule_name: &str,
            ) -> Option<&'static str> {
                match group {
                    #(
                        RuleGroup::#group_pascal_idents => #group_pascal_idents::has_rule(rule_name),
                    )*
                }
            }

            /// Given a category coming from [Diagnostic](pgls_diagnostics::Diagnostic), this function returns
            /// the [Severity](pgls_diagnostics::Severity) associated to the rule, if the configuration changed it.
            /// If the severity is off or not set, then the function returns the default severity of the rule,
            /// which is configured at the rule definition.
            /// The function can return `None` if the rule is not properly configured.
            pub fn get_severity_from_code(&self, category: &Category) -> Option<Severity> {
                let mut split_code = category.name().split('/');

                let _category_prefix = split_code.next();
                debug_assert_eq!(_category_prefix, Some(#category_prefix));

                let group = <RuleGroup as std::str::FromStr>::from_str(split_code.next()?).ok()?;
                let rule_name = split_code.next()?;
                let rule_name = Self::has_rule(group, rule_name)?;
                let severity = match group {
                    #(
                        RuleGroup::#group_pascal_idents => self
                            .#group_idents
                            .as_ref()
                            .and_then(|group| group.get_rule_configuration(rule_name))
                            .filter(|(level, _)| !matches!(level, RulePlainConfiguration::Off))
                            .map_or_else(
                                || #group_pascal_idents::severity(rule_name),
                                |(level, _)| level.into()
                            ),
                    )*
                };
                Some(severity)
            }

            /// Ensure that `recommended` is set to `true` or implied.
            pub fn set_recommended(&mut self) {
                if self.all != Some(true) && self.recommended == Some(false) {
                    self.recommended = Some(true)
                }
                #(
                    if let Some(group) = &mut self.#group_idents {
                        group.recommended = None;
                    }
                )*
            }

            pub(crate) const fn is_recommended_false(&self) -> bool {
                matches!(self.recommended, Some(false))
            }

            pub(crate) const fn is_all_true(&self) -> bool {
                matches!(self.all, Some(true))
            }

            /// It returns the enabled rules by default.
            ///
            /// The enabled rules are calculated from the difference with the disabled rules.
            pub fn as_enabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
                let mut enabled_rules = FxHashSet::default();
                let mut disabled_rules = FxHashSet::default();
                #( #group_as_default_rules )*

                enabled_rules.difference(&disabled_rules).copied().collect()
            }

            /// It returns the disabled rules by configuration.
            pub fn as_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
                let mut disabled_rules = FxHashSet::default();
                #( #group_as_disabled_rules )*
                disabled_rules
            }

            #get_ignore_matchers_method
        }

        #( #struct_groups )*

        /// Push the configured rules to the analyser
        pub fn push_to_analyser_rules(
            rules: &Rules,
            metadata: &pgls_analyse::MetadataRegistry,
            analyser_rules: &mut pgls_analyser::LinterRules,
        ) {
            #(
                if let Some(rules) = rules.#group_idents.as_ref() {
                    for rule_name in #group_pascal_idents::GROUP_RULES {
                        if let Some((_, Some(rule_options))) = rules.get_rule_configuration(rule_name) {
                            if let Some(rule_key) = metadata.find_rule(#group_strings, rule_name) {
                                analyser_rules.push_rule(rule_key, rule_options);
                            }
                        }
                    }
                }
            )*
        }

        #[test]
        fn test_order() {
            #(
                for items in #group_pascal_idents::GROUP_RULES.windows(2) {
                    assert!(items[0] < items[1], "{} < {}", items[0], items[1]);
                }
            )*
        }
    };

    xtask::reformat(rules_struct_content.to_string())
}

/// Generate a group struct for lint rules
fn generate_lint_group_struct(
    tool_name: &str,
    group: &str,
    rules: &BTreeMap<&'static str, RuleMetadata>,
) -> TokenStream {
    let mut lines_recommended_rule_as_filter = Vec::new();
    let mut lines_all_rule_as_filter = Vec::new();
    let mut lines_rule = Vec::new();
    let mut schema_lines_rules = Vec::new();
    let mut rule_enabled_check_line = Vec::new();
    let mut rule_disabled_check_line = Vec::new();
    let mut get_rule_configuration_line = Vec::new();
    let mut get_severity_lines = Vec::new();

    // For splinter, generate code to build matchers from ignore patterns
    let mut splinter_ignore_matcher_lines = Vec::new();

    for (index, (rule, metadata)) in rules.iter().enumerate() {
        let summary = extract_summary_from_docs(metadata.docs);
        let rule_position = Literal::u8_unsuffixed(index as u8);
        let rule_identifier = quote::format_ident!("{}", Case::Snake.convert(rule));
        let rule_name = Ident::new(&to_capitalized(rule), Span::call_site());

        if metadata.recommended {
            lines_recommended_rule_as_filter.push(quote! {
                RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[#rule_position])
            });
        }

        lines_all_rule_as_filter.push(quote! {
            RuleFilter::Rule(Self::GROUP_NAME, Self::GROUP_RULES[#rule_position])
        });

        lines_rule.push(quote! {
             #rule
        });

        // For splinter, generate code to check each rule's ignore patterns
        if tool_name == "splinter" {
            let rule_str = Literal::string(rule);
            splinter_ignore_matcher_lines.push(quote! {
                if let Some(conf) = &self.#rule_identifier {
                    if let Some(options) = conf.get_options_ref() {
                        if !options.ignore.is_empty() {
                            let mut m = pgls_matcher::Matcher::new(pgls_matcher::MatchOptions::default());
                            for p in &options.ignore {
                                let _ = m.add_pattern(p);
                            }
                            matchers.insert(#rule_str, m);
                        }
                    }
                }
            });
        }

        // For splinter rules, use SplinterRuleOptions for the shared ignore patterns
        // For linter rules, use pgls_analyser::options::#rule_name
        let rule_option_type = if tool_name == "splinter" {
            quote! { crate::splinter::SplinterRuleOptions }
        } else {
            quote! { pgls_analyser::options::#rule_name }
        };
        let rule_option = quote! { Option<RuleConfiguration<#rule_option_type>> };

        schema_lines_rules.push(quote! {
            #[doc = #summary]
            #[serde(skip_serializing_if = "Option::is_none")]
            pub #rule_identifier: #rule_option
        });

        rule_enabled_check_line.push(quote! {
            if let Some(rule) = self.#rule_identifier.as_ref() {
                if rule.is_enabled() {
                    index_set.insert(RuleFilter::Rule(
                        Self::GROUP_NAME,
                        Self::GROUP_RULES[#rule_position],
                    ));
                }
            }
        });

        rule_disabled_check_line.push(quote! {
            if let Some(rule) = self.#rule_identifier.as_ref() {
                if rule.is_disabled() {
                    index_set.insert(RuleFilter::Rule(
                        Self::GROUP_NAME,
                        Self::GROUP_RULES[#rule_position],
                    ));
                }
            }
        });

        get_rule_configuration_line.push(quote! {
            #rule => self.#rule_identifier.as_ref().map(|conf| (conf.level(), conf.get_options()))
        });

        let severity = match metadata.severity {
            Severity::Hint => quote! { Severity::Hint },
            Severity::Information => quote! { Severity::Information },
            Severity::Warning => quote! { Severity::Warning },
            Severity::Error => quote! { Severity::Error },
            Severity::Fatal => quote! { Severity::Fatal },
        };

        get_severity_lines.push(quote! {
            #rule => #severity
        })
    }

    let group_pascal_ident = Ident::new(&to_capitalized(group), Span::call_site());

    // For splinter, generate get_ignore_matchers method
    let get_ignore_matchers_group_method = if tool_name == "splinter" {
        quote! {
            /// Build matchers for rules in this group that have ignore patterns configured
            pub fn get_ignore_matchers(&self) -> rustc_hash::FxHashMap<&'static str, pgls_matcher::Matcher> {
                let mut matchers = rustc_hash::FxHashMap::default();
                #( #splinter_ignore_matcher_lines )*
                matchers
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #[derive(Clone, Debug, Default, Deserialize, Eq, Merge, PartialEq, Serialize)]
        #[cfg_attr(feature = "schema", derive(JsonSchema))]
        #[serde(rename_all = "camelCase", default, deny_unknown_fields)]
        /// A list of rules that belong to this group
        pub struct #group_pascal_ident {
            /// It enables the recommended rules for this group
            #[serde(skip_serializing_if = "Option::is_none")]
            pub recommended: Option<bool>,

            /// It enables ALL rules for this group.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub all: Option<bool>,

            #( #schema_lines_rules ),*
        }

        impl #group_pascal_ident {
            const GROUP_NAME: &'static str = #group;
            pub(crate) const GROUP_RULES: &'static [&'static str] = &[
                #( #lines_rule ),*
            ];

            const RECOMMENDED_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
                #( #lines_recommended_rule_as_filter ),*
            ];

            const ALL_RULES_AS_FILTERS: &'static [RuleFilter<'static>] = &[
                #( #lines_all_rule_as_filter ),*
            ];

            /// Retrieves the recommended rules
            pub(crate) fn is_recommended_true(&self) -> bool {
                matches!(self.recommended, Some(true))
            }

            pub(crate) fn is_recommended_unset(&self) -> bool {
                self.recommended.is_none()
            }

            pub(crate) fn is_all_true(&self) -> bool {
                matches!(self.all, Some(true))
            }

            pub(crate) fn is_all_unset(&self) -> bool {
                self.all.is_none()
            }

            pub(crate) fn get_enabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
               let mut index_set = FxHashSet::default();
               #( #rule_enabled_check_line )*
               index_set
            }

            pub(crate) fn get_disabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
               let mut index_set = FxHashSet::default();
               #( #rule_disabled_check_line )*
               index_set
            }

            /// Checks if, given a rule name, matches one of the rules contained in this category
            pub(crate) fn has_rule(rule_name: &str) -> Option<&'static str> {
                Some(Self::GROUP_RULES[Self::GROUP_RULES.binary_search(&rule_name).ok()?])
            }

            pub(crate) fn recommended_rules_as_filters() -> &'static [RuleFilter<'static>] {
                Self::RECOMMENDED_RULES_AS_FILTERS
            }

            pub(crate) fn all_rules_as_filters() -> &'static [RuleFilter<'static>] {
                Self::ALL_RULES_AS_FILTERS
            }

            /// Select preset rules
            pub(crate) fn collect_preset_rules(
                &self,
                parent_is_all: bool,
                parent_is_recommended: bool,
                enabled_rules: &mut FxHashSet<RuleFilter<'static>>,
            ) {
                if self.is_all_true() || self.is_all_unset() && parent_is_all {
                    enabled_rules.extend(Self::all_rules_as_filters());
                } else if self.is_recommended_true() || self.is_recommended_unset() && self.is_all_unset() && parent_is_recommended {
                    enabled_rules.extend(Self::recommended_rules_as_filters());
                }
            }

            pub(crate) fn severity(rule_name: &str) -> Severity {
                match rule_name {
                    #( #get_severity_lines ),*,
                    _ => unreachable!()
                }
            }

            pub(crate) fn get_rule_configuration(&self, rule_name: &str) -> Option<(RulePlainConfiguration, Option<RuleOptions>)> {
                match rule_name {
                    #( #get_rule_configuration_line ),*,
                    _ => None
                }
            }

            #get_ignore_matchers_group_method
        }
    }
}

/// Extract the first paragraph from markdown documentation as a summary
fn extract_summary_from_docs(docs: &str) -> String {
    let mut summary = String::new();
    let parser = Parser::new(docs);

    for event in parser {
        match event {
            Event::Text(text) => {
                summary.push_str(text.as_ref());
            }
            Event::Code(text) => {
                // Escape `[` and `<` to obtain valid Markdown
                summary.push_str(text.replace('[', "\\[").replace('<', "\\<").as_ref());
            }
            Event::SoftBreak => {
                summary.push(' ');
            }
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                break;
            }
            Event::Start(tag) => match tag {
                Tag::Strong | Tag::Paragraph => continue,
                _ => {
                    // Skip unsupported tags instead of panicking
                    continue;
                }
            },
            Event::End(tag) => match tag {
                TagEnd::Strong | TagEnd::Paragraph => continue,
                _ => {
                    // Skip unsupported tags instead of panicking
                    continue;
                }
            },
            // Skip HTML, links, and other events that we don't need in the summary
            _ => continue,
        }
    }

    summary
}

/// Generate configuration files for Action category tools (assists)
fn generate_action_config(
    tool: &ToolConfig,
    groups: BTreeMap<&'static str, BTreeMap<&'static str, RuleMetadata>>,
) -> Result<(String, String)> {
    let mod_file = generate_action_mod_file(tool);
    let actions_file = generate_action_actions_file(tool, groups)?;
    Ok((mod_file, actions_file))
}

/// Generate the mod.rs file for an Action tool
fn generate_action_mod_file(tool: &ToolConfig) -> String {
    let config_struct = Ident::new(&tool.config_struct_name(), Span::call_site());
    let partial_config_struct = Ident::new(&tool.partial_config_struct_name(), Span::call_site());
    let generated_file = tool.generated_file().trim_end_matches(".rs");
    let generated_file_ident = Ident::new(generated_file, Span::call_site());

    let content = quote! {
        //! Generated file, do not edit by hand, see `xtask/codegen`

        mod #generated_file_ident;

        use biome_deserialize::StringSet;
        use biome_deserialize_macros::{Deserializable, Merge, Partial};
        use bpaf::Bpaf;
        pub use #generated_file_ident::*;
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Debug, Deserialize, Eq, Partial, PartialEq, Serialize)]
        #[partial(derive(Bpaf, Clone, Deserializable, Eq, Merge, PartialEq))]
        #[partial(cfg_attr(feature = "schema", derive(schemars::JsonSchema)))]
        #[partial(serde(deny_unknown_fields, rename_all = "camelCase"))]
        pub struct #config_struct {
            /// Whether assists should be enabled via LSP.
            #[partial(bpaf(long("assists-enabled"), argument("true|false")))]
            pub enabled: bool,

            /// List of actions
            #[partial(bpaf(pure(Default::default()), optional, hide))]
            pub actions: Actions,

            /// A list of Unix shell style patterns. The assists will ignore files/folders that will
            /// match these patterns.
            #[partial(bpaf(hide))]
            pub ignore: StringSet,

            /// A list of Unix shell style patterns. The assists will include files/folders that will
            /// match these patterns.
            #[partial(bpaf(hide))]
            pub include: StringSet,
        }

        impl Default for #config_struct {
            fn default() -> Self {
                Self {
                    enabled: true,
                    actions: Actions::default(),
                    ignore: Default::default(),
                    include: Default::default(),
                }
            }
        }

        impl #partial_config_struct {
            pub const fn is_disabled(&self) -> bool {
                matches!(self.enabled, Some(false))
            }

            pub fn get_actions(&self) -> Actions {
                self.actions.clone().unwrap_or_default()
            }
        }
    };

    xtask::reformat(content.to_string()).unwrap()
}

/// Generate the actions.rs file for an Action tool
fn generate_action_actions_file(
    _tool: &ToolConfig,
    groups: BTreeMap<&'static str, BTreeMap<&'static str, RuleMetadata>>,
) -> Result<String> {
    let mut struct_groups = Vec::with_capacity(groups.len());
    let mut group_pascal_idents = Vec::with_capacity(groups.len());
    let mut group_idents = Vec::with_capacity(groups.len());
    let mut group_strings = Vec::with_capacity(groups.len());
    let mut group_as_enabled_rules = Vec::with_capacity(groups.len());

    for (group, rules) in groups {
        let group_pascal_ident = quote::format_ident!("{}", &Case::Pascal.convert(group));
        let group_ident = quote::format_ident!("{}", group);

        group_as_enabled_rules.push(quote! {
            if let Some(group) = self.#group_ident.as_ref() {
                enabled_rules.extend(&group.get_enabled_rules());
            }
        });

        group_pascal_idents.push(group_pascal_ident);
        group_idents.push(group_ident);
        group_strings.push(Literal::string(group));
        struct_groups.push(generate_action_group_struct(group, &rules));
    }

    let actions_struct_content = quote! {
        //! Generated file, do not edit by hand, see `xtask/codegen`

        use crate::rules::{RuleAssistConfiguration, RuleAssistPlainConfiguration};
        use biome_deserialize_macros::{Deserializable, Merge};
        use pgls_analyse::RuleFilter;
        use pgls_analyser::RuleOptions;
        use pgls_diagnostics::{Category, Severity};
        use rustc_hash::FxHashSet;
        #[cfg(feature = "schema")]
        use schemars::JsonSchema;
        use serde::{Deserialize, Serialize};

        #[derive(
            Clone,
            Copy,
            Debug,
            Deserializable,
            Eq,
            Hash,
            Merge,
            Ord,
            PartialEq,
            PartialOrd,
            serde::Deserialize,
            serde::Serialize,
        )]
        #[cfg_attr(feature = "schema", derive(JsonSchema))]
        #[serde(rename_all = "camelCase")]
        pub enum RuleGroup {
            #( #group_pascal_idents ),*
        }

        impl RuleGroup {
            pub const fn as_str(self) -> &'static str {
                match self {
                    #( Self::#group_pascal_idents => #group_pascal_idents::GROUP_NAME, )*
                }
            }
        }

        impl std::str::FromStr for RuleGroup {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #( #group_pascal_idents::GROUP_NAME => Ok(Self::#group_pascal_idents), )*
                    _ => Err("This rule group doesn't exist.")
                }
            }
        }

        #[derive(Clone, Debug, Default, Deserialize, Deserializable, Eq, Merge, PartialEq, Serialize)]
        #[cfg_attr(feature = "schema", derive(JsonSchema))]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        pub struct Actions {
            #(
                #[deserializable(rename = #group_strings)]
                #[serde(skip_serializing_if = "Option::is_none")]
                pub #group_idents: Option<#group_pascal_idents>,
            )*
        }

        impl Actions {
            /// Checks if the code coming from [pgls_diagnostics::Diagnostic] corresponds to a rule.
            /// Usually the code is built like {group}/{rule_name}
            pub fn has_rule(
                group: RuleGroup,
                rule_name: &str,
            ) -> Option<&'static str> {
                match group {
                    #(
                        RuleGroup::#group_pascal_idents => #group_pascal_idents::has_rule(rule_name),
                    )*
                }
            }

            /// Given a category coming from [Diagnostic](pgls_diagnostics::Diagnostic), this function returns
            /// the [Severity](pgls_diagnostics::Severity) associated to the rule, if the configuration changed it.
            /// If the severity is off or not set, then the function returns the default severity of the rule:
            /// [Severity::Error] for recommended rules and [Severity::Warning] for other rules.
            ///
            /// If not, the function returns [None].
            pub fn get_severity_from_code(&self, category: &Category) -> Option<Severity> {
                let mut split_code = category.name().split('/');

                let _assists = split_code.next();
                debug_assert_eq!(_assists, Some("assists"));

                let group = <RuleGroup as std::str::FromStr>::from_str(split_code.next()?).ok()?;
                let rule_name = split_code.next()?;
                let rule_name = Self::has_rule(group, rule_name)?;
                match group {
                    #(
                        RuleGroup::#group_pascal_idents => self
                            .#group_idents
                            .as_ref()
                            .and_then(|group| group.get_rule_configuration(rule_name))
                            .filter(|(level, _)| !matches!(level, RuleAssistPlainConfiguration::Off))
                            .map(|(level, _)| level.into()),
                    )*
                }
            }

            /// It returns the enabled rules by default.
            ///
            /// The enabled rules are calculated from the difference with the disabled rules.
            pub fn as_enabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
                let mut enabled_rules = FxHashSet::default();
                #( #group_as_enabled_rules )*
                enabled_rules
            }
        }

        #( #struct_groups )*

        #[test]
        fn test_order() {
            #(
                for items in #group_pascal_idents::GROUP_RULES.windows(2) {
                    assert!(items[0] < items[1], "{} < {}", items[0], items[1]);
                }
            )*
        }
    };

    xtask::reformat(actions_struct_content.to_string())
}

/// Generate a group struct for action rules
fn generate_action_group_struct(
    group: &str,
    rules: &BTreeMap<&'static str, RuleMetadata>,
) -> TokenStream {
    let mut lines_rule = Vec::new();
    let mut schema_lines_rules = Vec::new();
    let mut rule_enabled_check_line = Vec::new();
    let mut get_rule_configuration_line = Vec::new();

    for (index, (rule, metadata)) in rules.iter().enumerate() {
        let summary = extract_summary_from_docs(metadata.docs);
        let rule_position = Literal::u8_unsuffixed(index as u8);
        let rule_identifier = quote::format_ident!("{}", Case::Snake.convert(rule));
        let rule_name = Ident::new(&to_capitalized(rule), Span::call_site());

        lines_rule.push(quote! {
             #rule
        });

        let rule_option_type = quote! {
            pgls_analyser::options::#rule_name
        };
        let rule_option = quote! { Option<RuleAssistConfiguration<#rule_option_type>> };

        schema_lines_rules.push(quote! {
            #[doc = #summary]
            #[serde(skip_serializing_if = "Option::is_none")]
            pub #rule_identifier: #rule_option
        });

        rule_enabled_check_line.push(quote! {
            if let Some(rule) = self.#rule_identifier.as_ref() {
                if rule.is_enabled() {
                    index_set.insert(RuleFilter::Rule(
                        Self::GROUP_NAME,
                        Self::GROUP_RULES[#rule_position],
                    ));
                }
            }
        });

        get_rule_configuration_line.push(quote! {
            #rule => self.#rule_identifier.as_ref().map(|conf| (conf.level(), conf.get_options()))
        });
    }

    let group_pascal_ident = Ident::new(&to_capitalized(group), Span::call_site());

    quote! {
        #[derive(Clone, Debug, Default, Deserialize, Deserializable, Eq, Merge, PartialEq, Serialize)]
        #[cfg_attr(feature = "schema", derive(JsonSchema))]
        #[serde(rename_all = "camelCase", default, deny_unknown_fields)]
        /// A list of rules that belong to this group
        pub struct #group_pascal_ident {
            #( #schema_lines_rules ),*
        }

        impl #group_pascal_ident {
            const GROUP_NAME: &'static str = #group;
            pub(crate) const GROUP_RULES: &'static [&'static str] = &[
                #( #lines_rule ),*
            ];

            pub(crate) fn get_enabled_rules(&self) -> FxHashSet<RuleFilter<'static>> {
               let mut index_set = FxHashSet::default();
               #( #rule_enabled_check_line )*
               index_set
            }

            /// Checks if, given a rule name, matches one of the rules contained in this category
            pub(crate) fn has_rule(rule_name: &str) -> Option<&'static str> {
                Some(Self::GROUP_RULES[Self::GROUP_RULES.binary_search(&rule_name).ok()?])
            }

            pub(crate) fn get_rule_configuration(
                &self,
                rule_name: &str,
            ) -> Option<(RuleAssistPlainConfiguration, Option<RuleOptions>)> {
                match rule_name {
                    #( #get_rule_configuration_line ),*,
                    _ => None
                }
            }
        }
    }
}
