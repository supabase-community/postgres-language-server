//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
use pgls_analyse::RuleMeta;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub RlsEnabledNoPolicy { version : "1.0.0" , name : "rlsEnabledNoPolicy" , severity : pgls_diagnostics :: Severity :: Information , } }
impl SplinterRule for RlsEnabledNoPolicy {
    fn sql_file_path() -> &'static str {
        "security/rls_enabled_no_policy.sql"
    }
}
