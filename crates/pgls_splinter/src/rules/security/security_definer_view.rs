//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
use pgls_analyse::RuleMeta;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub SecurityDefinerView { version : "1.0.0" , name : "securityDefinerView" , severity : pgls_diagnostics :: Severity :: Error , } }
impl SplinterRule for SecurityDefinerView {
    fn sql_file_path() -> &'static str {
        "security/security_definer_view.sql"
    }
}
