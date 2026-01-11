//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# PgHbaEntriesWithMethodTrustOrPasswordShouldNotExists (C002)\n\nThis configuration is extremely insecure and should only be used in a controlled, non-production environment for testing purposes. In a production environment, you should use more secure authentication methods such as md5, scram-sha-256, or cert, and restrict access to trusted IP addresses only.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"cluster\": {\n        \"pgHbaEntriesWithMethodTrustOrPasswordShouldNotExists\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 20%\n- Error level: 80%\n\n## Fixes\n\n- change trust or password method in pg_hba.conf\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#c002>"] pub PgHbaEntriesWithMethodTrustOrPasswordShouldNotExists { version : "1.0.0" , name : "pgHbaEntriesWithMethodTrustOrPasswordShouldNotExists" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for PgHbaEntriesWithMethodTrustOrPasswordShouldNotExists {
    const CODE: &'static str = "C002";
    const SCOPE: &'static str = "CLUSTER";
    const DESCRIPTION: &'static str = "This configuration is extremely insecure and should only be used in a controlled, non-production environment for testing purposes. In a production environment, you should use more secure authentication methods such as md5, scram-sha-256, or cert, and restrict access to trusted IP addresses only.";
    const FIXES: &'static [&'static str] = &["change trust or password method in pg_hba.conf"];
}
