//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# HowManyTablesWithReservedKeywords (B010)\n\nCount number of database objects using reserved keywords in their names.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"base\": {\n        \"howManyTablesWithReservedKeywords\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 20%\n- Error level: 80%\n\n## Fixes\n\n- Rename database objects to avoid using reserved keywords.\n- Using reserved keywords can lead to SQL syntax errors and maintenance difficulties.\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#b010>"] pub HowManyTablesWithReservedKeywords { version : "1.0.0" , name : "howManyTablesWithReservedKeywords" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for HowManyTablesWithReservedKeywords {
    const CODE: &'static str = "B010";
    const SCOPE: &'static str = "BASE";
    const DESCRIPTION: &'static str =
        "Count number of database objects using reserved keywords in their names.";
    const FIXES: &'static [&'static str] = &[
        "Rename database objects to avoid using reserved keywords.",
        "Using reserved keywords can lead to SQL syntax errors and maintenance difficulties.",
    ];
}
