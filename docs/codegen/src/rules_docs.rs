use anyhow::{Result, bail};
use convert_case::{Case, Casing};
use pgls_analyse::{AnalysisFilter, AppliesTo, RuleFilter, RuleMetadata};
use pgls_analyser::{AnalysableStatement, Analyser, AnalyserConfig, LinterOptions};
use pgls_console::StdDisplay;
use pgls_diagnostics::{Diagnostic, DiagnosticExt, PrintDiagnostic};
use pgls_query_ext::diagnostics::SyntaxDiagnostic;
use pgls_workspace::settings::Settings;
use pulldown_cmark::{CodeBlockKind, Event, LinkType, Parser, Tag, TagEnd};
use std::{
    fmt::Write as _,
    fs,
    io::{self, Write as _},
    path::Path,
    slice,
    str::{self, FromStr},
};

/// Generates the documentation page for each lint rule.
///
/// * `docs_dir`: Path to the docs directory.
pub fn generate_rules_docs(docs_dir: &Path) -> anyhow::Result<()> {
    let rules_dir = docs_dir.join("reference/rules");

    if rules_dir.exists() {
        fs::remove_dir_all(&rules_dir)?;
    }
    fs::create_dir_all(&rules_dir)?;

    let mut visitor = crate::utils::LintRulesVisitor::default();
    pgls_analyser::visit_registry(&mut visitor);

    let crate::utils::LintRulesVisitor { groups } = visitor;

    for (group, rules) in groups {
        for (rule, metadata) in rules {
            let content = generate_rule_doc(group, rule, metadata)?;
            let dashed_rule = rule.to_case(Case::Kebab);
            fs::write(rules_dir.join(format!("{dashed_rule}.md")), content)?;
        }
    }

    Ok(())
}

