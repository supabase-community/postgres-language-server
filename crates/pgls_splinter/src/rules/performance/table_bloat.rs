//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
use pgls_analyse::RuleMeta;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub TableBloat { version : "1.0.0" , name : "tableBloat" , severity : pgls_diagnostics :: Severity :: Information , } }
impl SplinterRule for TableBloat {
    fn sql_file_path() -> &'static str {
        "performance/table_bloat.sql"
    }
}
