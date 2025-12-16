//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use pgls_analyse::RuleMeta;
#[doc = r" Trait for splinter (database-level) rules"]
#[doc = r""]
#[doc = r" Splinter rules are different from linter rules:"]
#[doc = r" - They execute SQL queries against the database"]
#[doc = r" - They don't have AST-based execution"]
#[doc = r" - Rule logic is in SQL files, not Rust"]
pub trait SplinterRule: RuleMeta {
    #[doc = r" Path to the SQL file containing the rule query"]
    const SQL_FILE_PATH: &'static str;
    #[doc = r" Description of what the rule detects"]
    const DESCRIPTION: &'static str;
    #[doc = r" URL to documentation/remediation guide"]
    const REMEDIATION: &'static str;
    #[doc = r" Whether this rule requires Supabase roles (anon, authenticated, service_role)"]
    const REQUIRES_SUPABASE: bool;
}