fn generate_rule_doc(
    group: &'static str,
    rule: &'static str,
    meta: RuleMetadata,
) -> Result<String> {
    let mut content = Vec::new();

    writeln!(content, "# {rule}")?;
    writeln!(content)?;

    // One compact metadata block instead of a line per property.
    let mut properties = vec![format!("**Group** [`{group}`](../rules.md#{group})")];
    if meta.recommended {
        properties.push("**Recommended**".to_string());
    }
    if meta.applies_to == AppliesTo::Migration {
        properties.push("**Migrations only**".to_string());
    }
    if group == pgls_analyser::TYPECHECK_GROUP {
        properties.push("**Needs a database connection**".to_string());
    }
    let since = if meta.version == "next" {
        "Unreleased".to_string()
    } else {
        meta.version.to_string()
    };
    properties.push(format!("**Since** `{since}`"));
    writeln!(content, "{}  ", properties.join(" · "))?;

    let mut identifiers = vec![format!("**Diagnostic** `lint/{rule}`")];
    let codes = postgres_error_codes(meta.docs);
    if !codes.is_empty() {
        let codes = codes
            .iter()
            .map(|code| format!("`{code}`"))
            .collect::<Vec<_>>()
            .join(", ");
        identifiers.push(format!("**Postgres error** {codes}"));
    }
    writeln!(content, "{}", identifiers.join(" · "))?;
    writeln!(content)?;

    if let Some(reason) = &meta.deprecated {
        writeln!(content, "!!! warning \"Deprecated\"")?;
        writeln!(
            content,
            "    This rule is deprecated and will be removed in the next major release. {reason}"
        )?;
        writeln!(content)?;
    }

    if !meta.sources.is_empty() {
        let sources = meta
            .sources
            .iter()
            .map(|source| {
                format!(
                    "[`{}`]({})",
                    source.to_namespaced_rule_name(),
                    source.to_rule_url()
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(content, "**Sources**: inspired by {sources}")?;
        writeln!(content)?;
    }

    write_documentation(group, rule, meta.docs, &mut content)?;

    write_how_to_configure(rule, &mut content)?;

    write_how_to_suppress(rule, &mut content)?;

    Ok(String::from_utf8(content)?)
}

/// The Postgres error codes a rule's documentation names, like `42P01` in
/// "Postgres raises `42P01 undefined_table`".
fn postgres_error_codes(docs: &str) -> Vec<&str> {
    let mut codes = Vec::new();
    for span in docs.split('`').skip(1).step_by(2) {
        let Some((code, condition)) = span.split_once(' ') else {
            continue;
        };
        let is_code = code.len() == 5
            && code
                .chars()
                .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase());
        let is_condition = !condition.is_empty()
            && condition
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '_');
        if is_code && is_condition && !codes.contains(&code) {
            codes.push(code);
        }
    }
    codes
}

fn write_how_to_configure(rule: &'static str, content: &mut Vec<u8>) -> io::Result<()> {
    writeln!(content, "## How to configure")?;
    let json = format!(
        r#"
{{
  "linter": {{
    "rules": {{
      "{rule}": "error"
    }}
  }}
}}
"#
    );

    writeln!(content, "```json")?;
    writeln!(content, "{json}")?;
    writeln!(content, "```")?;

    Ok(())
}

fn write_how_to_suppress(rule: &'static str, content: &mut Vec<u8>) -> io::Result<()> {
    writeln!(content, "## How to suppress")?;
    writeln!(content)?;
    writeln!(content, "Suppress this diagnostic with a comment:")?;
    writeln!(content)?;
    writeln!(content, "```sql")?;
    writeln!(content, "-- pgls-ignore {rule}")?;
    writeln!(content, "```")?;

    Ok(())
}

/// Parse the documentation fragment for a lint rule (in markdown) and generates
/// the content for the corresponding documentation page
fn write_documentation(
    group: &'static str,
    rule: &'static str,
    docs: &'static str,
    content: &mut Vec<u8>,
) -> Result<()> {
    writeln!(content, "## Description")?;

    let parser = Parser::new(docs);

    // Tracks the content of the current code block if it's using a
    // language supported for analysis
    let mut language = None;
    let mut list_order = None;
    let mut list_indentation = 0;

    // Tracks the type and metadata of the link
    let mut start_link_tag: Option<Tag> = None;

    for event in parser {
        match event {
            // CodeBlock-specific handling
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(meta))) => {
                // Track the content of code blocks to pass them through the analyzer
                let test = CodeBlockTest::from_str(meta.as_ref())?;

                // Erase the lintdoc-specific attributes in the output by
                // re-generating the language ID from the source type
                write!(content, "```{}", test.tag)?;
                writeln!(content)?;

                language = Some((test, String::new()));
            }

            Event::End(TagEnd::CodeBlock) => {
                writeln!(content, "```")?;
                writeln!(content)?;

                if let Some((test, block)) = language.take() {
                    if test.expect_diagnostic {
                        writeln!(content, "```sh")?;
                    }

                    print_diagnostics(group, rule, &test, &block, content)?;

                    if test.expect_diagnostic {
                        writeln!(content, "```")?;
                        writeln!(content)?;
                    }
                }
            }

            Event::Text(text) => {
                let mut hide_line = false;

                if let Some((test, block)) = &mut language {
                    if let Some(inner_text) = text.strip_prefix("# ") {
                        // Lines prefixed with "# " are hidden from the public documentation
                        write!(block, "{inner_text}")?;
                        hide_line = true;
                        test.hidden_lines.push(test.line_count);
                    } else {
                        write!(block, "{text}")?;
                    }
                    test.line_count += 1;
                }

                if hide_line {
                    // Line should not be emitted into the output
                } else if matches!(text.as_ref(), "`" | "*" | "_") {
                    write!(content, "\\{text}")?;
                } else {
                    write!(content, "{text}")?;
                }
            }

            // Other markdown events are emitted as-is
            Event::Start(Tag::Heading { level, .. }) => {
                write!(content, "{} ", "#".repeat(level as usize))?;
            }
            Event::End(TagEnd::Heading { .. }) => {
                writeln!(content)?;
                writeln!(content)?;
            }

            Event::Start(Tag::Paragraph) => {
                continue;
            }
            Event::End(TagEnd::Paragraph) => {
                writeln!(content)?;
                writeln!(content)?;
            }

            Event::Code(text) => {
                write!(content, "`{text}`")?;
            }
            Event::Start(ref link_tag @ Tag::Link { link_type, .. }) => {
                start_link_tag = Some(link_tag.clone());
                match link_type {
                    LinkType::Autolink => {
                        write!(content, "<")?;
                    }
                    LinkType::Inline | LinkType::Reference | LinkType::Shortcut => {
                        write!(content, "[")?;
                    }
                    _ => {
                        panic!("unimplemented link type")
                    }
                }
            }
            Event::End(TagEnd::Link) => {
                if let Some(Tag::Link {
                    link_type,
                    dest_url,
                    title,
                    ..
                }) = start_link_tag
                {
                    match link_type {
                        LinkType::Autolink => {
                            write!(content, ">")?;
                        }
                        LinkType::Inline | LinkType::Reference | LinkType::Shortcut => {
                            write!(content, "]({dest_url}")?;
                            if !title.is_empty() {
                                write!(content, " \"{title}\"")?;
                            }
                            write!(content, ")")?;
                        }
                        _ => {
                            panic!("unimplemented link type")
                        }
                    }
                    start_link_tag = None;
                } else {
                    panic!("missing start link tag");
                }
            }

            Event::SoftBreak => {
                writeln!(content)?;
            }

            Event::HardBreak => {
                writeln!(content, "<br />")?;
            }

            Event::Start(Tag::List(num)) => {
                list_indentation += 1;
                if let Some(num) = num {
                    list_order = Some(num);
                }
                if list_indentation > 1 {
                    writeln!(content)?;
                }
            }

            Event::End(TagEnd::List(_)) => {
                list_order = None;
                list_indentation -= 1;
                writeln!(content)?;
            }
            Event::Start(Tag::Item) => {
                write!(content, "{}", "  ".repeat(list_indentation - 1))?;
                if let Some(num) = list_order {
                    write!(content, "{num}. ")?;
                } else {
                    write!(content, "- ")?;
                }
            }

            Event::End(TagEnd::Item) => {
                list_order = list_order.map(|item| item + 1);
                writeln!(content)?;
            }

            Event::Start(Tag::Strong) => {
                write!(content, "**")?;
            }

            Event::End(TagEnd::Strong) => {
                write!(content, "**")?;
            }

            Event::Start(Tag::Emphasis) => {
                write!(content, "_")?;
            }

            Event::End(TagEnd::Emphasis) => {
                write!(content, "_")?;
            }

            Event::Start(Tag::Strikethrough) => {
                write!(content, "~")?;
            }

            Event::End(TagEnd::Strikethrough) => {
                write!(content, "~")?;
            }

            Event::Start(Tag::BlockQuote(_)) => {
                write!(content, ">")?;
            }

            Event::End(TagEnd::BlockQuote(_)) => {
                writeln!(content)?;
            }

            _ => {
                bail!("unimplemented event {event:?}")
            }
        }
    }

    Ok(())
}

