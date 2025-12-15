//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub RlsDisabledInPublic { version : "1.0.0" , name : "rlsDisabledInPublic" , severity : pgls_diagnostics :: Severity :: Error , } }
impl SplinterRule for RlsDisabledInPublic {
    fn sql_file_path() -> &'static str {
        "security/rls_disabled_in_public.sql"
    }
}
