use std::{ops::Deref, sync::LazyLock};

use pgt_analyse::{
    AnalysedFileContext, AnalyserOptions, AnalysisFilter, MetadataRegistry, RegistryRuleParams,
    RuleDiagnostic, RuleRegistry,
};
pub use registry::visit_registry;

mod lint;
pub mod options;
mod registry;

pub static METADATA: LazyLock<MetadataRegistry> = LazyLock::new(|| {
    let mut metadata = MetadataRegistry::default();
    visit_registry(&mut metadata);
    metadata
});

/// Main entry point to the analyser.
pub struct Analyser<'a> {
    /// Holds the metadata for all the rules statically known to the analyser
    /// we need this later when we add suppression support
    #[allow(dead_code)]
    metadata: &'a MetadataRegistry,

    /// Holds all rule options
    options: &'a AnalyserOptions,

    /// Holds all rules
    registry: RuleRegistry,
}

#[derive(Debug)]
pub struct AnalysableStatement {
    pub root: pgt_query_ext::NodeEnum,
    pub range: pgt_text_size::TextRange,
}

pub struct AnalyserParams<'a> {
    pub stmts: Vec<AnalysableStatement>,
    pub schema_cache: Option<&'a pgt_schema_cache::SchemaCache>,
}

pub struct AnalyserConfig<'a> {
    pub options: &'a AnalyserOptions,
    pub filter: AnalysisFilter<'a>,
}

impl<'a> Analyser<'a> {
    pub fn new(conf: AnalyserConfig<'a>) -> Self {
        let mut builder = RuleRegistry::builder(&conf.filter);
        visit_registry(&mut builder);
        let registry = builder.build();

        Self {
            metadata: METADATA.deref(),
            registry,
            options: conf.options,
        }
    }

    pub fn run(&self, params: AnalyserParams) -> Vec<RuleDiagnostic> {
        let mut diagnostics = vec![];

        let mut file_context = AnalysedFileContext::default();

        for stmt in params.stmts {
            let rule_params = RegistryRuleParams {
                root: &stmt.root,
                options: self.options,
                analysed_file_context: &file_context,
                schema_cache: params.schema_cache,
            };

            diagnostics.extend(
                self.registry
                    .rules
                    .iter()
                    .flat_map(|rule| (rule.run)(&rule_params)),
            );

            file_context.update_from(&stmt.root);
        }

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use core::slice;

    use pgt_analyse::{AnalyserOptions, AnalysisFilter, RuleFilter};
    use pgt_console::{
        Markup,
        fmt::{Formatter, Termcolor},
        markup,
    };
    use pgt_diagnostics::PrintDiagnostic;
    use pgt_text_size::TextRange;
    use termcolor::NoColor;

    use crate::{AnalysableStatement, Analyser};

    #[ignore]
    #[test]
    fn debug_test() {
        fn markup_to_string(markup: Markup) -> String {
            let mut buffer = Vec::new();
            let mut write = Termcolor(NoColor::new(&mut buffer));
            let mut fmt = Formatter::new(&mut write);
            fmt.write_markup(markup).unwrap();

            String::from_utf8(buffer).unwrap()
        }

        const SQL: &str = r#"alter table test drop column id;"#;
        let rule_filter = RuleFilter::Rule("safety", "banDropColumn");

        let filter = AnalysisFilter {
            enabled_rules: Some(slice::from_ref(&rule_filter)),
            ..Default::default()
        };

        let ast = pgt_query_ext::parse(SQL).expect("failed to parse SQL");
        let range = TextRange::new(0.into(), u32::try_from(SQL.len()).unwrap().into());

        let options = AnalyserOptions::default();

        let analyser = Analyser::new(crate::AnalyserConfig {
            options: &options,
            filter,
        });

        let results = analyser.run(crate::AnalyserParams {
            stmts: vec![AnalysableStatement { root: ast, range }],
            schema_cache: None,
        });

        println!("*******************");
        for result in &results {
            let text = markup_to_string(markup! {
                {PrintDiagnostic::simple(result)}
            });
            eprintln!("{}", text);
        }
        println!("*******************");

        // assert_eq!(results, vec![]);
    }
}
