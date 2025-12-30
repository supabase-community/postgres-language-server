//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# CompositePrimaryKeyTooManyColumns (B012)\n\nDetect tables with composite primary keys involving more than 4 columns\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"base\": {\n        \"compositePrimaryKeyTooManyColumns\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 1%\n- Error level: 80%\n\n## Fixes\n\n- Consider redesigning the table to avoid composite primary keys with more than 4 columns\n- Use surrogate keys (e.g., serial, UUID) instead of composite primary keys, and establish unique constraints on necessary column combinations, to enforce uniqueness.\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#b012>"] pub CompositePrimaryKeyTooManyColumns { version : "1.0.0" , name : "compositePrimaryKeyTooManyColumns" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for CompositePrimaryKeyTooManyColumns {
    const CODE: &'static str = "B012";
    const SCOPE: &'static str = "BASE";
    const DESCRIPTION: &'static str =
        "Detect tables with composite primary keys involving more than 4 columns";
    const FIXES: &'static [&'static str] = &[
        "Consider redesigning the table to avoid composite primary keys with more than 4 columns",
        "Use surrogate keys (e.g., serial, UUID) instead of composite primary keys, and establish unique constraints on necessary column combinations, to enforce uniqueness.",
    ];
}
