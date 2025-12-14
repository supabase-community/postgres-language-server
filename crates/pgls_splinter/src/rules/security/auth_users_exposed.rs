//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
use pgls_analyse::RuleMeta;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub AuthUsersExposed { version : "1.0.0" , name : "authUsersExposed" , severity : pgls_diagnostics :: Severity :: Error , } }
impl SplinterRule for AuthUsersExposed {
    fn sql_file_path() -> &'static str {
        "security/auth_users_exposed.sql"
    }
}
