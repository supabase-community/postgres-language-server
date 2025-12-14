//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
use pgls_analyse::RuleMeta;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub NoPrimaryKey { version : "1.0.0" , name : "noPrimaryKey" , severity : pgls_diagnostics :: Severity :: Information , } }
impl SplinterRule for NoPrimaryKey {
    fn sql_file_path() -> &'static str {
        "performance/no_primary_key.sql"
    }
}
