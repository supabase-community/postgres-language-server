use anyhow::{Context, Result};
use biome_string_case::Case;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use xtask::{glue::fs2, project_root, Mode};

use crate::update;

/// Metadata extracted from SQL file comments
#[derive(Debug, Clone)]
struct SqlRuleMetadata {
    /// Rule name in camelCase (from meta comment)
    name: String,
    /// Rule name in snake_case (from filename)
    snake_name: String,
    /// Human-readable title
    title: String,
    /// Severity level (INFO, WARN, ERROR)
    severity: String,
    /// Category (PERFORMANCE, SECURITY, etc.)
    category: String,
    /// Description of what the rule detects
    description: String,
    /// Remediation URL or text
    remediation: String,
    /// Whether this rule requires Supabase roles (anon, authenticated, service_role)
    requires_supabase: bool,
    /// The actual SQL query content (everything after metadata comments)
    sql_query: String,
    /// Path to SQL file relative to vendor/
    sql_file_path: PathBuf,
}

/// Generate splinter rules, registry, and categories from individual SQL files
pub fn generate_splinter() -> Result<()> {
    let vendor_dir = project_root().join("crates/pgls_splinter/vendor");

    // Scan for SQL files in performance/ and security/ directories
    let mut all_rules = BTreeMap::new();

    for category in &["performance", "security"] {
        let category_dir = vendor_dir.join(category);
        if !category_dir.exists() {
            continue;
        }

        for entry in fs::read_dir(&category_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                let metadata = extract_metadata_from_sql(&path, category)?;
                all_rules.insert(metadata.snake_name.clone(), metadata);
            }
        }
    }

    // Generate Rust rule files
    generate_rule_trait()?;
    generate_rule_files(&all_rules)?;
    generate_registry(&all_rules)?;

    // Update categories file (keep existing logic for backward compat)
    update_categories_file(&all_rules)?;

    Ok(())
}

/// Extract metadata from SQL file comment headers
fn extract_metadata_from_sql(sql_path: &Path, category: &str) -> Result<SqlRuleMetadata> {
    let content = fs::read_to_string(sql_path)
        .with_context(|| format!("Failed to read SQL file: {sql_path:?}"))?;

    let mut name = None;
    let mut title = None;
    let mut severity = None;
    let mut meta_category = None;
    let mut description = None;
    let mut remediation = None;
    let mut requires_supabase = None;

    // Parse metadata comments and find where SQL starts
    let mut sql_start_line = 0;
    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if !line.starts_with("--") {
            sql_start_line = idx;
            break; // Stop at first non-comment line
        }

        if line.starts_with("-- meta:") {
            let meta_line = &line[8..].trim(); // Remove "-- meta:"

            if let Some(value) = extract_meta_value(meta_line, "name") {
                name = Some(value);
            } else if let Some(value) = extract_meta_value(meta_line, "title") {
                title = Some(value);
            } else if let Some(value) = extract_meta_value(meta_line, "severity") {
                severity = Some(value);
            } else if let Some(value) = extract_meta_value(meta_line, "category") {
                meta_category = Some(value);
            } else if let Some(value) = extract_meta_value(meta_line, "description") {
                description = Some(value);
            } else if let Some(value) = extract_meta_value(meta_line, "remediation") {
                remediation = Some(value);
            } else if let Some(value) = extract_meta_value(meta_line, "requires_supabase") {
                requires_supabase = Some(value.to_lowercase() == "true");
            }
        }
    }

    // Extract SQL query (everything after metadata comments)
    let sql_query: String = content
        .lines()
        .skip(sql_start_line)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    // Get snake_case name from filename
    let snake_name = sql_path
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Invalid filename")?
        .to_string();

    // Build metadata
    let name = name.context("Missing 'name' in metadata comments")?;
    let title = title.context("Missing 'title' in metadata comments")?;
    let severity = severity.context("Missing 'severity' in metadata comments")?;
    let category_from_meta = meta_category.context("Missing 'category' in metadata comments")?;
    let description = description.context("Missing 'description' in metadata comments")?;
    let remediation = remediation.unwrap_or_else(|| {
        "https://supabase.com/docs/guides/database/database-advisors".to_string()
    });
    let requires_supabase = requires_supabase.unwrap_or(false);

    // Verify category matches directory
    if category_from_meta.to_lowercase() != category {
        anyhow::bail!(
            "Category mismatch: file in {category}/ but metadata says {category_from_meta}"
        );
    }

    let sql_file_path = PathBuf::from(category).join(format!("{snake_name}.sql"));

    Ok(SqlRuleMetadata {
        name,
        snake_name,
        title,
        severity,
        category: category_from_meta,
        description,
        remediation,
        requires_supabase,
        sql_query,
        sql_file_path,
    })
}

