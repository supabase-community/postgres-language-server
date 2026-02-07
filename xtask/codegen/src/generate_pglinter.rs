use anyhow::{Context, Result};
use biome_string_case::Case;
use quote::{format_ident, quote};
use regex::Regex;
use std::collections::BTreeMap;
use std::path::Path;
use xtask::{glue::fs2, project_root, Mode};

use crate::update;

/// Metadata extracted from rules.sql INSERT statements
#[derive(Debug, Clone)]
struct PglinterRuleMeta {
    /// Rule name in PascalCase (e.g., "HowManyTableWithoutPrimaryKey")
    name: String,
    /// Rule name in snake_case (e.g., "how_many_table_without_primary_key")
    snake_name: String,
    /// Rule name in camelCase (e.g., "howManyTableWithoutPrimaryKey")
    camel_name: String,
    /// Rule code (e.g., "B001")
    code: String,
    /// Scope: BASE, SCHEMA, or CLUSTER
    scope: String,
    /// Description of the rule
    description: String,
    /// Message template with placeholders
    #[allow(dead_code)]
    message: String,
    /// Suggested fixes
    fixes: Vec<String>,
    /// Warning threshold percentage
    warning_level: i32,
    /// Error threshold percentage
    error_level: i32,
}

/// Parse pglinter rules from rules.sql and generate Rust code
pub fn generate_pglinter() -> Result<()> {
    let rules_sql_path = project_root().join("crates/pgls_pglinter/vendor/sql/rules.sql");

    if !rules_sql_path.exists() {
        anyhow::bail!(
            "Vendor files not found at crates/pgls_pglinter/vendor/sql/rules.sql. Run 'cargo build -p pgls_pglinter' first to download them."
        );
    }

    let sql_content = fs2::read_to_string(&rules_sql_path)?;
    let rules = parse_rules_sql(&sql_content)?;

    // Generate rule files
    generate_rule_trait()?;
    generate_rule_files(&rules)?;
    generate_registry(&rules)?;
    update_categories_file(&rules)?;

    Ok(())
}

/// Parse INSERT statements from rules.sql to extract rule metadata
fn parse_rules_sql(content: &str) -> Result<BTreeMap<String, PglinterRuleMeta>> {
    let mut rules = BTreeMap::new();

    // Normalize the content: remove newlines within parentheses to make regex easier
    // This handles multi-line ARRAY declarations
    let normalized = normalize_sql_values(content);

    // Use regex to find value tuples
    // Pattern: ('Name', 'CODE', num, num, 'SCOPE', 'desc', 'msg', ARRAY[...])
    let value_pattern = Regex::new(
        r#"\(\s*'([^']+)',\s*'([^']+)',\s*(\d+),\s*(\d+),\s*'([^']+)',\s*'([^']+)',\s*'([^']+)',\s*ARRAY\s*\[(.*?)\]\s*\)"#,
    )?;

    for caps in value_pattern.captures_iter(&normalized) {
        let name = caps.get(1).unwrap().as_str().to_string();
        let code = caps.get(2).unwrap().as_str().to_string();
        let warning_level: i32 = caps.get(3).unwrap().as_str().parse()?;
        let error_level: i32 = caps.get(4).unwrap().as_str().parse()?;
        let scope = caps.get(5).unwrap().as_str().to_string();
        let description = caps
            .get(6)
            .unwrap()
            .as_str()
            .replace("''", "'") // Unescape single quotes
            .to_string();
        let message = caps.get(7).unwrap().as_str().to_string();
        let fixes_str = caps.get(8).unwrap().as_str();

        // Parse fixes array
        let fixes: Vec<String> = parse_fixes_array(fixes_str);

        let snake_name = Case::Snake.convert(&name);
        let camel_name = to_camel_case(&name);

        let meta = PglinterRuleMeta {
            name,
            snake_name: snake_name.clone(),
            camel_name,
            code,
            scope,
            description,
            message,
            fixes,
            warning_level,
            error_level,
        };

        rules.insert(snake_name, meta);
    }

    if rules.is_empty() {
        anyhow::bail!("No rules found in rules.sql. Check the file format.");
    }

    Ok(rules)
}

