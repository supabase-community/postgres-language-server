//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
use pgls_analyse::RuleMeta;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub FunctionSearchPathMutable { version : "1.0.0" , name : "functionSearchPathMutable" , severity : pgls_diagnostics :: Severity :: Warning , } }
impl SplinterRule for FunctionSearchPathMutable {
    fn sql_file_path() -> &'static str {
        "security/function_search_path_mutable.sql"
    }
}
