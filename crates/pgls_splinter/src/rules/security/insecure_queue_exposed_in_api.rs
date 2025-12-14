//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::SplinterRule;
use pgls_analyse::RuleMeta;
::pgls_analyse::declare_rule! { # [doc = r" #title"] # [doc = r""] # [doc = r" #description"] pub InsecureQueueExposedInApi { version : "1.0.0" , name : "insecureQueueExposedInApi" , severity : pgls_diagnostics :: Severity :: Error , } }
impl SplinterRule for InsecureQueueExposedInApi {
    fn sql_file_path() -> &'static str {
        "security/insecure_queue_exposed_in_api.sql"
    }
}
