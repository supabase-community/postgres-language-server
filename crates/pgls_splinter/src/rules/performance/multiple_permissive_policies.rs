//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub MultiplePermissivePolicies { version : "1.0.0" , name : "multiplePermissivePolicies" , severity : pgls_diagnostics :: Severity :: Warning , } }
impl SplinterRule for MultiplePermissivePolicies {
    fn sql_file_path() -> &'static str {
        "performance/multiple_permissive_policies.sql"
    }
}