/// Extract value from metadata line like "name = value"
fn extract_meta_value(line: &str, key: &str) -> Option<String> {
    if let Some(pos) = line.find(&format!("{key} =")) {
        let value_start = pos + key.len() + " =".len();
        let value = line[value_start..].trim();
        return Some(value.to_string());
    }
    None
}

/// Generate src/rule.rs with SplinterRule trait
fn generate_rule_trait() -> Result<()> {
    let rule_path = project_root().join("crates/pgls_splinter/src/rule.rs");

    let content = quote! {
        //! Generated file, do not edit by hand, see `xtask/codegen`

        use pgls_analyse::RuleMeta;

        /// Trait for splinter (database-level) rules
        ///
        /// Splinter rules are different from linter rules:
        /// - They execute SQL queries against the database
        /// - They don't have AST-based execution
        /// - Rule logic is in SQL files, not Rust
        pub trait SplinterRule: RuleMeta {
            /// Path to the SQL file containing the rule query
            const SQL_FILE_PATH: &'static str;

            /// Description of what the rule detects
            const DESCRIPTION: &'static str;

            /// URL to documentation/remediation guide
            const REMEDIATION: &'static str;

            /// Whether this rule requires Supabase roles (anon, authenticated, service_role)
            const REQUIRES_SUPABASE: bool;
        }
    };

    let formatted = xtask::reformat(content)?;
    update(&rule_path, &formatted, &Mode::Overwrite)?;

    Ok(())
}

/// Generate rule files in src/rules/{category}/{rule_name}.rs
fn generate_rule_files(rules: &BTreeMap<String, SqlRuleMetadata>) -> Result<()> {
    let rules_dir = project_root().join("crates/pgls_splinter/src/rules");

    // Group rules by category
    let mut rules_by_category: BTreeMap<String, Vec<&SqlRuleMetadata>> = BTreeMap::new();
    for rule in rules.values() {
        rules_by_category
            .entry(rule.category.to_lowercase())
            .or_default()
            .push(rule);
    }

    // Generate category mod files and rule files
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

    // Generate main rules mod.rs
    generate_rules_mod(&rules_dir, &rules_by_category)?;

    Ok(())
}

/// Generate individual rule file
fn generate_rule_file(category_dir: &Path, metadata: &SqlRuleMetadata) -> Result<()> {
    let rule_file = category_dir.join(format!("{}.rs", metadata.snake_name));

    let struct_name = Case::Pascal.convert(&metadata.snake_name);
    let struct_name = format_ident!("{}", struct_name);

    // These will be used as string literals in the quote!
    let title = &metadata.title;
    let description = &metadata.description;
    let name = &metadata.name; // camelCase name
    let remediation = &metadata.remediation;
    let sql_query = &metadata.sql_query;
    let category_lower = metadata.category.to_lowercase();
    let sql_path = metadata.sql_file_path.display().to_string();
    let requires_supabase = metadata.requires_supabase;

    // Parse severity - this will be a Rust expression
    let severity = match metadata.severity.as_str() {
        "INFO" => quote! { pgls_diagnostics::Severity::Information },
        "WARN" => quote! { pgls_diagnostics::Severity::Warning },
        "ERROR" => quote! { pgls_diagnostics::Severity::Error },
        _ => quote! { pgls_diagnostics::Severity::Information },
    };

    // Build comprehensive documentation
    let requires_supabase_note = if requires_supabase {
        "\n/// \n/// **Note:** This rule requires Supabase roles (`anon`, `authenticated`, `service_role`). \n/// It will be automatically skipped if these roles don't exist in your database.".to_string()
    } else {
        String::new()
    };

    let doc_string = format!(
        r#"/// # {title}
///
/// {description}{requires_supabase_note}
///
/// ## SQL Query
///
/// ```sql
{sql_query_commented}
/// ```
///
/// ## Configuration
///
/// Enable or disable this rule in your configuration:
///
/// ```json
/// {{
///   "splinter": {{
///     "rules": {{
///       "{category_lower}": {{
///         "{name}": "warn"
///       }}
///     }}
///   }}
/// }}
/// ```
///
/// ## Remediation
///
/// See: <{remediation}>"#,
        title = title,
        description = description,
        requires_supabase_note = requires_supabase_note,
        sql_query_commented = sql_query
            .lines()
            .map(|line| format!("/// {line}"))
            .collect::<Vec<_>>()
            .join("\n"),
        category_lower = category_lower,
        name = name,
        remediation = remediation,
    );

    let content = quote! {
        //! Generated file, do not edit by hand, see `xtask/codegen`

        use crate::rule::SplinterRule;

        ::pgls_analyse::declare_rule! {
            #[doc = #doc_string]
            pub #struct_name {
                version: "1.0.0",
                name: #name,
                severity: #severity,
            }
        }

        impl SplinterRule for #struct_name {
            const SQL_FILE_PATH: &'static str = #sql_path;
            const DESCRIPTION: &'static str = #description;
            const REMEDIATION: &'static str = #remediation;
            const REQUIRES_SUPABASE: bool = #requires_supabase;
        }
    };

    let formatted = xtask::reformat(content)?;
    update(&rule_file, &formatted, &Mode::Overwrite)?;

    Ok(())
}

