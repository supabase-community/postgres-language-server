//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# HowManyTablesWithFkMismatch (B008)\n\nCount number of tables with foreign keys that do not match the key reference type.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"base\": {\n        \"howManyTablesWithFkMismatch\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 1%\n- Error level: 80%\n\n## Fixes\n\n- Consider column type adjustments to ensure foreign key matches referenced key type\n- ask a dba\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#b008>"] pub HowManyTablesWithFkMismatch { version : "1.0.0" , name : "howManyTablesWithFkMismatch" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for HowManyTablesWithFkMismatch {
    const CODE: &'static str = "B008";
    const SCOPE: &'static str = "BASE";
    const DESCRIPTION: &'static str =
        "Count number of tables with foreign keys that do not match the key reference type.";
    const FIXES: &'static [&'static str] = &[
        "Consider column type adjustments to ensure foreign key matches referenced key type",
        "ask a dba",
    ];
}
