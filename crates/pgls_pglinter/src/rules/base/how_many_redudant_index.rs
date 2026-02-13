//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# HowManyRedudantIndex (B002)\n\nCount number of redundant index vs nb index.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"base\": {\n        \"howManyRedudantIndex\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 1%\n- Error level: 80%\n\n## Fixes\n\n- remove duplicated index or check if a constraint does not create a redundant index, or change warning/error threshold\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#b002>"] pub HowManyRedudantIndex { version : "1.0.0" , name : "howManyRedudantIndex" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for HowManyRedudantIndex {
    const CODE: &'static str = "B002";
    const SCOPE: &'static str = "BASE";
    const DESCRIPTION: &'static str = "Count number of redundant index vs nb index.";
    const FIXES: &'static [&'static str] = &[
        "remove duplicated index or check if a constraint does not create a redundant index, or change warning/error threshold",
    ];
}