/// Generate category mod.rs that exports all rules in the category
fn generate_category_mod(
    category_dir: &Path,
    category: &str,
    rules: &[&SqlRuleMetadata],
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
            let struct_name = format_ident!("{}", Case::Pascal.convert(&r.snake_name));
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
    rules_by_category: &BTreeMap<String, Vec<&SqlRuleMetadata>>,
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
            pub Splinter {
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

/// Generate src/registry.rs with visit_registry() and get_sql_file_path()
fn generate_registry(rules: &BTreeMap<String, SqlRuleMetadata>) -> Result<()> {
    let registry_path = project_root().join("crates/pgls_splinter/src/registry.rs");

    // Group rules by category for organized output
    let mut rules_by_category: BTreeMap<String, Vec<&SqlRuleMetadata>> = BTreeMap::new();
    for rule in rules.values() {
        rules_by_category
            .entry(rule.category.to_lowercase())
            .or_default()
            .push(rule);
    }

    // Record the top-level category (which contains all groups)
    let record_calls = vec![quote! {
        registry.record_category::<crate::rules::Splinter>();
    }];

    // Generate match arms for SQL file path mapping (camelCase → path)
    let sql_path_arms: Vec<_> = rules
        .values()
        .map(|rule| {
            let name = &rule.name;
            // Use relative path from crate root
            let relative_path = format!("vendor/{}", rule.sql_file_path.display());

            quote! {
                #name => Some(#relative_path)
            }
        })
        .collect();

    // Generate match arms for category lookup (snake_case → &'static Category)
    let category_arms: Vec<_> = rules
        .values()
        .map(|rule| {
            let snake_name = &rule.snake_name;
            let group = rule.category.to_lowercase();
            let camel_name = &rule.name;
            let category_path = format!("splinter/{group}/{camel_name}");

            quote! {
                #snake_name => Some(::pgls_diagnostics::category!(#category_path))
            }
        })
        .collect();

    // Generate match arms for Supabase requirement check (camelCase → bool)
    let supabase_arms: Vec<_> = rules
        .values()
        .map(|rule| {
            let camel_name = &rule.name;
            let requires = rule.requires_supabase;

            quote! {
                #camel_name => #requires
            }
        })
        .collect();

    // Generate match arms for SQL content lookup (camelCase → embedded SQL)
    // Use include_str! to embed SQL files at compile time
    let sql_content_arms: Vec<_> = rules
        .values()
        .map(|rule| {
            let camel_name = &rule.name;
            // Path relative to crate root (where Cargo.toml is)
            let relative_path = format!("vendor/{}", rule.sql_file_path.display());

            quote! {
                #camel_name => Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #relative_path)))
            }
        })
        .collect();

    // Generate match arms for metadata fields lookup (camelCase → tuple)
    // These call the trait constants from the generated rule types
    let metadata_fields_arms: Vec<_> = rules
        .values()
        .map(|rule| {
            let camel_name = &rule.name;
            let category_ident = format_ident!("{}", rule.category.to_lowercase());
            let module_name = format_ident!("{}", &rule.snake_name);
            let struct_name = format_ident!("{}", Case::Pascal.convert(&rule.snake_name));

            quote! {
                #camel_name => Some((
                    <crate::rules::#category_ident::#module_name::#struct_name as crate::rule::SplinterRule>::DESCRIPTION,
                    <crate::rules::#category_ident::#module_name::#struct_name as crate::rule::SplinterRule>::REMEDIATION,
                    <crate::rules::#category_ident::#module_name::#struct_name as crate::rule::SplinterRule>::REQUIRES_SUPABASE,
                ))
            }
        })
        .collect();

    let content = quote! {
        //! Generated file, do not edit by hand, see `xtask/codegen`

        use pgls_analyse::RegistryVisitor;

        /// Metadata for a splinter rule
        #[derive(Debug, Clone, Copy)]
        pub struct SplinterRuleMetadata {
            /// Description of what the rule detects
            pub description: &'static str,
            /// URL to documentation/remediation guide
            pub remediation: &'static str,
            /// Whether this rule requires Supabase roles (anon, authenticated, service_role)
            pub requires_supabase: bool,
        }

        /// Visit all splinter rules using the visitor pattern
        /// This is called during registry building to collect enabled rules
        pub fn visit_registry<V: RegistryVisitor>(registry: &mut V) {
            #( #record_calls )*
        }

        /// Map rule name (camelCase) to SQL file path
        /// Returns None if rule not found
        #[deprecated(note = "Use get_sql_content() instead - SQL is now embedded at compile time")]
        pub fn get_sql_file_path(rule_name: &str) -> Option<&'static str> {
            match rule_name {
                #( #sql_path_arms, )*
                _ => None,
            }
        }

        /// Get embedded SQL content for a rule (camelCase name)
        /// Returns None if rule not found
        ///
        /// SQL files are embedded at compile time using include_str! for performance
        /// and to make the binary self-contained.
        pub fn get_sql_content(rule_name: &str) -> Option<&'static str> {
            match rule_name {
                #( #sql_content_arms, )*
                _ => None,
            }
        }

        /// Get metadata fields for a rule (camelCase name)
        /// Returns (description, remediation, requires_supabase) tuple
        ///
        /// This calls the trait constants from the generated rule types
        pub fn get_rule_metadata_fields(
            rule_name: &str,
        ) -> Option<(&'static str, &'static str, bool)> {
            match rule_name {
                #( #metadata_fields_arms, )*
                _ => None,
            }
        }

        /// Get metadata for a rule (camelCase name)
        /// Returns None if rule not found
        ///
        /// This provides structured access to rule metadata by calling trait constants
        pub fn get_rule_metadata(rule_name: &str) -> Option<SplinterRuleMetadata> {
            let (description, remediation, requires_supabase) = get_rule_metadata_fields(rule_name)?;
            Some(SplinterRuleMetadata {
                description,
                remediation,
                requires_supabase,
            })
        }

        /// Map rule name from SQL result (snake_case) to diagnostic category
        /// Returns None if rule not found
        ///
        /// This replaces the hardcoded match in convert.rs
        pub fn get_rule_category(rule_name: &str) -> Option<&'static ::pgls_diagnostics::Category> {
            match rule_name {
                #( #category_arms, )*
                _ => None,
            }
        }

        /// Check if a rule requires Supabase roles (anon, authenticated, service_role)
        /// Rules that require Supabase should be filtered out if these roles don't exist
        #[deprecated(note = "Use get_rule_metadata() instead")]
        pub fn rule_requires_supabase(rule_name: &str) -> bool {
            match rule_name {
                #( #supabase_arms, )*
                _ => false,
            }
        }
    };

    let formatted = xtask::reformat(content)?;
    update(&registry_path, &formatted, &Mode::Overwrite)?;

    Ok(())
}

