//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# HowManyTablesNeverSelected (B006)\n\nCount number of table(s) that has never been selected.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"base\": {\n        \"howManyTablesNeverSelected\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 1%\n- Error level: 80%\n\n## Fixes\n\n- Is it necessary to update/delete/insert rows in table(s) that are never selected ?\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#b006>"] pub HowManyTablesNeverSelected { version : "1.0.0" , name : "howManyTablesNeverSelected" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for HowManyTablesNeverSelected {
    const CODE: &'static str = "B006";
    const SCOPE: &'static str = "BASE";
    const DESCRIPTION: &'static str = "Count number of table(s) that has never been selected.";
    const FIXES: &'static [&'static str] =
        &["Is it necessary to update/delete/insert rows in table(s) that are never selected ?"];
}
