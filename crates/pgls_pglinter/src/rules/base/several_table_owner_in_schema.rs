//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# SeveralTableOwnerInSchema (B011)\n\nIn a schema there are several tables owned by different owners.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"base\": {\n        \"severalTableOwnerInSchema\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 1%\n- Error level: 80%\n\n## Fixes\n\n- change table owners to the same functional role\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#b011>"] pub SeveralTableOwnerInSchema { version : "1.0.0" , name : "severalTableOwnerInSchema" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for SeveralTableOwnerInSchema {
    const CODE: &'static str = "B011";
    const SCOPE: &'static str = "BASE";
    const DESCRIPTION: &'static str =
        "In a schema there are several tables owned by different owners.";
    const FIXES: &'static [&'static str] = &["change table owners to the same functional role"];
}
