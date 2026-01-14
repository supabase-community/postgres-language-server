//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# HowManyUnusedIndex (B004)\n\nCount number of unused index vs nb index (base on pg_stat_user_indexes, indexes associated to unique constraints are discard.)\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"base\": {\n        \"howManyUnusedIndex\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 20%\n- Error level: 80%\n\n## Fixes\n\n- remove unused index or change warning/error threshold\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#b004>"] pub HowManyUnusedIndex { version : "1.0.0" , name : "howManyUnusedIndex" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for HowManyUnusedIndex {
    const CODE: &'static str = "B004";
    const SCOPE: &'static str = "BASE";
    const DESCRIPTION: &'static str = "Count number of unused index vs nb index (base on pg_stat_user_indexes, indexes associated to unique constraints are discard.)";
    const FIXES: &'static [&'static str] =
        &["remove unused index or change warning/error threshold"];
}