/// Normalize SQL content by joining lines within value tuples
fn normalize_sql_values(content: &str) -> String {
    let mut result = String::new();
    let mut in_value = false;
    let mut paren_depth = 0;

    for c in content.chars() {
        match c {
            '(' => {
                paren_depth += 1;
                in_value = true;
                result.push(c);
            }
            ')' => {
                paren_depth -= 1;
                if paren_depth == 0 {
                    in_value = false;
                }
                result.push(c);
            }
            '\n' | '\r' if in_value => {
                result.push(' '); // Replace newlines with spaces inside values
            }
            _ => result.push(c),
        }
    }

    result
}

/// Parse ARRAY['fix1', 'fix2'] into Vec<String>
fn parse_fixes_array(s: &str) -> Vec<String> {
    let fix_pattern = Regex::new(r#"'([^']+)'"#).unwrap();
    fix_pattern
        .captures_iter(s)
        .map(|cap| cap.get(1).unwrap().as_str().to_string())
        .collect()
}

/// Convert PascalCase to camelCase
fn to_camel_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

/// Map scope to category directory name
fn scope_to_category(scope: &str) -> &'static str {
    match scope {
        "BASE" => "base",
        "SCHEMA" => "schema",
        "CLUSTER" => "cluster",
        _ => "base",
    }
}

/// Generate src/rule.rs with PglinterRule trait
fn generate_rule_trait() -> Result<()> {
    let rule_path = project_root().join("crates/pgls_pglinter/src/rule.rs");

    let content = quote! {
        //! Generated file, do not edit by hand, see `xtask/codegen`

        use pgls_analyse::RuleMeta;

        /// Trait for pglinter (database-level) rules
        ///
        /// Pglinter rules are different from linter rules:
        /// - They execute SQL queries against the database via pglinter extension
        /// - They don't have AST-based execution
        /// - Rule logic is in the pglinter Postgres extension
        /// - Threshold configuration (warning/error levels) is handled by pglinter extension
        pub trait PglinterRule: RuleMeta {
            /// Rule code (e.g., "B001", "S001", "C001")
            const CODE: &'static str;

            /// Rule scope (BASE, SCHEMA, or CLUSTER)
            const SCOPE: &'static str;

            /// Description of what the rule detects
            const DESCRIPTION: &'static str;

            /// Suggested fixes for violations
            const FIXES: &'static [&'static str];
        }
    };

    let formatted = xtask::reformat(content)?;
    update(&rule_path, &formatted, &Mode::Overwrite)?;

    Ok(())
}

/// Generate rule files in src/rules/{category}/{rule_name}.rs
fn generate_rule_files(rules: &BTreeMap<String, PglinterRuleMeta>) -> Result<()> {
    let rules_dir = project_root().join("crates/pgls_pglinter/src/rules");

    // Group rules by scope/category
    let mut rules_by_category: BTreeMap<String, Vec<&PglinterRuleMeta>> = BTreeMap::new();
    for rule in rules.values() {
        let category = scope_to_category(&rule.scope).to_string();
        rules_by_category.entry(category).or_default().push(rule);
    }

    // Generate category directories and files
    for (category, category_rules) in &rules_by_category {
        let category_dir = rules_dir.join(category);
        fs2::create_dir_all(&category_dir)?;

        // Generate individual rule files
        for rule in category_rules {
            generate_rule_file(&category_dir, rule)?;
        }

        // Generate category mod.rs
        generate_category_mod(&category_dir, category, category_rules)?;
    }

    // Generate main rules/mod.rs
    generate_rules_mod(&rules_dir, &rules_by_category)?;

    Ok(())
}

