//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# OwnerSchemaIsInternalRole (S004)\n\nOwner of schema should not be any internal pg roles, or owner is a superuser (not sure it is necesary).\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"schema\": {\n        \"ownerSchemaIsInternalRole\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 20%\n- Error level: 80%\n\n## Fixes\n\n- change schema owner to a functional role\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#s004>"] pub OwnerSchemaIsInternalRole { version : "1.0.0" , name : "ownerSchemaIsInternalRole" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for OwnerSchemaIsInternalRole {
    const CODE: &'static str = "S004";
    const SCOPE: &'static str = "SCHEMA";
    const DESCRIPTION: &'static str = "Owner of schema should not be any internal pg roles, or owner is a superuser (not sure it is necesary).";
    const FIXES: &'static [&'static str] = &["change schema owner to a functional role"];
}
