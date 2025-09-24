use std::collections::BTreeMap;
use std::str::FromStr;
use std::{fmt::Write, slice};

use anyhow::bail;
use pgls_analyse::{
    AnalyserOptions, AnalysisFilter, GroupCategory, RegistryVisitor, Rule, RuleCategory,
    RuleFilter, RuleGroup, RuleMetadata,
};
use pgls_analyser::{AnalysableStatement, Analyser, AnalyserConfig};
use pgls_console::{markup, Console};
use pgls_diagnostics::{Diagnostic, DiagnosticExt, PrintDiagnostic};
use pgls_query_ext::diagnostics::SyntaxDiagnostic;
use pgls_workspace::settings::Settings;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

pub fn check_rules() -> anyhow::Result<()> {
    #[derive(Default)]
    struct LintRulesVisitor {
        groups: BTreeMap<&'static str, BTreeMap<&'static str, RuleMetadata>>,
    }

    impl LintRulesVisitor {
        fn push_rule<R>(&mut self)
        where
            R: Rule<Options: Default> + 'static,
        {
            self.groups
                .entry(<R::Group as RuleGroup>::NAME)
                .or_default()
                .insert(R::METADATA.name, R::METADATA);
        }
    }

    impl RegistryVisitor for LintRulesVisitor {
        fn record_category<C: GroupCategory>(&mut self) {
            if matches!(C::CATEGORY, RuleCategory::Lint) {
                C::record_groups(self);
            }
        }

        fn record_rule<R>(&mut self)
        where
            R: Rule<Options: Default> + 'static,
        {
            self.push_rule::<R>()
        }
    }

    let mut visitor = LintRulesVisitor::default();
    pgls_analyser::visit_registry(&mut visitor);

    let LintRulesVisitor { groups } = visitor;

    for (group, rules) in groups {
        for (_, meta) in rules {
            parse_documentation(group, meta.name, meta.docs)?;
        }
    }

    Ok(())
}

/// Parse and analyze the provided code block, and asserts that it emits
/// exactly zero or one diagnostic depending on the value of `expect_diagnostic`.
/// That diagnostic is then emitted as text into the `content` buffer
fn assert_lint(
    group: &'static str,
    rule: &'static str,
    test: &CodeBlockTest,
    code: &str,
) -> anyhow::Result<()> {
    let file_path = format!("code-block.{}", test.tag);
    let mut diagnostic_count = 0;
    let mut all_diagnostics = vec![];
    let mut has_error = false;
    let mut write_diagnostic = |code: &str, diag: pgls_diagnostics::Error| {
        all_diagnostics.push(diag);
        // Fail the test if the analysis returns more diagnostics than expected
        if test.expect_diagnostic {
            // Print all diagnostics to help the user
            if all_diagnostics.len() > 1 {
                let mut console = pgls_console::EnvConsole::default();
                for diag in all_diagnostics.iter() {
                    console.println(
                        pgls_console::LogLevel::Error,
                        markup! {
                            {PrintDiagnostic::verbose(diag)}
                        },
                    );
                }
                has_error = true;
                bail!("Analysis of '{group}/{rule}' on the following code block returned multiple diagnostics.\n\n{code}");
            }
        } else {
            // Print all diagnostics to help the user
            let mut console = pgls_console::EnvConsole::default();
            for diag in all_diagnostics.iter() {
                console.println(
                    pgls_console::LogLevel::Error,
                    markup! {
                        {PrintDiagnostic::verbose(diag)}
                    },
                );
            }
            has_error = true;
            bail!("Analysis of '{group}/{rule}' on the following code block returned an unexpected diagnostic.\n\n{code}");
        }
        diagnostic_count += 1;
        Ok(())
    };

    if test.ignore {
        return Ok(());
    }

    let rule_filter = RuleFilter::Rule(group, rule);
    let filter = AnalysisFilter {
        enabled_rules: Some(slice::from_ref(&rule_filter)),
        ..AnalysisFilter::default()
    };
    let settings = Settings::default();
    let options = AnalyserOptions::default();
    let analyser = Analyser::new(AnalyserConfig {
        options: &options,
        filter,
    });

    let result = pgls_statement_splitter::split(code);
    for stmt_range in result.ranges {
        match pgls_query::parse(&code[stmt_range]) {
            Ok(ast) => {
                if let Some(root) = ast.into_root() {
                    for rule_diag in analyser.run(pgls_analyser::AnalyserParams {
                        schema_cache: None,
                        stmts: vec![AnalysableStatement {
                            range: stmt_range,
                            root,
                        }],
                    }) {
                        let diag = pgls_diagnostics::serde::Diagnostic::new(rule_diag);

                        let category = diag.category().expect("linter diagnostic has no code");
                        let severity = settings.get_severity_from_rule_code(category).expect(
                                "If you see this error, it means you need to run cargo codegen-configuration",
                            );

                        let error = diag
                            .with_severity(severity)
                            .with_file_path(&file_path)
                            .with_file_source_code(code);

                        write_diagnostic(code, error)?;
                    }
                }
            }
            Err(e) => {
                let error = SyntaxDiagnostic::from(e)
                    .with_file_path(&file_path)
                    .with_file_source_code(code);
                write_diagnostic(code, error)?;
            }
        };
    }
    if !result.errors.is_empty() {
        // Print all diagnostics to help the user
        let mut console = pgls_console::EnvConsole::default();
        for err in result.errors {
            console.println(
                pgls_console::LogLevel::Error,
                markup! {
                    {PrintDiagnostic::verbose(&err)}
                },
            );
        }
        bail!("Analysis of '{group}/{rule}' on the following code block returned a scan diagnostic.\n\n{code}");
    }

    Ok(())
}

struct CodeBlockTest {
    tag: String,
    expect_diagnostic: bool,
    ignore: bool,
}

impl FromStr for CodeBlockTest {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> anyhow::Result<Self> {
        // This is based on the parsing logic for code block languages in `rustdoc`:
        // https://github.com/rust-lang/rust/blob/6ac8adad1f7d733b5b97d1df4e7f96e73a46db42/src/librustdoc/html/markdown.rs#L873
        let tokens = input
            .split([',', ' ', '\t'])
            .map(str::trim)
            .filter(|token| !token.is_empty());

        let mut test = CodeBlockTest {
            tag: String::new(),
            expect_diagnostic: false,
            ignore: false,
        };

        for token in tokens {
            match token {
                // Other attributes
                "expect_diagnostic" => test.expect_diagnostic = true,
                "ignore" => test.ignore = true,
                // Regard as language tags, last one wins
                _ => test.tag = token.to_string(),
            }
        }

        Ok(test)
    }
}

/// Parse the documentation fragment for a lint rule (in markdown) and lint the code blcoks.
fn parse_documentation(
    group: &'static str,
    rule: &'static str,
    docs: &'static str,
) -> anyhow::Result<()> {
    let parser = Parser::new(docs);

    // Tracks the content of the current code block if it's using a
    // language supported for analysis
    let mut language = None;
    for event in parser {
        match event {
            // CodeBlock-specific handling
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(meta))) => {
                // Track the content of code blocks to pass them through the analyser
                let test = CodeBlockTest::from_str(meta.as_ref())?;
                language = Some((test, String::new()));
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((test, block)) = language.take() {
                    assert_lint(group, rule, &test, &block)?;
                }
            }
            Event::Text(text) => {
                if let Some((_, block)) = &mut language {
                    write!(block, "{text}")?;
                }
            }
            // We don't care other events
            _ => {}
        }
    }

    Ok(())
}
