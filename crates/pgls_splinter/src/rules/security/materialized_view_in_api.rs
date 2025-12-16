//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub MaterializedViewInApi { version : "1.0.0" , name : "materializedViewInApi" , severity : pgls_diagnostics :: Severity :: Warning , } }
impl SplinterRule for MaterializedViewInApi {
    fn sql_file_path() -> &'static str {
        "security/materialized_view_in_api.sql"
    }
}