/// Generate individual rule file
fn generate_rule_file(category_dir: &Path, rule: &PglinterRuleMeta) -> Result<()> {
    let rule_file = category_dir.join(format!("{}.rs", rule.snake_name));

    let struct_name = format_ident!("{}", rule.name);
    let camel_name = &rule.camel_name;
    let code = &rule.code;
    let scope = &rule.scope;
    let description = &rule.description;
    let warning_level = rule.warning_level;
    let error_level = rule.error_level;
    let category = scope_to_category(&rule.scope);

    // Create fixes as static slice
    let fixes: Vec<&str> = rule.fixes.iter().map(|s| s.as_str()).collect();

    // Build doc string
    let doc_string = format!(
        r#"# {} ({})

{}

## Configuration

Enable or disable this rule in your configuration:

```json
{{
  "pglinter": {{
    "rules": {{
      "{}": {{
        "{}": "warn"
      }}
    }}
  }}
}}
```

## Thresholds

- Warning level: {}%
- Error level: {}%

## Fixes

{}

## Documentation

See: <https://github.com/pmpetit/pglinter#{}>"#,
        rule.name,
        code,
        description,
        category,
        camel_name,
        warning_level,
        error_level,
        rule.fixes
            .iter()
            .map(|f| format!("- {f}"))
            .collect::<Vec<_>>()
            .join("\n"),
        code.to_lowercase(),
    );

    let content = quote! {
        //! Generated file, do not edit by hand, see `xtask/codegen`

        use crate::rule::PglinterRule;

        ::pgls_analyse::declare_rule! {
            #[doc = #doc_string]
            pub #struct_name {
                version: "1.0.0",
                name: #camel_name,
                severity: pgls_diagnostics::Severity::Warning,
                recommended: true,
            }
        }

        impl PglinterRule for #struct_name {
            const CODE: &'static str = #code;
            const SCOPE: &'static str = #scope;
            const DESCRIPTION: &'static str = #description;
            const FIXES: &'static [&'static str] = &[#(#fixes),*];
        }
    };

    let formatted = xtask::reformat(content)?;
    update(&rule_file, &formatted, &Mode::Overwrite)?;

    Ok(())
}

/// Generate category mod.rs that exports all rules
fn generate_category_mod(
    category_dir: &Path,
    category: &str,
    rules: &[&PglinterRuleMeta],
) -> Result<()> {
    let mod_file = category_dir.join("mod.rs");

    let category_title = Case::Pascal.convert(category);
    let category_struct = format_ident!("{}", category_title);

    // Generate mod declarations
    let mod_names: Vec<_> = rules
        .iter()
        .map(|r| format_ident!("{}", r.snake_name))
        .collect();

    // Generate rule paths for declare_lint_group!
    let rule_paths: Vec<_> = rules
        .iter()
        .map(|r| {
            let mod_name = format_ident!("{}", r.snake_name);
            let struct_name = format_ident!("{}", r.name);
            quote! { self::#mod_name::#struct_name }
        })
        .collect();

    let content = quote! {
        //! Generated file, do not edit by hand, see `xtask/codegen`

        #( pub mod #mod_names; )*

        ::pgls_analyse::declare_lint_group! {
            pub #category_struct {
                name: #category,
                rules: [
                    #( #rule_paths, )*
                ]
            }
        }
    };

    let formatted = xtask::reformat(content)?;
    update(&mod_file, &formatted, &Mode::Overwrite)?;

    Ok(())
}

