//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# HowManyTableWithoutPrimaryKey (B001)\n\nCount number of tables without primary key.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"base\": {\n        \"howManyTableWithoutPrimaryKey\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 1%\n- Error level: 80%\n\n## Fixes\n\n- create a primary key or change warning/error threshold\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#b001>"] pub HowManyTableWithoutPrimaryKey { version : "1.0.0" , name : "howManyTableWithoutPrimaryKey" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for HowManyTableWithoutPrimaryKey {
    const CODE: &'static str = "B001";
    const SCOPE: &'static str = "BASE";
    const DESCRIPTION: &'static str = "Count number of tables without primary key.";
    const FIXES: &'static [&'static str] =
        &["create a primary key or change warning/error threshold"];
}
