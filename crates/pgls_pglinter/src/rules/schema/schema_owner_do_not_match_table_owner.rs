//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# SchemaOwnerDoNotMatchTableOwner (S005)\n\nThe schema owner and tables in the schema do not match.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"schema\": {\n        \"schemaOwnerDoNotMatchTableOwner\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 20%\n- Error level: 80%\n\n## Fixes\n\n- For maintenance facilities, schema and tables owners should be the same.\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#s005>"] pub SchemaOwnerDoNotMatchTableOwner { version : "1.0.0" , name : "schemaOwnerDoNotMatchTableOwner" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for SchemaOwnerDoNotMatchTableOwner {
    const CODE: &'static str = "S005";
    const SCOPE: &'static str = "SCHEMA";
    const DESCRIPTION: &'static str = "The schema owner and tables in the schema do not match.";
    const FIXES: &'static [&'static str] =
        &["For maintenance facilities, schema and tables owners should be the same."];
}