struct CodeBlockTest {
    /// The language tag of this code block.
    tag: String,

    /// True if this is an invalid example that should trigger a diagnostic.
    expect_diagnostic: bool,

    /// Whether to ignore this code block.
    ignore: bool,

    /// The number of lines in this code block.
    line_count: u32,

    // The indices of lines that should be hidden from the public documentation.
    hidden_lines: Vec<u32>,
}

impl FromStr for CodeBlockTest {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
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
            line_count: 0,
            hidden_lines: vec![],
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

/// Prints diagnostics documentation from a gode block into the content buffer.
///
/// * `group`: The group of the rule.
/// * `rule`: The rule name.
/// * `test`: The code block test.
/// * `code`: The code block content.
/// * `content`: The buffer to write the documentation to.
fn print_diagnostics(
    group: &'static str,
    rule: &'static str,
    test: &CodeBlockTest,
    code: &str,
    content: &mut Vec<u8>,
) -> Result<()> {
    let file_path = format!("code-block.{}", test.tag);

    let mut write_diagnostic = |_: &str, diag: pgls_diagnostics::Error| -> Result<()> {
        let printer = PrintDiagnostic::simple(&diag);
        writeln!(content, "{}", StdDisplay(printer)).unwrap();

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
    let options = LinterOptions::default();
    let analyser = Analyser::new(AnalyserConfig {
        options: &options,
        filter,
    });

    // Analyse all statements of the block together, as rules can depend on earlier statements.
    let result = pgls_statement_splitter::split(code);
    let mut stmts = Vec::new();
    for stmt_range in &result.ranges {
        let sql = &code[*stmt_range];
        match pgls_query::parse(sql) {
            Ok(ast) => {
                if let Some(root) = ast.into_root() {
                    stmts.push(AnalysableStatement::new(root, *stmt_range).with_sql(sql));
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

    // Typecheck rules need a database. Examples run against the built-in catalog of a fresh
    // database on the latest supported Postgres, and create everything else themselves.
    let is_typecheck = group == pgls_analyser::TYPECHECK_GROUP;
    let diagnostics = analyser.run(pgls_analyser::AnalyserParams {
        stmts,
        catalog_base: is_typecheck.then(builtin_catalog),
        search_path: vec!["public".into()],
        typecheck: is_typecheck,
        ..Default::default()
    });
    for rule_diag in diagnostics {
        let diag = pgls_diagnostics::serde::Diagnostic::new(rule_diag);

        let category = diag.category().expect("linter diagnostic has no code");
        let severity = settings
            .get_severity_from_rule_code(category)
            .expect("If you see this error, it means you need to run cargo codegen-configuration");

        let error = diag
            .with_severity(severity)
            .with_file_path(&file_path)
            .with_file_source_code(code);

        write_diagnostic(code, error)?;
    }

    Ok(())
}

/// The catalog of a fresh database on the latest Postgres version with regression fixtures.
fn builtin_catalog() -> std::sync::Arc<pgls_catalog::CatalogBase> {
    static CATALOG: std::sync::OnceLock<std::sync::Arc<pgls_catalog::CatalogBase>> =
        std::sync::OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let version = pgls_postgres_regress::versions()
                .pop()
                .expect("regression fixtures for at least one version");
            let snapshot: pgls_catalog::Snapshot =
                serde_json::from_str(&version.catalog_json()).expect("parse the built-in catalog");
            std::sync::Arc::new(pgls_catalog::CatalogBase::new(std::sync::Arc::new(
                snapshot,
            )))
        })
        .clone()
}