/// Generate main rules/mod.rs
fn generate_rules_mod(
    rules_dir: &Path,
    rules_by_category: &BTreeMap<String, Vec<&PglinterRuleMeta>>,
) -> Result<()> {
    let mod_file = rules_dir.join("mod.rs");

    let category_mods: Vec<_> = rules_by_category
        .keys()
        .map(|cat| {
            let mod_name = format_ident!("{}", cat);
            quote! { pub mod #mod_name; }
        })
        .collect();

    // Generate group paths for declare_category!
    let group_paths: Vec<_> = rules_by_category
        .keys()
        .map(|cat| {
            let mod_name = format_ident!("{}", cat);
            let group_name = format_ident!("{}", Case::Pascal.convert(cat));
            quote! { self::#mod_name::#group_name }
        })
        .collect();

    let content = quote! {
        //! Generated file, do not edit by hand, see `xtask/codegen`

        #( #category_mods )*

        ::pgls_analyse::declare_category! {
            pub PgLinter {
                kind: Lint,
                groups: [
                    #( #group_paths, )*
                ]
            }
        }
    };

    let formatted = xtask::reformat(content)?;
    update(&mod_file, &formatted, &Mode::Overwrite)?;

    Ok(())
}

/// Generate src/registry.rs with visit_registry() and get_rule_category()
fn generate_registry(rules: &BTreeMap<String, PglinterRuleMeta>) -> Result<()> {
    let registry_path = project_root().join("crates/pgls_pglinter/src/registry.rs");

    // Generate match arms for rule code lookup (camelCase → code)
    let code_arms: Vec<_> = rules
        .values()
        .map(|rule| {
            let camel_name = &rule.camel_name;
            let code = &rule.code;
            quote! {
                #camel_name => Some(#code)
            }
        })
        .collect();

    // Generate match arms for category lookup (code → &'static Category)
    let category_arms: Vec<_> = rules
        .values()
        .map(|rule| {
            let code = &rule.code;
            let category = scope_to_category(&rule.scope);
            let camel_name = &rule.camel_name;
            let category_path = format!("pglinter/{category}/{camel_name}");

            quote! {
                #code => Some(::pgls_diagnostics::category!(#category_path))
            }
        })
        .collect();

    // Generate match arms for rule metadata lookup by name
    let metadata_arms: Vec<_> = rules
        .values()
        .map(|rule| {
            let camel_name = &rule.camel_name;
            let code = &rule.code;
            let scope = &rule.scope;
            let description = &rule.description;
            let fixes: Vec<&str> = rule.fixes.iter().map(|s| s.as_str()).collect();

            quote! {
                #camel_name => Some(RuleMetadata {
                    code: #code,
                    name: #camel_name,
                    scope: #scope,
                    description: #description,
                    fixes: &[#(#fixes),*],
                })
            }
        })
        .collect();

    // Generate match arms for rule metadata lookup by code
    let metadata_by_code_arms: Vec<_> = rules
        .values()
        .map(|rule| {
            let camel_name = &rule.camel_name;
            let code = &rule.code;
            let scope = &rule.scope;
            let description = &rule.description;
            let fixes: Vec<&str> = rule.fixes.iter().map(|s| s.as_str()).collect();

            quote! {
                #code => Some(RuleMetadata {
                    code: #code,
                    name: #camel_name,
                    scope: #scope,
                    description: #description,
                    fixes: &[#(#fixes),*],
                })
            }
        })
        .collect();

    let content = quote! {
        //! Generated file, do not edit by hand, see `xtask/codegen`

        use pgls_analyse::RegistryVisitor;
        use pgls_diagnostics::Category;

        /// Metadata for a pglinter rule
        #[derive(Debug, Clone, Copy)]
        pub struct RuleMetadata {
            /// Rule code (e.g., "B001")
            pub code: &'static str,
            /// Rule name in camelCase
            pub name: &'static str,
            /// Rule scope (BASE, SCHEMA, CLUSTER)
            pub scope: &'static str,
            /// Description of what the rule detects
            pub description: &'static str,
            /// Suggested fixes
            pub fixes: &'static [&'static str],
        }

        /// Visit all pglinter rules using the visitor pattern
        pub fn visit_registry<V: RegistryVisitor>(registry: &mut V) {
            registry.record_category::<crate::rules::PgLinter>();
        }

        /// Get the pglinter rule code from the camelCase name
        pub fn get_rule_code(name: &str) -> Option<&'static str> {
            match name {
                #( #code_arms, )*
                _ => None,
            }
        }

        /// Get the diagnostic category for a rule code
        pub fn get_rule_category(code: &str) -> Option<&'static Category> {
            match code {
                #( #category_arms, )*
                _ => None,
            }
        }

        /// Get rule metadata by name (camelCase)
        pub fn get_rule_metadata(name: &str) -> Option<RuleMetadata> {
            match name {
                #( #metadata_arms, )*
                _ => None,
            }
        }

        /// Get rule metadata by code (e.g., "B001", "S001", "C001")
        pub fn get_rule_metadata_by_code(code: &str) -> Option<RuleMetadata> {
            match code {
                #( #metadata_by_code_arms, )*
                _ => None,
            }
        }
    };

    let formatted = xtask::reformat(content)?;
    update(&registry_path, &formatted, &Mode::Overwrite)?;

    Ok(())
}

