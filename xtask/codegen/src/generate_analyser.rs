use std::path::PathBuf;
use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Ok, Result};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use xtask::{glue::fs2, project_root};

pub fn generate_analyser() -> Result<()> {
    generate_linter()?;
    Ok(())
}

fn generate_linter() -> Result<()> {
    let base_path = project_root().join("crates/pgls_analyser/src");
    let mut analysers = BTreeMap::new();
    let mut all_rules = BTreeMap::new();
    generate_category("lint", &mut analysers, &mut all_rules, &base_path)?;

    generate_options(&base_path)?;

    update_linter_registry_builder(analysers, all_rules)
}

fn generate_options(base_path: &Path) -> Result<()> {
    let mut rules_options = BTreeMap::new();
    let mut crates = vec![];
    for category in ["lint"] {
        let category_path = base_path.join(category);
        if !category_path.exists() {
            continue;
        }
        let category_name = format_ident!("{}", filename(&category_path)?);
        for group_path in list_entry_paths(&category_path)?.filter(|path| path.is_dir()) {
            let group_name = format_ident!("{}", filename(&group_path)?.to_string());
            for rule_path in list_entry_paths(&group_path)?.filter(|path| !path.is_dir()) {
                let rule_filename = filename(&rule_path)?;
                let rule_name = rule_filename.to_case(Case::Pascal);
                let rule_module_name = format_ident!("{}", rule_filename);
                let rule_name = format_ident!("{}", rule_name);
                rules_options.insert(rule_filename.to_string(), quote! {
                    pub type #rule_name = <#category_name::#group_name::#rule_module_name::#rule_name as crate::LinterRule>::Options;
                });
            }
        }
        if category == "lint" {
            crates.push(quote! {
                use crate::lint;
            })
        }
    }
    let rules_options = rules_options.values();
    let tokens = xtask::reformat(quote! {
        #( #crates )*

        #( #rules_options )*
    })?;
    fs2::write(base_path.join("options.rs"), tokens)?;

    Ok(())
}

fn generate_category(
    name: &'static str,
    entries: &mut BTreeMap<&'static str, TokenStream>,
    all_rules: &mut BTreeMap<String, (TokenStream, String)>,
    base_path: &Path,
) -> Result<()> {
    let path = base_path.join(name);

    let mut groups = BTreeMap::new();
    for entry in fs2::read_dir(path)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let entry = entry.path();
        let file_name = entry
            .file_stem()
            .context("path has no file name")?
            .to_str()
            .context("could not convert file name to string")?;

        generate_group(name, file_name, all_rules, base_path)?;

        let module_name = format_ident!("{}", file_name);
        let group_name = format_ident!("{}", file_name.to_case(Case::Pascal));

        groups.insert(
            file_name.to_string(),
            (
                quote! {
                   pub mod #module_name;
                },
                quote! {
                    self::#module_name::#group_name
                },
            ),
        );
    }

    let key = name;
    let module_name = format_ident!("{name}");

    let category_name = name.to_case(Case::Pascal);
    let category_name = format_ident!("{category_name}");

    let kind = match name {
        "lint" => format_ident!("Lint"),
        _ => panic!("unimplemented analyser category {name:?}"),
    };

    entries.insert(
        key,
        quote! {
            registry.record_category::<crate::#module_name::#category_name>();
        },
    );

    let (modules, paths): (Vec<_>, Vec<_>) = groups.into_values().unzip();
    let tokens = xtask::reformat(quote! {
        #( #modules )*
        ::pgls_analyse::declare_category! {
            pub #category_name {
                kind: #kind,
                groups: [
                    #( #paths, )*
                ]
            }
        }
    })?;

    fs2::write(base_path.join(format!("{name}.rs")), tokens)?;

    Ok(())
}

fn generate_group(
    category: &'static str,
    group: &str,
    all_rules: &mut BTreeMap<String, (TokenStream, String)>,
    base_path: &Path,
) -> Result<()> {
    let path = base_path.join(category).join(group);

    let mut rules = BTreeMap::new();
    for entry in fs2::read_dir(path)? {
        let entry = entry?.path();
        let file_name = entry
            .file_stem()
            .context("path has no file name")?
            .to_str()
            .context("could not convert file name to string")?;

        let rule_type = file_name.to_case(Case::Pascal);

        let key = rule_type.clone();
        let module_name = format_ident!("{}", file_name);
        let rule_type_ident = format_ident!("{}", rule_type);

        // Collect for factory generation
        // Key is the rule name (camelCase for config), value is (full_path_tokens, metadata_name)
        let category_ident = format_ident!("{}", category);
        let group_module_ident = format_ident!("{}", group); // Module name (lowercase)
        all_rules.insert(
            key.to_case(Case::Camel),
            (
                quote! {
                    crate::#category_ident::#group_module_ident::#module_name::#rule_type_ident
                },
                file_name.to_string(),
            ),
        );

        rules.insert(
            key,
            (
                quote! {
                    pub mod #module_name;
                },
                quote! {
                    self::#module_name::#rule_type_ident
                },
            ),
        );
    }

    let group_name = format_ident!("{}", group.to_case(Case::Pascal));

    let (rule_imports, rule_names): (Vec<_>, Vec<_>) = rules.into_values().unzip();

    let (import_macro, use_macro) = match category {
        "lint" => (
            quote!(
                use pgls_analyse::declare_lint_group;
            ),
            quote!(declare_lint_group),
        ),
        _ => panic!("Category not supported: {category}"),
    };
    let tokens = xtask::reformat(quote! {
        #import_macro

        #(#rule_imports)*

        #use_macro! {
            pub #group_name {
                name: #group,
                rules: [
                    #(#rule_names,)*
                ]
            }
        }
    })?;

    fs2::write(base_path.join(category).join(format!("{group}.rs")), tokens)?;

    Ok(())
}

fn update_linter_registry_builder(
    rules: BTreeMap<&'static str, TokenStream>,
    all_rules: BTreeMap<String, (TokenStream, String)>,
) -> Result<()> {
    let path = project_root().join("crates/pgls_analyser/src/registry.rs");

    let categories = rules.into_values();

    // Generate match arms that directly create executors (no closure/Box overhead)
    let executor_arms = all_rules.iter().map(|(rule_name, (rule_path, _))| {
        quote! {
            #rule_name => Some(crate::linter_registry::RegistryLinterRule::new::<#rule_path>())
        }
    });

    let tokens = xtask::reformat(quote! {
        use pgls_analyse::{RegistryVisitor, RuleKey};
        use crate::linter_registry::RegistryLinterRule;

        pub fn visit_registry<V: RegistryVisitor>(registry: &mut V) {
            #( #categories )*
        }

        /// Maps rule keys to rule executors (zero-cost abstraction)
        /// This function is generated by codegen and includes all linter rules
        pub fn get_linter_rule_executor(
            key: &RuleKey,
        ) -> Option<RegistryLinterRule> {
            match key.rule_name() {
                #( #executor_arms, )*
                _ => None,
            }
        }
    })?;

    fs2::write(path, tokens)?;

    Ok(())
}

/// Returns file paths of the given directory.
fn list_entry_paths(dir: &Path) -> Result<impl Iterator<Item = PathBuf> + use<>> {
    Ok(fs2::read_dir(dir)
        .context("A directory is expected")?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path()))
}

/// Returns filename if any.
fn filename(file: &Path) -> Result<&str> {
    file.file_stem()
        .context("path has no file name")?
        .to_str()
        .context("could not convert file name to string")
}
