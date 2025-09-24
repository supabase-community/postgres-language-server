//! Codegen tools. Derived from Biome's codegen

mod generate_analyser;
mod generate_bindings;
mod generate_configuration;
mod generate_crate;
mod generate_new_analyser_rule;

pub use self::generate_analyser::generate_analyser;
pub use self::generate_bindings::generate_bindings;
pub use self::generate_configuration::generate_rules_configuration;
pub use self::generate_crate::generate_crate;
pub use self::generate_new_analyser_rule::generate_new_analyser_rule;
use bpaf::Bpaf;
use generate_new_analyser_rule::Category;
use pgls_diagnostics::Severity;
use std::path::Path;
use xtask::{glue::fs2, Mode, Result};

pub enum UpdateResult {
    NotUpdated,
    Updated,
}

/// A helper to update file on disk if it has changed.
/// With verify = false, the contents of the file will be updated to the passed in contents.
/// With verify = true, an Err will be returned if the contents of the file do not match the passed-in contents.
pub fn update(path: &Path, contents: &str, mode: &Mode) -> Result<UpdateResult> {
    if fs2::read_to_string(path).is_ok_and(|old_contents| old_contents == contents) {
        return Ok(UpdateResult::NotUpdated);
    }

    if *mode == Mode::Verify {
        anyhow::bail!("`{}` is not up-to-date", path.display());
    }

    eprintln!("updating {}", path.display());
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs2::create_dir_all(parent)?;
        }
    }
    fs2::write(path, contents)?;
    Ok(UpdateResult::Updated)
}

pub fn to_capitalized(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum TaskCommand {
    /// Generate TypeScript definitions for the JavaScript bindings to the Workspace API
    #[bpaf(command)]
    Bindings,
    /// Generate factory functions for the analyser and the configuration of the analysers
    #[bpaf(command)]
    Analyser,
    /// Generate the part of the configuration that depends on some metadata
    #[bpaf(command)]
    Configuration,
    /// Creates a new crate
    #[bpaf(command, long("new-crate"))]
    NewCrate {
        /// The name of the crate
        #[bpaf(long("name"), argument("STRING"))]
        name: String,
    },
    /// Creates a new lint rule
    #[bpaf(command, long("new-lintrule"))]
    NewRule {
        /// Name of the rule
        #[bpaf(long("name"))]
        name: String,

        /// Category of the rule
        #[bpaf(long("category"))]
        category: Category,

        /// Group of the rule
        #[bpaf(long("group"))]
        group: String,

        /// Severity of the rule
        #[bpaf(long("severity"), fallback(Severity::Error))]
        severity: Severity,
    },
}