/// Update the categories.rs file with splinter rules
/// This maintains backward compatibility with existing category system
fn update_categories_file(rules: &BTreeMap<String, SqlRuleMetadata>) -> Result<()> {
    let categories_path =
        project_root().join("crates/pgls_diagnostics_categories/src/categories.rs");

    let mut content = fs2::read_to_string(&categories_path)?;

    // Generate splinter rule entries grouped by category
    let mut splinter_rules: Vec<(String, String)> = rules
        .values()
        .map(|rule| {
            let group = rule.category.to_lowercase();
            let url = &rule.remediation;

            (
                group.clone(),
                format!("    \"splinter/{}/{}\": \"{}\",", group, rule.name, url),
            )
        })
        .collect();

    // Sort by group, then by entry
    splinter_rules.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    // Extract just the formatted strings
    let splinter_entries: String = splinter_rules
        .iter()
        .map(|(_, entry)| entry.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    // Replace content between splinter rules markers
    let rules_start = "// splinter rules start";
    let rules_end = "// splinter rules end";

    content = replace_between_markers(
        &content,
        rules_start,
        rules_end,
        &format!("\n{splinter_entries}\n    "),
    )?;

    // Generate splinter group entries
    let mut groups: Vec<String> = splinter_rules
        .iter()
        .map(|(group, _)| group.clone())
        .collect();
    groups.sort();
    groups.dedup();

    let mut group_entries = vec!["    \"splinter\",".to_string()];
    for group in groups {
        group_entries.push(format!("    \"splinter/{group}\","));
    }
    let groups_content = group_entries.join("\n");

    // Replace content between splinter groups markers
    let groups_start = "// Splinter groups start";
    let groups_end = "// Splinter groups end";

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
