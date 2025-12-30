//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use pgls_analyse::RuleMeta;
#[doc = r" Trait for pglinter (database-level) rules"]
#[doc = r""]
#[doc = r" Pglinter rules are different from linter rules:"]
#[doc = r" - They execute SQL queries against the database via pglinter extension"]
#[doc = r" - They don't have AST-based execution"]
#[doc = r" - Rule logic is in the pglinter Postgres extension"]
#[doc = r" - Threshold configuration (warning/error levels) is handled by pglinter extension"]
pub trait PglinterRule: RuleMeta {
    #[doc = r#" Rule code (e.g., "B001", "S001", "C001")"#]
    const CODE: &'static str;
    #[doc = r" Rule scope (BASE, SCHEMA, or CLUSTER)"]
    const SCOPE: &'static str;
    #[doc = r" Description of what the rule detects"]
    const DESCRIPTION: &'static str;
    #[doc = r" Suggested fixes for violations"]
    const FIXES: &'static [&'static str];
}
