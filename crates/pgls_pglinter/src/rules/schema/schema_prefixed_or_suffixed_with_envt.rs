//! Generated file, do not edit by hand, see `xtask/codegen`

#![doc = r" Generated file, do not edit by hand, see `xtask/codegen`"]
use crate::rule::PglinterRule;
::pgls_analyse::declare_rule! { # [doc = "# SchemaPrefixedOrSuffixedWithEnvt (S002)\n\nThe schema is prefixed with one of staging,stg,preprod,prod,sandbox,sbox string. Means that when you refresh your preprod, staging environments from production, you have to rename the target schema from prod_ to stg_ or something like. It is possible, but it is never easy.\n\n## Configuration\n\nEnable or disable this rule in your configuration:\n\n```json\n{\n  \"pglinter\": {\n    \"rules\": {\n      \"schema\": {\n        \"schemaPrefixedOrSuffixedWithEnvt\": \"warn\"\n      }\n    }\n  }\n}\n```\n\n## Thresholds\n\n- Warning level: 1%\n- Error level: 1%\n\n## Fixes\n\n- Keep the same schema name across environments. Prefer prefix or suffix the database name\n\n## Documentation\n\nSee: <https://github.com/pmpetit/pglinter#s002>"] pub SchemaPrefixedOrSuffixedWithEnvt { version : "1.0.0" , name : "schemaPrefixedOrSuffixedWithEnvt" , severity : pgls_diagnostics :: Severity :: Warning , recommended : true , } }
impl PglinterRule for SchemaPrefixedOrSuffixedWithEnvt {
    const CODE: &'static str = "S002";
    const SCOPE: &'static str = "SCHEMA";
    const DESCRIPTION: &'static str = "The schema is prefixed with one of staging,stg,preprod,prod,sandbox,sbox string. Means that when you refresh your preprod, staging environments from production, you have to rename the target schema from prod_ to stg_ or something like. It is possible, but it is never easy.";
    const FIXES: &'static [&'static str] = &[
        "Keep the same schema name across environments. Prefer prefix or suffix the database name",
    ];
}
