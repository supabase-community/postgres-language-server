//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
use pgls_analyse::RuleMeta;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub UnusedIndex { version : "1.0.0" , name : "unusedIndex" , severity : pgls_diagnostics :: Severity :: Information , } }
impl SplinterRule for UnusedIndex {
    fn sql_file_path() -> &'static str {
        "performance/unused_index.sql"
    }
}
