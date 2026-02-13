//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# PasswordEncryptionIsMd5 (C003)\n\nThis configuration is not secure anymore and will prevent an upgrade to Postgres 18. Warning, you will need to reset all passwords after this is changed to scram-sha-256.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"cluster\": {\n        \"passwordEncryptionIsMd5\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 20%\n- Error level: 80%\n\n## Fixes\n\n- change password_encryption parameter to scram-sha-256 (ALTER SYSTEM SET password_encryption = \n- scram-sha-256\n-  ). Warning, you will need to reset all passwords after this parameter is updated.\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#c003>"] pub PasswordEncryptionIsMd5 { version : "1.0.0" , name : "passwordEncryptionIsMd5" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for PasswordEncryptionIsMd5 {
    const CODE: &'static str = "C003";
    const SCOPE: &'static str = "CLUSTER";
    const DESCRIPTION: &'static str = "This configuration is not secure anymore and will prevent an upgrade to Postgres 18. Warning, you will need to reset all passwords after this is changed to scram-sha-256.";
    const FIXES: &'static [&'static str] = &[
        "change password_encryption parameter to scram-sha-256 (ALTER SYSTEM SET password_encryption = ",
        "scram-sha-256",
        " ). Warning, you will need to reset all passwords after this parameter is updated.",
    ];
}
