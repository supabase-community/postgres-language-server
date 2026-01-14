//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# UnsecuredPublicSchema (S003)\n\nOnly authorized users should be allowed to create objects.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"schema\": {\n        \"unsecuredPublicSchema\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 1%\n- Error level: 80%\n\n## Fixes\n\n- REVOKE CREATE ON SCHEMA <schema_name> FROM PUBLIC\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#s003>"] pub UnsecuredPublicSchema { version : "1.0.0" , name : "unsecuredPublicSchema" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for UnsecuredPublicSchema {
    const CODE: &'static str = "S003";
    const SCOPE: &'static str = "SCHEMA";
    const DESCRIPTION: &'static str = "Only authorized users should be allowed to create objects.";
    const FIXES: &'static [&'static str] = &["REVOKE CREATE ON SCHEMA <schema_name> FROM PUBLIC"];
}
