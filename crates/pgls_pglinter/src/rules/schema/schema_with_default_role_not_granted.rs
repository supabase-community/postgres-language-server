//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# SchemaWithDefaultRoleNotGranted (S001)\n\nThe schema has no default role. Means that futur table will not be granted through a role. So you will have to re-execute grants on it.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"schema\": {\n        \"schemaWithDefaultRoleNotGranted\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 1%\n- Error level: 1%\n\n## Fixes\n\n- add a default privilege=> ALTER DEFAULT PRIVILEGES IN SCHEMA <schema> for user <schema\n- s owner>\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#s001>"] pub SchemaWithDefaultRoleNotGranted { version : "1.0.0" , name : "schemaWithDefaultRoleNotGranted" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for SchemaWithDefaultRoleNotGranted {
    const CODE: &'static str = "S001";
    const SCOPE: &'static str = "SCHEMA";
    const DESCRIPTION: &'static str = "The schema has no default role. Means that futur table will not be granted through a role. So you will have to re-execute grants on it.";
    const FIXES: &'static [&'static str] = &[
        "add a default privilege=> ALTER DEFAULT PRIVILEGES IN SCHEMA <schema> for user <schema",
        "s owner>",
    ];
}
