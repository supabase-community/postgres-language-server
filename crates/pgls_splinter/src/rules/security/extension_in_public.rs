//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
use pgls_analyse::RuleMeta;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub ExtensionInPublic { version : "1.0.0" , name : "extensionInPublic" , severity : pgls_diagnostics :: Severity :: Warning , } }
impl SplinterRule for ExtensionInPublic {
    fn sql_file_path() -> &'static str {
        "security/extension_in_public.sql"
    }
}
