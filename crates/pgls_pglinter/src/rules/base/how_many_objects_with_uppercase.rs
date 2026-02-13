//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# HowManyObjectsWithUppercase (B005)\n\nCount number of objects with uppercase in name or in columns.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"base\": {\n        \"howManyObjectsWithUppercase\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 20%\n- Error level: 80%\n\n## Fixes\n\n- Do not use uppercase for any database objects\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#b005>"] pub HowManyObjectsWithUppercase { version : "1.0.0" , name : "howManyObjectsWithUppercase" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for HowManyObjectsWithUppercase {
    const CODE: &'static str = "B005";
    const SCOPE: &'static str = "BASE";
    const DESCRIPTION: &'static str =
        "Count number of objects with uppercase in name or in columns.";
    const FIXES: &'static [&'static str] = &["Do not use uppercase for any database objects"];
}
