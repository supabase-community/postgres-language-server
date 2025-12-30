//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# HowManyTablesWithFkOutsideSchema (B007)\n\nCount number of tables with foreign keys outside their schema.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"base\": {\n        \"howManyTablesWithFkOutsideSchema\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 20%\n- Error level: 80%\n\n## Fixes\n\n- Consider restructuring schema design to keep related tables in same schema\n- ask a dba\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#b007>"] pub HowManyTablesWithFkOutsideSchema { version : "1.0.0" , name : "howManyTablesWithFkOutsideSchema" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for HowManyTablesWithFkOutsideSchema {
    const CODE: &'static str = "B007";
    const SCOPE: &'static str = "BASE";
    const DESCRIPTION: &'static str =
        "Count number of tables with foreign keys outside their schema.";
    const FIXES: &'static [&'static str] = &[
        "Consider restructuring schema design to keep related tables in same schema",
        "ask a dba",
    ];
}
