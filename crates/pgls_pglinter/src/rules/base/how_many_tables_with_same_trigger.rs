//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# HowManyTablesWithSameTrigger (B009)\n\nCount number of tables using the same trigger vs nb table with their own triggers.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"base\": {\n        \"howManyTablesWithSameTrigger\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 20%\n- Error level: 80%\n\n## Fixes\n\n- For more readability and other considerations use one trigger function per table.\n- Sharing the same trigger function add more complexity.\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#b009>"] pub HowManyTablesWithSameTrigger { version : "1.0.0" , name : "howManyTablesWithSameTrigger" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for HowManyTablesWithSameTrigger {
    const CODE: &'static str = "B009";
    const SCOPE: &'static str = "BASE";
    const DESCRIPTION: &'static str =
        "Count number of tables using the same trigger vs nb table with their own triggers.";
    const FIXES: &'static [&'static str] = &[
        "For more readability and other considerations use one trigger function per table.",
        "Sharing the same trigger function add more complexity.",
    ];
}
