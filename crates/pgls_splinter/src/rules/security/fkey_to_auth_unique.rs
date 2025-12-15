//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub FkeyToAuthUnique { version : "1.0.0" , name : "fkeyToAuthUnique" , severity : pgls_diagnostics :: Severity :: Error , } }
impl SplinterRule for FkeyToAuthUnique {
    fn sql_file_path() -> &'static str {
        "security/fkey_to_auth_unique.sql"
    }
}
