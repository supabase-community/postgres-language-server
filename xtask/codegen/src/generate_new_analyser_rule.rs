use biome_string_case::Case;
use bpaf::Bpaf;
use pgls_diagnostics::Severity;
use pgls_env::PGLS_WEBSITE;
use std::str::FromStr;
use xtask::project_root;

#[derive(Debug, Clone, Bpaf)]
pub enum Category {
    /// Lint rules
    Lint,
}

impl FromStr for Category {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "lint" => Ok(Self::Lint),
            _ => Err("Not supported"),
        }
    }
}

fn generate_rule_template(
    category: &Category,
    rule_name_upper_camel: &str,
    rule_name_lower_camel: &str,
    severity: Severity,
) -> String {
    let macro_name = match category {
        Category::Lint => "declare_lint_rule",
    };

    let severity_code = match severity {
        Severity::Hint => "Severity::Hint",
        Severity::Information => "Severity::Information",
        Severity::Warning => "Severity::Warning",
        Severity::Error => "Severity::Error",
        Severity::Fatal => "Severity::Fatal",
    };

    format!(
        r#"use pgls_analyse::{{
    AnalysedFileContext, context::RuleContext, {macro_name}, Rule, RuleDiagnostic,
}};
use pgls_console::markup;
use pgls_diagnostics::Severity;
use pgls_schema_cache::SchemaCache;

{macro_name}! {{
    /// Succinct description of the rule.
    ///
    /// Put context and details about the rule.
    ///
    /// Try to stay consistent with the descriptions of implemented rules.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// select 1;
    /// ```
    ///
    /// ### Valid
    ///
    /// ``sql`
    /// select 2;
    /// ```
    ///
    pub {rule_name_upper_camel} {{
        version: "next",
        name: "{rule_name_lower_camel}",
        severity: {severity_code},
        recommended: false,
    }}
}}

impl Rule for {rule_name_upper_camel} {{
    type Options = ();

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {{
        Vec::new()
    }}
}}
"#
    )
}

fn gen_sql(category_name: &str) -> String {
    format!("-- expect_only_{category_name}\n-- select 1;")
}

pub fn generate_new_analyser_rule(
    category: Category,
    rule_name: &str,
    group: &str,
    severity: Severity,
) {
    let rule_name_camel = Case::Camel.convert(rule_name);
    let crate_folder = project_root().join("crates/pgls_analyser");
    let rule_folder = match &category {
        Category::Lint => crate_folder.join(format!("src/lint/{group}")),
    };
    if !rule_folder.exists() {
        std::fs::create_dir(rule_folder.clone()).expect("To create the rule folder");
    }

    // Generate rule code
    let code = generate_rule_template(
        &category,
        Case::Pascal.convert(rule_name).as_str(),
        rule_name_camel.as_str(),
        severity,
    );
    let file_name = format!(
        "{}/{}.rs",
        rule_folder.display(),
        Case::Snake.convert(rule_name)
    );
    std::fs::write(file_name.clone(), code).unwrap_or_else(|_| panic!("To write {}", &file_name));

    let categories_path = "crates/pgls_diagnostics_categories/src/categories.rs";
    let mut categories = std::fs::read_to_string(categories_path).unwrap();

    if !categories.contains(&rule_name_camel) {
        let kebab_case_rule = Case::Kebab.convert(&rule_name_camel);
        // We sort rules to reduce conflicts between contributions made in parallel.
        let rule_line = match category {
            Category::Lint => format!(
                r#"    "lint/{group}/{rule_name_camel}": "{PGLS_WEBSITE}/latest/rules/{kebab_case_rule}","#
            ),
        };
        let lint_start = match category {
            Category::Lint => "define_categories! {\n",
        };
        let lint_end = match category {
            Category::Lint => "\n    // end lint rules\n",
        };
        debug_assert!(categories.contains(lint_start), "{}", lint_start);
        debug_assert!(categories.contains(lint_end), "{}", lint_end);
        let lint_start_index = categories.find(lint_start).unwrap() + lint_start.len();
        let lint_end_index = categories.find(lint_end).unwrap();
        let lint_rule_text = &categories[lint_start_index..lint_end_index];
        let mut lint_rules: Vec<_> = lint_rule_text.lines().chain(Some(&rule_line[..])).collect();
        lint_rules.sort_unstable();
        let new_lint_rule_text = lint_rules.join("\n");
        categories.replace_range(lint_start_index..lint_end_index, &new_lint_rule_text);
        std::fs::write(categories_path, categories).unwrap();
    }

    let test_group_folder = match &category {
        Category::Lint => crate_folder.join(format!("tests/specs/{group}")),
    };
    if !test_group_folder.exists() {
        std::fs::create_dir(test_group_folder.clone()).expect("To create the test group folder");
    }

    let test_folder = match &category {
        Category::Lint => crate_folder.join(format!("tests/specs/{group}/{rule_name_camel}")),
    };
    if !test_folder.exists() {
        std::fs::create_dir(test_folder.clone()).expect("To create the test rule folder");
    }

    let test_file_name = format!("{}/basic.sql", test_folder.display());
    std::fs::write(
        test_file_name.clone(),
        gen_sql(format!("lint/{group}/{rule_name_camel}").as_str()),
    )
    .unwrap_or_else(|_| panic!("To write {}", &test_file_name));
}