/// Update the categories.rs file with pglinter rules
fn update_categories_file(rules: &BTreeMap<String, PglinterRuleMeta>) -> Result<()> {
    let categories_path =
        project_root().join("crates/pgls_diagnostics_categories/src/categories.rs");

    let mut content = fs2::read_to_string(&categories_path)?;

    // Generate pglinter rule entries grouped by category
    let mut pglinter_rules: Vec<(String, String)> = rules
        .values()
        .map(|rule| {
            let category = scope_to_category(&rule.scope);
            let url = format!(
                "https://github.com/pmpetit/pglinter#{}",
                rule.code.to_lowercase()
            );

            (
                category.to_string(),
                format!(
                    "    \"pglinter/{}/{}\": \"{}\",",
                    category, rule.camel_name, url
                ),
            )
        })
        .collect();

    // Sort by category, then by entry
    pglinter_rules.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    // Add meta diagnostics at the start
    let mut all_entries = vec![
        "    // Meta diagnostics".to_string(),
        "    \"pglinter/extensionNotInstalled\": \"Install the pglinter extension with: CREATE EXTENSION pglinter\",".to_string(),
        "    \"pglinter/ruleDisabledInExtension\": \"Enable the rule in the extension with: UPDATE pglinter.rules SET enable = true WHERE code = '<code>'\",".to_string(),
    ];

    // Add rule categories
    let mut current_category = String::new();
    for (category, entry) in &pglinter_rules {
        if category != &current_category {
            current_category = category.clone();
            all_entries.push(format!(
                "    // {} rules ({}-series)",
                Case::Pascal.convert(category),
                match category.as_str() {
                    "base" => "B",
                    "schema" => "S",
                    "cluster" => "C",
                    _ => "?",
                }
            ));
        }
        all_entries.push(entry.clone());
    }

    let pglinter_entries = all_entries.join("\n");

    // Replace content between pglinter rules markers
    let rules_start = "// pglinter rules start";
    let rules_end = "// pglinter rules end";

    content = replace_between_markers(
        &content,
        rules_start,
        rules_end,
        &format!("\n{pglinter_entries}\n    "),
    )?;

    // Generate pglinter group entries
    let mut categories: Vec<String> = pglinter_rules.iter().map(|(cat, _)| cat.clone()).collect();
    categories.sort();
    categories.dedup();

    let mut group_entries = vec!["    \"pglinter\",".to_string()];
    for category in categories {
        group_entries.push(format!("    \"pglinter/{category}\","));
    }
    let groups_content = group_entries.join("\n");

    // Replace content between pglinter groups markers
    let groups_start = "// Pglinter groups start";
    let groups_end = "// Pglinter groups end";

    content = replace_between_markers(
        &content,
        groups_start,
        groups_end,
        &format!("\n{groups_content}\n    "),
    )?;

    fs2::write(categories_path, content)?;

    Ok(())
}

/// Replace content between two markers
fn replace_between_markers(
    content: &str,
    start_marker: &str,
    end_marker: &str,
    new_content: &str,
) -> Result<String> {
    let start_pos = content
        .find(start_marker)
        .with_context(|| format!("Could not find '{start_marker}' marker"))?;

    let end_pos = content
        .find(end_marker)
        .with_context(|| format!("Could not find '{end_marker}' marker"))?;

    let mut result = String::new();
    result.push_str(&content[..start_pos + start_marker.len()]);
    result.push_str(new_content);
    result.push_str(&content[end_pos..]);

    Ok(result)
}
