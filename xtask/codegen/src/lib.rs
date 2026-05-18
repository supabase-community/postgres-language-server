//! Codegen tools. Derived from Biome's codegen

use convert_case::{Boundary, Case, Converter};

mod generate_analyser;
mod generate_bindings;
mod generate_configuration;
mod generate_crate;
mod generate_new_analyser_rule;
mod generate_pglinter;
mod generate_schema_types;
mod generate_splinter;

pub mod glue;

use std::{
    env,
    fmt::Display,
    path::{Path, PathBuf},
};

pub use crate::glue::{pushd, pushenv};
pub use anyhow::{anyhow, bail, ensure, Context as _, Error, Result};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Overwrite,
    Verify,
}

pub fn project_root() -> PathBuf {
    Path::new(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(2)
    .unwrap()
    .to_path_buf()
}

pub fn run_rustfmt(mode: Mode) -> Result<()> {
    let _dir = pushd(project_root());
    // Only set RUSTUP_TOOLCHAIN if nightly isn't already on PATH (e.g., in Nix)
    let _e = if !is_nightly_rustfmt_available() {
        Some(pushenv("RUSTUP_TOOLCHAIN", "nightly"))
    } else {
        None
    };
    ensure_rustfmt()?;
    match mode {
        Mode::Overwrite => crate::run!("cargo fmt"),
        Mode::Verify => crate::run!("cargo fmt -- --check"),
    }?;
    Ok(())
}

pub fn reformat(text: impl Display) -> Result<String> {
    reformat_without_preamble(text).map(prepend_generated_preamble)
}

pub fn reformat_with_command(text: impl Display, command: impl Display) -> Result<String> {
    reformat_without_preamble(text).map(|formatted| {
        format!("//! This is a generated file. Don't modify it by hand! Run '{command}' to re-generate the file.\n\n{formatted}")
    })
}

pub const PREAMBLE: &str = "Generated file, do not edit by hand, see `xtask/codegen`";
pub fn prepend_generated_preamble(content: impl Display) -> String {
    format!("//! {PREAMBLE}\n\n{content}")
}

pub fn reformat_without_preamble(text: impl Display) -> Result<String> {
    // Only set RUSTUP_TOOLCHAIN if nightly isn't already on PATH (e.g., in Nix)
    let _e = if !is_nightly_rustfmt_available() {
        Some(pushenv("RUSTUP_TOOLCHAIN", "nightly"))
    } else {
        None
    };
    ensure_rustfmt()?;
    let output = crate::run!(
        "rustfmt --config newline_style=Unix";
        <text.to_string().as_bytes()
    )?;

    Ok(format!("{output}\n"))
}

/// Check if nightly rustfmt is already available on PATH (e.g., provided by Nix)
fn is_nightly_rustfmt_available() -> bool {
    crate::run!("rustfmt --version")
        .map(|out| out.contains("nightly"))
        .unwrap_or(false)
}

pub fn ensure_rustfmt() -> Result<()> {
    let out = crate::run!("rustfmt --version")?;
    if !out.contains("nightly") {
        bail!(
            "Failed to run rustfmt from toolchain 'nightly'. \
             Please run `rustup component add rustfmt --toolchain nightly` to install it.",
        )
    }
    Ok(())
}

pub use self::generate_analyser::generate_analyser;
pub use self::generate_bindings::generate_bindings;
pub use self::generate_configuration::{generate_rules_configuration, generate_tool_configuration};
pub use self::generate_crate::generate_crate;
pub use self::generate_new_analyser_rule::generate_new_analyser_rule;
pub use self::generate_pglinter::generate_pglinter;
pub use self::generate_schema_types::generate_schema_types;
pub use self::generate_splinter::generate_splinter;
use crate::glue::fs2;
use bpaf::Bpaf;
use generate_new_analyser_rule::Category;
use pgls_diagnostics::Severity;

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

/// Convert to snake_case without splitting on digit boundaries.
///
/// `convert_case` treats digit-letter boundaries as word separators
/// (e.g. "Md5" → "md_5"), but we want digits attached to the preceding
/// word (e.g. "Md5" → "md5") to match the old biome_string_case behavior.
pub fn to_snake_case(s: &str) -> String {
    Converter::new()
        .to_case(Case::Snake)
        .remove_boundary(Boundary::DigitUpper)
        .remove_boundary(Boundary::DigitLower)
        .remove_boundary(Boundary::UpperDigit)
        .remove_boundary(Boundary::LowerDigit)
        .convert(s)
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
    /// Generate splinter categories from the SQL file
    #[bpaf(command)]
    Splinter,
    /// Generate pglinter rules from pglinter_repo/sql/rules.sql
    #[bpaf(command)]
    Pglinter,
    /// Generate schema cache TypeScript types
    #[bpaf(command)]
    SchemaTypes,
}
